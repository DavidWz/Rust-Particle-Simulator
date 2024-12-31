use std::ops::{Add, Div, Mul, Sub};

use crate::util::vector2d::Vector2D;
use crate::Particle;

pub trait QuadtreePointValue<T> {
    fn from(value: usize) -> T;
}

impl QuadtreePointValue<f32> for f32 {
    fn from(value: usize) -> f32 {
        value as f32
    }
}

pub struct ParticleQuadTree<T> {
    pub center: Vector2D<T>,
    summary_particle: Particle<T>,
    pub width: T,
    pub height: T,
    max_capacity: usize,
    num_elements: usize,
    node: QuadtreeNode<T>,
}

enum QuadtreeNode<T> {
    Node {
        top_left: Box<ParticleQuadTree<T>>,
        top_right: Box<ParticleQuadTree<T>>,
        bottom_left: Box<ParticleQuadTree<T>>,
        bottom_right: Box<ParticleQuadTree<T>>,
    },
    Leaf {
        element_indices: Vec<usize>, // only stores indices to actual particles
    },
}

pub trait QuadtreeVisitor<T> {
    fn visit_node(&mut self, tree: &ParticleQuadTree<T>);
    fn visit_leaf_node(&mut self, tree: &ParticleQuadTree<T>, element_indices: &Vec<usize>);
    fn visit_element(&mut self, index: usize);
}

