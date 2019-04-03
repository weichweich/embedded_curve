use core::ops::Add;

#[derive(Copy,Clone)]
pub struct Vector2D {
    pub x: isize,
    pub y: isize,
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
        let new_x = (self.x as isize) + other.x;
        let new_y = (self.y as isize) + other.y;
        assert!(new_x >= 0, "New X value is less than 0!");
        assert!(new_y >= 0, "New Y value is less than 0!");

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
            top_left: top_left,
            bottom_right: bottom_right,
        }
    }

    pub fn inside(&self, point: Point) -> bool {
        (self.top_left.x <= point.x && self.top_left.y <= point.y
        && point.x <= self.bottom_right.x && point.y <= self.bottom_right.y)
    }
}