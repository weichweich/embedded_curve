extern crate alloc;

use embedded_graphics::prelude::*;
use embedded_graphics::coord::Coord;
use embedded_graphics::primitives::{Circle, Rect};
use alloc::vec::Vec;
use crate::geometry::{
    AABBox, Point, Vector2D
};
use stm32f7_discovery::lcd::{Framebuffer, Layer, Color, HEIGHT, WIDTH};
use crate::display::LcdDisplay;
use core::f32::consts::PI;

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

    pub fn is_active(&self, touches: &[Point]) -> bool {
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
    pos: (f32, f32),
    color: Color,
    direction: Vector2D,
}

impl Player {
    pub fn new(left_input_box: AABBox, right_input_box: AABBox) -> Self {
        Self {
            input_left: InputRegion::new(left_input_box),
            input_right: InputRegion::new(right_input_box),
            pos: (100.0,  100.0),
            color: Color::from_hex(0xFF0000),
            direction: Vector2D {x: 0.2, y: 0.2},
        }
    }

    pub fn get_player_input(&self, touches: &[Point]) -> PlayerInput {
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
        display.draw(Circle::new(Coord::new(self.pos.0 as i32, self.pos.1 as i32), 3)
            .with_stroke(Some(1u8.into()))
            .with_fill(Some(1u8.into()))
            .into_iter());
    }

    fn update_pos(&mut self) {
        let mut new_x = (self.pos.0 + self.direction.x) % WIDTH as f32;
        let mut new_y = (self.pos.1 + self.direction.y) % HEIGHT as f32;
        if new_x < 0.01 {
            new_x += 0.6;
        }
        self.pos = (new_x % (WIDTH as f32), new_y % (HEIGHT as f32));
    }

    pub fn act(&mut self, touches: &[Point]) {
        let a = (2.0 * PI) / 180.0;
        match self.get_player_input(touches) {
            PlayerInput::Left => {
                self.direction = self.direction.rotate(a);
            },
            PlayerInput::Right => {
                self.direction = self.direction.rotate(-a);
            },
            _ => {},
        }
        self.update_pos();
    }
}