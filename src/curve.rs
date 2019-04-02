extern crate alloc;

use stm32f7_discovery::{
    lcd::{Framebuffer, Layer, Color},
};
use alloc::vec::Vec;


pub struct Point {
    pub x: usize,
    pub y: usize,
}


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
            n => draw_line(&p, &self.points[n], layer, color),
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

pub fn draw_line<F: Framebuffer>(start: &Point, end: &Point, layer: &mut Layer<F>,
                             color: Color) {
    layer.print_point_color_at(start.x, start.y, color);
    layer.print_point_color_at(end.x, end.y, color);
}