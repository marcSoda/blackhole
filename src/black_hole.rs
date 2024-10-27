use egui::{emath::*, *};
use rand::Rng;
use std::f32::consts::PI;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct BlackHoleSimulation {
    particles: Vec<Particle>, // Vector of particles in the simulation
    black_hole: BlackHole,    // The black hole
    max_dist: f32,            // The maximum distance that a particle may travel from the black hole
    particle_radius: f32,     // The radius of the drawn particles
    max_spawn_dist: f32,      // The max distance a particle can spawn from the black hole
    min_spawn_dist: f32,      // The min distance a particle can spawn from the black hole
    paused: bool,             // True if game is paused
    kill_boundary: bool,      // True if max_dist boundary destroys particles. False if it stops them
    dark_mode: bool,          // True if dark mode
}

#[derive(serde::Deserialize, serde::Serialize)]
struct BlackHole {
    position: Pos2,    // Position of black hole
    radius: f32,       // Radius of black hole
    gravity: f32,      // Magnitiude of gravitational pull of black hole
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Particle {
    position: Pos2,   // Position of particle
    velocity: Vec2,   // Velocity of particle
    color: Color32,   // Color of particle
}

impl eframe::App for BlackHoleSimulation {
    // Save state on exit
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    // Update ui every frame
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.ui(ui);
        });
    }
}

impl Default for BlackHoleSimulation {
    fn default() -> Self {
        Self {
            particles: Vec::new(),
            black_hole: BlackHole {
                position: Pos2::new(640.0, 400.0),
                radius: 5.0,
                gravity: 500.0,
            },
            max_dist: 500.0,
            particle_radius: 2.0,
            min_spawn_dist: 50.0,
            max_spawn_dist: 100.0,
            paused: false,
            kill_boundary: true,
            dark_mode: true,
        }
    }
}

impl BlackHoleSimulation {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // restore state if state was saved. default otherwise
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
        Default::default()
    }

    // ui panel
    pub fn ui(&mut self, ui: &mut Ui) {
        egui::Window::new("Simulation Settings")
            .default_size([300.0, 200.0])
            .show(ui.ctx(), |ui| {
                // Horizontal toolbar
                ui.horizontal(|ui| {
                    // Dark/light toggle
                    if ui.button(if self.dark_mode {
                        ui.ctx().set_visuals(egui::Visuals::dark());
                        "Light Mode"
                    } else {
                        ui.ctx().set_visuals(egui::Visuals::light());
                        "Dark Mode"
                    }).clicked() {
                        self.dark_mode = !self.dark_mode;
                    }

                    // Kill/sticky boundary toggle
                    if ui.button(if self.kill_boundary { "Sticky Boundary" } else { "Kill Boundary" }).clicked() {
                        self.kill_boundary = !self.kill_boundary;
                    }
                    // Reset button
                    if ui.button("Reset").clicked() {
                        *self = Self::default();
                    }
                    // Play/Pause toggle
                    if ui.button(if self.paused { "Play" } else { "Pause" }).clicked() {
                        self.paused = !self.paused;
                    }
                });
                // Sliders
                ui.add(egui::Slider::new(&mut self.black_hole.position.x, 0.0..=1600.0).text("Black Hole X Pos"));
                ui.add(egui::Slider::new(&mut self.black_hole.position.y, 0.0..=1200.0).text("Black Hole Y Pos"));
                ui.add(egui::Slider::new(&mut self.black_hole.radius, 0.0..=100.0).text("Black Hole Radius"));
                ui.add(egui::Slider::new(&mut self.black_hole.gravity, 0.0..=10000.0).text("Black Hole Gravity"));
                ui.add(egui::Slider::new(&mut self.max_dist, 50.0..=1000.0).text("Max Distance"));

                // Sliders for min and max spawn distances. Doesn't allow them to overlap
                ui.add(egui::Slider::new(&mut self.min_spawn_dist, 50.0..=1000.0).text("Min Spawn Distance"));
                if self.min_spawn_dist > self.max_spawn_dist {
                    self.max_spawn_dist = self.min_spawn_dist + 1.;
                }
                ui.add(egui::Slider::new(&mut self.max_spawn_dist, 50.0..=1000.0).text("Max Spawn Distance"));
                if self.max_spawn_dist < self.min_spawn_dist {
                    self.min_spawn_dist = self.max_spawn_dist - 1.;
                }

                ui.add(egui::Slider::new(&mut self.particle_radius, 1.0..=10.0).text("Particle Radius"));
            });

        let painter = ui.painter();

        // If there are no more particles, draw some
        if self.particles.is_empty() {
            self.particles = (0..50).map(|_| Particle::new_random(self.black_hole.position, self.min_spawn_dist, self.max_spawn_dist)).collect();
        }

        // Update particles
        self.update();
        // Draw everything
        self.paint(painter);
        ui.ctx().request_repaint();
    }

    fn update(&mut self) {
        if self.paused { return; }

        let mut new_particles = Vec::new();

        self.particles.retain_mut(|particle| {
            let direction_to_black_hole = (self.black_hole.position - particle.position).normalized();
            let distance = (self.black_hole.position - particle.position).length();

            // Particle is absorbed by the black hole; spawn two new particles
            if distance <= self.black_hole.radius {
                new_particles.push(Particle::new_random(self.black_hole.position, self.min_spawn_dist, self.max_spawn_dist));
                new_particles.push(Particle::new_random(self.black_hole.position, self.min_spawn_dist, self.max_spawn_dist));
                return false; // Remove absorbed particle
            }

            // Apply gravitational force
            let force_magnitude = self.black_hole.gravity / distance.powi(2);
            particle.velocity += direction_to_black_hole * force_magnitude;
            let next_position = particle.position + particle.velocity;

            // Check if the particle will go beyond boundary
            if (self.black_hole.position - next_position).length() > self.max_dist {
                if self.kill_boundary { return false; } // Remove particle if kill boundry
                else { // Stop particle if not kill boundary
                    particle.position = self.black_hole.position - direction_to_black_hole * self.max_dist; // Put particle on boundary
                    particle.velocity *= 0.1;
                }
            } else {
                // Update the position normally
                particle.position = next_position;
            }

            true // Keep particle
        });

        self.particles.extend(new_particles); // Add new particles
    }

    // paint everything
    fn paint(&self, painter: &Painter) {
        painter.circle_filled(self.black_hole.position, self.black_hole.radius, Color32::BLACK);
        painter.circle_stroke(self.black_hole.position, self.black_hole.radius, Stroke::new(1.0, Color32::WHITE));
        painter.circle_stroke(
            self.black_hole.position,
            self.max_dist,
            Stroke::new(1.0, Color32::GRAY)
        );
        for particle in &self.particles {
            painter.circle_filled(particle.position, self.particle_radius, particle.color);
        }
    }
}

impl Particle {
    // Generate a new random particle
    fn new_random(black_hole_center: Pos2, min_distance: f32, max_distance: f32) -> Self {
        let mut rng = rand::thread_rng();
        let angle = rng.gen_range(0.0..=2.0 * PI);
        let distance = rng.gen_range(min_distance..=max_distance);
        let position = black_hole_center + Vec2::angled(angle) * distance;
        let tangential_velocity_magnitude = rng.gen_range(1.0..=3.0);
        let tangential_velocity = Vec2::new(-angle.sin(), angle.cos()) * tangential_velocity_magnitude;

        // Random color particle
        let color = Color32::from_rgb(
            rng.gen_range(0..=255),
            rng.gen_range(0..=255),
            rng.gen_range(0..=255),
        );

        Self {
            position,
            velocity: tangential_velocity,
            color,
        }
    }
}
