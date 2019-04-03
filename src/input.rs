extern crate alloc;

use embedded_graphics::prelude::*;
use embedded_graphics::coord::Coord;
use embedded_graphics::primitives::{Circle, Rect};
use alloc::vec::Vec;
use crate::geometry::{
    AABBox, Point, Vector2D
};
use stm32f7_discovery::lcd::{Framebuffer, Layer, Color};
use crate::display::LcdDisplay;

pub enum PlayerInput {
    Left,
    Right,
    Both,
    None
}

struct InputRegion {
    sensitive_rect: AABBox
}

impl InputRegion {
    pub fn new(boxx: AABBox) -> Self {
        Self {
            sensitive_rect: boxx
        }
    }

    pub fn is_active(&self, touches: &Vec<Point>) -> bool {
        for touch in touches {
            if self.sensitive_rect.inside(touch.clone()) {
                return true;
            }
        }
        false
    }
}

pub struct Player {
    input_left: InputRegion,
    input_right: InputRegion,
    pos: Point,
    color: Color,
    direction: Vector2D
}

impl Player {
    pub fn new(left_input_box: AABBox, right_input_box: AABBox) -> Self {
        Self {
            input_left: InputRegion::new(left_input_box),
            input_right: InputRegion::new(right_input_box),
            pos: Point {x: 100, y: 100},
            color: Color::from_hex(0xFF0000),
            direction: Vector2D {x: 1, y: 1},
        }
    }

    pub fn get_player_input(&self, touches: &Vec<Point>) -> PlayerInput {
        let push_left = self.input_left.is_active(touches);
        let push_right = self.input_right.is_active(touches);

        match (push_left, push_right) {
            (true, true) => PlayerInput::Both,
            (true, false) => PlayerInput::Left,
            (false, true) => PlayerInput::Right,
            (false, false) => PlayerInput::None,
        }
    }

    pub fn draw<F: Framebuffer>(&self, display: &mut LcdDisplay<F>) {
        display.draw(Circle::new(Coord::new(self.pos.x as i32, self.pos.y as i32), 10)
            .with_stroke(Some(1u8.into()))
            .with_fill(Some(1u8.into()))
            .into_iter());
    }

    pub fn act(&mut self) {
        self.pos = self.pos + self.direction;
    }
}