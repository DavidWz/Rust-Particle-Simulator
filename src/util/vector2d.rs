use std::ops::{Add, Mul, Sub};

/// A two-dimensional vector of type <T>
#[derive(Copy, Clone)]
pub struct Vector2D<T> {
    pub(crate) x: T,
    pub(crate) y: T,
}

impl<T: Default> Default for Vector2D<T> {
    fn default() -> Self {
        Vector2D {
            x: Default::default(),
            y: Default::default(),
        }
    }
}

impl<T: Copy + Sub<Output = T>> Sub<&Vector2D<T>> for &Vector2D<T> {
    type Output = Vector2D<T>;

    fn sub(self, rhs: &Vector2D<T>) -> Self::Output {
        Vector2D {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T: Copy + Sub<Output = T>> Sub<Vector2D<T>> for Vector2D<T> {
    type Output = Vector2D<T>;

    fn sub(self, rhs: Vector2D<T>) -> Self::Output {
        Vector2D {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T: Copy + Add<Output = T>> Add<Vector2D<T>> for &Vector2D<T> {
    type Output = Vector2D<T>;

    fn add(self, rhs: Vector2D<T>) -> Self::Output {
        Vector2D {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T: Copy + Add<Output = T>> Add<&Vector2D<T>> for &Vector2D<T> {
    type Output = Vector2D<T>;

    fn add(self, rhs: &Vector2D<T>) -> Self::Output {
        Vector2D {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T: Copy + Mul<Output = T>> Mul<T> for Vector2D<T> {
    type Output = Vector2D<T>;

    fn mul(self, rhs: T) -> Self::Output {
        Vector2D {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl<T: Copy + Mul<Output = T>> Mul<T> for &Vector2D<T> {
    type Output = Vector2D<T>;

    fn mul(self, rhs: T) -> Self::Output {
        Vector2D {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl<T: Copy + Add<Output = T> + Mul<Output = T>> Vector2D<T> {
    pub(crate) fn length_sq(&self) -> T {
        self.x * self.x + self.y * self.y
    }
}
