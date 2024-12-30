extern crate core;

use std::time::Instant;

use rand::Rng;
use speedy2d::{Graphics2D, Window};
use speedy2d::color::Color;
use speedy2d::window::{WindowHandler, WindowHelper, WindowStartupInfo};

use util::vector2d::Vector2D;

use crate::util::particle_quad_tree::{ParticleQuadTree, QuadtreeVisitor};

pub mod util;

/// A single particle that will be simulated.
#[derive(Clone)]
pub struct Particle<T> {
    position: Vector2D<T>,
    velocity: Vector2D<T>,
    radius: T,
    mass: T,
}

struct Universe<T> {
    particles: Vec<Particle<T>>,
    grav_const: T,
}

fn main() {
    const WIDTH: f32 = 1600.0;
    const HEIGHT: f32 = 900.0;
    let window = Window::new_centered("Particles", (WIDTH as u32, HEIGHT as u32)).unwrap();
    window.run_loop(UniverseWindowHandler {
        universe: Universe {
            particles: Vec::new(),
            grav_const: 10.0,
        },
        last_tick: Instant::now(),
    })
}

struct UniverseWindowHandler
{
    universe: Universe<f32>,
    last_tick: Instant,
}

impl WindowHandler for UniverseWindowHandler
{
    fn on_start(&mut self, _helper: &mut WindowHelper<()>, _info: WindowStartupInfo) {
        // initialize particles
        const NUM_PARTICLES: usize = 10_000;
        let mut rng = rand::thread_rng();
        (0..NUM_PARTICLES).for_each(|_| {
            let x = rng.gen_range(100.0..1400.0);
            let y = rng.gen_range(100.0..800.0);
            self.universe.particles.push(create_particle(x, y));
        });
    }

    fn on_draw(&mut self, helper: &mut WindowHelper, graphics: &mut Graphics2D)
    {
        // do calculations
        self.last_tick = Instant::now();

        // draw graphics
        graphics.clear_screen(Color::BLACK);

        // find bounds of the universe
        let mut min_x = self.universe.particles[0].position.x;
        let mut max_x = min_x;
        let mut min_y = self.universe.particles[0].position.y;
        let mut max_y = min_y;
        let num_particles = self.universe.particles.len();
        (1..num_particles).for_each(|i| {
            let x = self.universe.particles[i].position.x;
            let y = self.universe.particles[i].position.y;
            if x < min_x {
                min_x = x;
            }
            if x > max_x {
                max_x = x;
            }
            if y < min_y {
                min_y = y;
            }
            if y > max_y {
                max_y = y;
            }
        });
        let width = max_x - min_x;
        let height = max_y - min_y;
        let cx = (min_x + max_x) / 2.0;
        let cy = (min_y + max_y) / 2.0;

        // create the temporary quadtree
        let mut quadtree = ParticleQuadTree::new(
            Vector2D {
                x: cx,
                y: cy,
            },
            width,
            height,
            100);
        let num_particles = self.universe.particles.len();
        (0..num_particles).for_each(|i| { quadtree.insert(&self.universe.particles, i); });

        quadtree.tick(&mut self.universe.particles, self.universe.grav_const, 1.0 / 30.0);

        let mut tree_visitor = WindowHandlerTreeVisitor {
            graphics,
            universe: &self.universe,
            univ_width: width,
            univ_height: height,
            univ_center: Vector2D {
                x: cx,
                y: cy,
            },
            screen_width: helper.get_size_pixels().x as f32,
            screen_height: helper.get_size_pixels().y as f32,
        };
        quadtree.visit(&mut tree_visitor);

        // Request that we draw another frame once this one has finished
        helper.request_redraw();
    }
}

/// Creates a random particle.
fn create_particle(x: f32, y: f32) -> Particle<f32> {
    let mass: f32 = 1.0;
    Particle {
        position: Vector2D {
            x,
            y,
        },
        velocity: Vector2D { x: 0.0, y: 0.0 },
        radius: 1.0,
        mass,
    }
}

struct WindowHandlerTreeVisitor<'a, T> {
    graphics: &'a mut Graphics2D,
    universe: &'a Universe<T>,
    univ_width: f32,
    univ_height: f32,
    univ_center: Vector2D<f32>,
    screen_width: f32,
    screen_height: f32,
}

impl QuadtreeVisitor<f32> for WindowHandlerTreeVisitor<'_, f32> {
    fn visit_node(&mut self, _tree: &ParticleQuadTree<f32>) {
        // nop
    }

    fn visit_leaf_node(&mut self, _tree: &ParticleQuadTree<f32>, _element_indices: &Vec<usize>) {
        // nop
    }

    fn visit_element(&mut self, element_index: usize) {
        let element = self.universe.particles.get(element_index).unwrap();
        let screen_pos = self.local_to_screen(element.position);
        self.graphics.draw_circle((screen_pos.x, screen_pos.y), element.radius, Color::WHITE);
    }
}

impl WindowHandlerTreeVisitor<'_, f32> {
    fn local_to_screen(&self, p: Vector2D<f32>) -> Vector2D<f32> {
        let factor;
        if self.univ_width > self.univ_height {
            factor = self.univ_width;
        }
        else {
            factor = self.univ_height;
        }
        Vector2D {
            x: (p.x - self.univ_center.x) / factor * self.screen_width + self.screen_width / 2.0,
            y: (p.y - self.univ_center.y) / factor * self.screen_height + self.screen_height / 2.0,
        }
    }
}