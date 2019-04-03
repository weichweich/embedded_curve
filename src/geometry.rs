use core::ops::{Add, Sub};
use libm::{cosf, sinf};

#[derive(Copy,Clone)]
pub struct Vector2D {
    pub x: f32,
    pub y: f32,
}

impl Vector2D {
    pub fn rotate(self, angle: f32) -> Vector2D {
        let ca = cosf(angle);
        let sa = sinf(angle);
        Vector2D {
            x: ca * self.x - sa * self.y,
            y: sa * self.x + ca * self.y,
        }
    }
}

impl Add<Point> for Vector2D {
    type Output = Point;

    fn add(self, other: Point) -> Point {
        let new_x = self.x + (other.x as f32);
        let new_y = self.y + (other.y as f32);
        assert!(new_x >= 0.0, "New X value is less than 0!");
        assert!(new_y >= 0.0, "New Y value is less than 0!");

        Point {
            x: (new_x as usize), y: (new_y as usize),
        }
    }
}

impl Add for Vector2D {
    type Output = Vector2D;

    fn add(self, other: Vector2D) -> Vector2D {
        Vector2D {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Vector2D {
    type Output = Vector2D;

    fn sub(self, other: Vector2D) -> Vector2D {
        Vector2D {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

#[derive(Copy,Clone)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

impl Add for Point {
    type Output = Point;

    fn add(self, other: Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Add<Vector2D> for Point {
    type Output = Point;

    fn add(self, other: Vector2D) -> Point {
        let new_x = (self.x as f32) + other.x;
        let new_y = (self.y as f32) + other.y;
        assert!(new_x >= 0.0, "New X value is less than 0!");
        assert!(new_y >= 0.0, "New Y value is less than 0!");

        Point {
            x: (new_x as usize), y: (new_y as usize),
        }
    }
}

#[derive(Copy,Clone)]
pub struct AABBox {
    top_left: Point,
    bottom_right: Point
}

impl AABBox {
    pub fn new(top_left: Point, bottom_right: Point) -> Self {
        assert!(top_left.x < bottom_right.x, "Left point is more to the right than right point");
        assert!(top_left.y < bottom_right.y, "Top point is more to the bottom than bottom point");
        AABBox {
            top_left,
            bottom_right,
        }
    }

    pub fn inside(&self, point: Point) -> bool {
        (self.top_left.x <= point.x && self.top_left.y <= point.y
        && point.x <= self.bottom_right.x && point.y <= self.bottom_right.y)
    }
}