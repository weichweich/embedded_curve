use embedded_graphics::prelude::Coord;
use crate::display::GameColor;
use embedded_graphics::prelude::Pixel;
use embedded_graphics::prelude::UnsignedCoord;
use core::ops::{Add, Sub, Mul};
use libm::{cosf, sinf};

#[derive(Copy,Clone, Debug)]
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

    pub fn length(self) -> f32 {
        libm::sqrtf(self.x*self.x + self.y*self.y)
    }

    pub fn dot(self, other: Vector2D) -> f32 {
        self.x*other.x + self.y*other.y
    }

    pub fn normalized(self) -> Vector2D {
        let l = self.length();
        Vector2D {x: self.x/l, y:self.y/l}
    }

    pub fn distance(self, other: Vector2D) -> f32 {
        let vec = self - other;
        libm::sqrtf(vec.dot(vec))
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

impl Mul<f32> for Vector2D {
    type Output = Vector2D;

    fn mul(self, scalar: f32) -> Vector2D {
        Vector2D {x: self.x*scalar, y: self.y * scalar}
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
    top_left: Coord,
    bottom_right: Coord
}

impl AABBox {
    pub fn new(top_left: Coord, bottom_right: Coord) -> Self {
        assert!(top_left[0] < bottom_right[0], "Left point is more to the right than right point");
        assert!(top_left[1] < bottom_right[1], "Top point is more to the bottom than bottom point");
        AABBox {
            top_left,
            bottom_right,
        }
    }

    pub fn inside(&self, point: Coord) -> bool {
        (self.top_left[0] <= point[0] && self.top_left[1] <= point[1]
        && point[0] <= self.bottom_right[0] && point[1] <= self.bottom_right[1])
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