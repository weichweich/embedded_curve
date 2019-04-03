extern crate alloc;

use stm32f7_discovery::{
    lcd::{Framebuffer, Layer, Color},
};
use alloc::vec::Vec;

use crate::geometry::{
    Point
};

use crate::draw::{
    draw_line
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
