use embedded_graphics::prelude::Coord;
use crate::display::GameColor;
use embedded_graphics::prelude::Pixel;
use embedded_graphics::prelude::UnsignedCoord;
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

pub struct ImgIterator {
    data: &'static [u8],
    i: usize,
    width: u32,
    pos: Coord,
    r: Option<u8>,
    g: Option<u8>,
    x: i32,
    y: i32,

}

impl ImgIterator {
    pub fn new(data: &'static [u8], width: u32, pos: Coord) -> ImgIterator {
        Self {
            data,
            width,
            pos,
            i: 0,
            r: None,
            g: None,
            x: 0,
            y: 0,
        }
    }
}

impl Iterator for ImgIterator {
    type Item = Pixel<GameColor>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.i >= self.data.len() {
                return None;
            }
            let item = self.data[self.i];
            self.i += 1;
            match (self.r, self.g) {
                (None, _) => self.r = Some(item),
                (Some(_), None) => self.g = Some(item),
                (Some(ri), Some(gi)) => {
                    let color = u32::from(ri).rotate_left(16)
                                | u32::from(gi).rotate_left(8)
                                | u32::from(item);
                    let pc = Pixel(UnsignedCoord::new((self.pos[0] + self.x) as u32,
                                                      (self.pos[1] + self.y) as u32), 
                                   GameColor{value:color});
                    self.x += 1;
                    if self.x >= self.width as i32 {
                        self.x = 0;
                        self.y += 1;
                    }
                    self.r = None;
                    self.g = None;
                    break Some(pc);
                }
            }
        }
    }

}