impl<
        T: Copy
            + Default
            + QuadtreePointValue<T>
            + PartialOrd
            + Sub<Output = T>
            + Add<Output = T>
            + Mul<Output = T>
            + Div<Output = T>
            + std::fmt::Display,
    > ParticleQuadTree<T>
{
    pub(crate) fn new(
        center: Vector2D<T>,
        width: T,
        height: T,
        max_capacity: usize,
    ) -> ParticleQuadTree<T> {
        ParticleQuadTree {
            center,
            summary_particle: Particle {
                position: center,
                velocity: Default::default(),
                radius: Default::default(),
                mass: Default::default(),
            },
            width,
            height,
            max_capacity,
            num_elements: 0,
            node: QuadtreeNode::Leaf {
                element_indices: Vec::with_capacity(max_capacity),
            },
        }
    }

    pub(crate) fn insert(&mut self, elements: &Vec<Particle<T>>, index: usize) {
        let element = elements.get(index).unwrap();

        // update summary particle
        if self.num_elements == 0 {
            self.summary_particle.position = element.position;
            self.summary_particle.mass = element.mass;
            self.num_elements = 1;
        } else {
            let cx = self.summary_particle.position.x;
            let cy = self.summary_particle.position.y;
            let n = <T as QuadtreePointValue<T>>::from(self.num_elements);
            let n_plus = <T as QuadtreePointValue<T>>::from(self.num_elements + 1);
            self.summary_particle.position = Vector2D {
                x: cx.mul(n).add(element.position.x).div(n_plus),
                y: cy.mul(n).add(element.position.y).div(n_plus),
            };
            self.summary_particle.mass = self.summary_particle.mass + element.mass;
            self.num_elements = self.num_elements + 1;
        }

        // recursion: add element to correct child node
        match self.node {
            QuadtreeNode::Node {
                ref mut top_left,
                ref mut top_right,
                ref mut bottom_left,
                ref mut bottom_right,
            } => {
                if element.position.x <= self.center.x {
                    // left
                    if element.position.y <= self.center.y {
                        // top
                        top_left.insert(elements, index);
                    } else {
                        // bottom
                        bottom_left.insert(elements, index);
                    }
                } else {
                    // right
                    if element.position.y <= self.center.y {
                        // top
                        top_right.insert(elements, index);
                    } else {
                        // bottom
                        bottom_right.insert(elements, index);
                    }
                }
            }
            QuadtreeNode::Leaf {
                ref mut element_indices,
            } => {
                if element_indices.len() < self.max_capacity {
                    // recursion end: add element to list of elements
                    element_indices.push(index);
                } else {
                    // if maximum capacity is reached, we need to split the elements into four quads
                    let two = <T as QuadtreePointValue<T>>::from(2);
                    let half_width = (self.width + two) / two; // increase slightly because of fuzzy floating point comparisons
                    let half_height = (self.height + two) / two;
                    let quarter_width = half_width / two;
                    let quarter_height = half_height / two;
                    let cx = self.center.x;
                    let cy = self.center.y;

                    let mut top_left = Box::new(ParticleQuadTree::new(
                        Vector2D {
                            x: cx.sub(quarter_width),
                            y: cy.sub(quarter_height),
                        },
                        half_width,
                        half_height,
                        self.max_capacity,
                    ));

                    let mut top_right = Box::new(ParticleQuadTree::new(
                        Vector2D {
                            x: cx.add(quarter_width),
                            y: cy.sub(quarter_height),
                        },
                        half_width,
                        half_height,
                        self.max_capacity,
                    ));

                    let mut bottom_left = Box::new(ParticleQuadTree::new(
                        Vector2D {
                            x: cx.sub(quarter_width),
                            y: cy.add(quarter_height),
                        },
                        half_width,
                        half_height,
                        self.max_capacity,
                    ));

                    let mut bottom_right = Box::new(ParticleQuadTree::new(
                        Vector2D {
                            x: cx.add(quarter_width),
                            y: cy.add(quarter_height),
                        },
                        half_width,
                        half_height,
                        self.max_capacity,
                    ));

                    while !element_indices.is_empty() {
                        let element_index = element_indices.swap_remove(0);
                        let element = elements.get(element_index).unwrap();

                        if element.position.x < cx {
                            // left
                            if element.position.y < cy {
                                // top

                                top_left.insert(elements, element_index);
                            } else {
                                // bottom
                                bottom_left.insert(elements, element_index);
                            }
                        } else {
                            // right
                            if element.position.y < cy {
                                // top
                                top_right.insert(elements, element_index);
                            } else {
                                // bottom
                                bottom_right.insert(elements, element_index);
                            }
                        }
                    }

                    self.node = QuadtreeNode::Node {
                        top_left,
                        top_right,
                        bottom_left,
                        bottom_right,
                    }
                }
            }
        }
    }

    pub fn visit(&self, visitor: &mut dyn QuadtreeVisitor<T>) {
        match self.node {
            QuadtreeNode::Node {
                ref top_left,
                ref top_right,
                ref bottom_left,
                ref bottom_right,
            } => {
                visitor.visit_node(self);
                top_left.visit(visitor);
                top_right.visit(visitor);
                bottom_left.visit(visitor);
                bottom_right.visit(visitor);
            }
            QuadtreeNode::Leaf {
                ref element_indices,
            } => {
                visitor.visit_leaf_node(self, element_indices);
                for element_index in element_indices {
                    visitor.visit_element(*element_index);
                }
            }
        }
    }

    pub fn tick(&self, elements: &mut Vec<Particle<T>>, grav_const: T, elapsed_s: T) {
        self.tick_with_summaries(elements, grav_const, elapsed_s, &Vec::new());
    }

    fn tick_with_summaries(
        &self,
        elements: &mut Vec<Particle<T>>,
        grav_const: T,
        elapsed_s: T,
        summaries: &Vec<Particle<T>>,
    ) {
        match &self.node {
            QuadtreeNode::Node {
                top_left,
                top_right,
                bottom_left,
                bottom_right,
            } => {
                top_left.tick_with_summaries(
                    elements,
                    grav_const,
                    elapsed_s,
                    &ParticleQuadTree::create_summaries(
                        summaries,
                        top_right,
                        bottom_left,
                        bottom_right,
                    ),
                );
                top_right.tick_with_summaries(
                    elements,
                    grav_const,
                    elapsed_s,
                    &ParticleQuadTree::create_summaries(
                        summaries,
                        top_left,
                        bottom_left,
                        bottom_right,
                    ),
                );
                bottom_left.tick_with_summaries(
                    elements,
                    grav_const,
                    elapsed_s,
                    &ParticleQuadTree::create_summaries(
                        summaries,
                        top_right,
                        top_left,
                        bottom_right,
                    ),
                );
                bottom_right.tick_with_summaries(
                    elements,
                    grav_const,
                    elapsed_s,
                    &ParticleQuadTree::create_summaries(
                        summaries,
                        top_right,
                        bottom_left,
                        top_left,
                    ),
                );
            }
            QuadtreeNode::Leaf { element_indices } => {
                // simple gravitational pull
                let mut delta_velocities = Vec::new();
                let num_particles = element_indices.len();
                (0..num_particles).for_each(|i| {
                    let mut delta_v: Vector2D<T> = Vector2D {
                        x: Default::default(),
                        y: Default::default(),
                    };

                    // calculate gravitational pull for every particle in the same node
                    (0..num_particles).for_each(|j| {
                        if i == j {
                            return;
                        }

                        let p1 = elements.get(*element_indices.get(i).unwrap()).unwrap();
                        let p2 = elements.get(*element_indices.get(j).unwrap()).unwrap();
                        let m2 = p2.mass;
                        let v_dir = p2.position - p1.position;
                        let r_sq = v_dir.length_sq();
                        let a1 = grav_const * m2 / r_sq;
                        delta_v = &delta_v + (v_dir * a1 * elapsed_s);
                    });

                    // calculate pull for the summaries of other nodes
                    for summary_particle in summaries {
                        let p1 = elements.get(*element_indices.get(i).unwrap()).unwrap();
                        let p2 = summary_particle;
                        let m2 = p2.mass;
                        let v_dir = p2.position - p1.position;
                        let r_sq = v_dir.length_sq();
                        let a1 = grav_const * m2 / r_sq;
                        delta_v = &delta_v + (v_dir * a1 * elapsed_s);
                    }

                    delta_velocities.push(delta_v);
                });

                // add delta velocities to total values and update position
                (0..num_particles).for_each(|i| {
                    let particle = elements.get_mut(*element_indices.get(i).unwrap()).unwrap();
                    particle.velocity = &particle.velocity + delta_velocities.get(i).unwrap();
                    particle.position = &particle.position + (&particle.velocity * elapsed_s);
                });
            }
        }
    }

    fn create_summaries(
        original: &Vec<Particle<T>>,
        tree1: &Box<ParticleQuadTree<T>>,
        tree2: &Box<ParticleQuadTree<T>>,
        tree3: &Box<ParticleQuadTree<T>>,
    ) -> Vec<Particle<T>> {
        let mut summaries = original.clone();
        summaries.push(tree1.summary_particle.clone());
        summaries.push(tree2.summary_particle.clone());
        summaries.push(tree3.summary_particle.clone());
        summaries
    }
}
