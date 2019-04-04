extern crate alloc;

use embedded_graphics::prelude::*;
use embedded_graphics::coord::Coord;
use embedded_graphics::primitives::{Circle, Rect};
use alloc::vec::Vec;
use crate::geometry::{
    AABBox, Point, Vector2D
};
use stm32f7_discovery::lcd::{Framebuffer, Layer, HEIGHT, WIDTH};
use crate::display::{LcdDisplay, GameColor};
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
    color: GameColor,
    direction: Vector2D,
    radius: u32,
}

impl Player {
    pub fn new(left_input_box: AABBox, right_input_box: AABBox, color: GameColor,
               start_pos: (f32, f32), radius: u32, direction: Vector2D) -> Self {
        Self {
            input_left: InputRegion::new(left_input_box),
            input_right: InputRegion::new(right_input_box),
            pos: start_pos,
            color,
            direction,
            radius,
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
        display.draw(Circle::new(Coord::new(self.pos.0 as i32, self.pos.1 as i32), self.radius)
            .with_stroke(Some(self.color))
            .with_fill(Some(self.color))
            .into_iter());
    }

    fn update_pos(&mut self) {
        let mut new_x = (self.pos.0 + self.direction.x) % WIDTH as f32;
        let mut new_y = (self.pos.1 + self.direction.y) % HEIGHT as f32;
        if new_x < 0.0 {
            new_x = WIDTH as f32 - 1.0;
        } else if new_x > WIDTH as f32 {
            new_x = 0.5;
        }
        if new_y < 0.0 {
            new_y = HEIGHT as f32 - 1.0;
        } else if new_y > HEIGHT as f32 {
            new_y = 0.5;
        }
        self.pos = (new_x, new_y);
    }

    pub fn act(&mut self, touches: &[Point]) {
        let a = 5.0 * (PI) / 180.0;
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