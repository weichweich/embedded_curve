extern crate alloc;

use math;

use stm32f7_discovery::{
    lcd::{Framebuffer, Layer, Color},
};
use alloc::vec::Vec;
use bresenham::{
    Bresenham
};
use crate::geometry::{
    Point
};


pub struct Curve {
    points: Vec<Point>,
}

impl Curve {

    pub fn new() -> Self {
        Self {
            points: Vec::new(),
        }
    }

    pub fn add_point<F: Framebuffer>(&mut self, p: Point, layer: &mut Layer<F>, color: Color) {
        match self.points.len() {
            0 => {},
            n => draw_line(p, self.points[n], layer, color),
        }
        self.points.push(p);
    }
}

pub struct CurveField {
    curves: Vec<Curve>,
}

impl CurveField {
    pub fn new() -> Self {
        Self {
            curves: Vec::new(),
        }
    }

    pub fn new_curve(&mut self) {
        self.curves.push(Curve::new());
    }

}

pub fn draw_line<F: Framebuffer>(start: Point, end: Point, layer: &mut Layer<F>, color: Color) {
    let bi = Bresenham::new((start.x as isize, start.y as isize), 
                            (end.x as isize, end.y as isize));
    for p in bi {
        layer.print_point_color_at(p.0 as usize, p.1 as usize, color);
    }
}
