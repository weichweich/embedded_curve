use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle};
use stm32f7_discovery::lcd::{Framebuffer, HEIGHT, WIDTH};
use core::f32::consts::PI;
use alloc::vec::Vec;

use crate::geometry::{
    AABBox, Point, Vector2D
};
use crate::display::{LcdDisplay, GameColor};
use crate::buffs::{PlayerBuff, Buff};

use crate::playingfield::PlayingField;


pub trait Collide<T> {
    fn collides_with(&self, incoming: &T) -> bool;
}

pub trait CollideSelf {
    fn collides(&self) -> bool;
}


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
    id: u8,
    direction: Vector2D,
    radius: u32,
    speed: f32,
    buffs: Vec<PlayerBuff>,
    trace: Vec<(f32, f32, u32)>,    //pos_x, pos_y,radius
}

impl Player {
    pub fn new(left_input_box: AABBox, right_input_box: AABBox, color: GameColor,
               start_pos: (f32, f32), radius: u32, angle: f32, id: u8) -> Self {
        let a = angle * (PI) / 180.0;

        let mut s = Self {
            input_left: InputRegion::new(left_input_box),
            input_right: InputRegion::new(right_input_box),
            pos: start_pos,
            color,
            id,
            direction: Vector2D{x: 1.0, y: 0.0}.rotate(a),
            speed: 1.0,
            radius,
            buffs: Vec::new(),
            trace: Vec::new(),
        };
        s.trace.push( (start_pos.0, start_pos.1, radius) );     //segment extends by updating
        s.trace.push( (start_pos.0, start_pos.1, radius) );
        s
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

    pub fn draw<F: Framebuffer>(&self, display: &mut LcdDisplay<F>, 
                                playing_field: &mut PlayingField) {
        let color = self.buffs
                        .iter()
                        .fold(self.color, |acc, func| (func.change_color)(acc));

        let circle_iter = 
        Circle::new(Coord::new(self.pos.0 as i32, self.pos.1 as i32), self.radius)
            .with_stroke(Some(color))
            .with_fill(Some(color))
            .into_iter();
            
        display.draw(circle_iter);
        playing_field.store(circle_iter, self.id);
    }

    fn update_pos(&mut self, new_trace_segment: bool) {
        let speed = self.buffs
                        .iter()
                        .fold(self.speed, |acc, func| (func.change_speed)(acc));
        let mut new_x = (self.pos.0 + self.direction.x * speed) % WIDTH as f32;
        let mut new_y = (self.pos.1 + self.direction.y * speed) % HEIGHT as f32;
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
        
        let tracepoint : (f32, f32, u32) = (self.pos.0, self.pos.1, self.radius);
        if new_trace_segment {
            self.trace.push( tracepoint );
            self.trace.push( tracepoint );
        } else {
            self.trace[self.trace.len()-1] = tracepoint;
        }
    }

    fn update_buffs(&mut self) {
        let mut i = 0;
        while i < self.buffs.len() {
            if self.buffs[i].timeout == 0 {
                self.buffs.remove(i);
            } else {
                self.buffs[i].timeout -= 1;
                i += 1;
            }
        }
    }

    pub fn act(&mut self, touches: &[Point]) {
        let d = self.buffs.iter().fold(5.0, |acc, func| (func.change_rotation)(acc));
        let a = d * (PI) / 180.0;
        let mut new_trace_segment: bool = false;
        match self.get_player_input(touches) {
            PlayerInput::Left => {
                self.direction = self.direction.rotate(-a);
                new_trace_segment = true;
            },
            PlayerInput::Right => {
                self.direction = self.direction.rotate(a);
                new_trace_segment = true;
            },
            _ => {},
        }
        self.update_pos(new_trace_segment);
        self.update_buffs();
    }

    pub fn add_buff(&mut self, buff: PlayerBuff) {
        self.buffs.push(buff);
    }
}

impl CollideSelf for Player {
    fn collides(&self) -> bool {
        false
    }
}

impl<T: Buff> Collide<T> for Player {
    fn collides_with(&self, incoming: &T) -> bool {
        false
    }
}

impl Collide<Player> for Player {
    fn collides_with(&self, incoming: &Player) -> bool {
        let trace_pos1 = self.trace.last().unwrap();
        let trace_pos2 = incoming.trace.last().unwrap();

        let p1_pos = (trace_pos1.0, trace_pos1.1);
        let p2_pos = (trace_pos2.0, trace_pos2.1);

        let r1 = trace_pos1.2;
        let r2 = trace_pos2.2;

        false
    }
}
