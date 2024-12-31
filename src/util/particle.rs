use crate::util::vector2d::Vector2D;

/// A single two-dimensional particle
#[derive(Copy, Clone)]
pub struct Particle<T> {
    pub(crate) position: Vector2D<T>,
    pub(crate) velocity: Vector2D<T>,
    pub(crate) radius: T,
    pub(crate) mass: T,
}