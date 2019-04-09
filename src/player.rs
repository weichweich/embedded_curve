use libm;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, Line};
use stm32f7_discovery::lcd::{HEIGHT, WIDTH};
use core::f32::consts::PI;
use alloc::{
    vec::Vec,
    boxed::Box
};

use crate::geometry::{
    AABBox, Vector2D
};
use crate::display::GameColor;
use crate::buffs::{PlayerBuff, Buff};


pub const PAD_LEFT: f32 = 10_f32;
pub const PAD_RIGHT: f32 = 10_f32;


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

    pub fn is_active(&self, touches: &[Coord]) -> bool {
        for touch in touches {
            if self.sensitive_rect.inside(touch.clone()) {
                return true;
            }
        }
        false
    }
}


#[derive(Copy, Clone, Debug)]
struct CurveSegment {
    pub start: Vector2D,
    pub end: Vector2D,
    pub radius: u32,
}


pub struct Player {
    input_left: InputRegion,
    input_right: InputRegion,
    pos: Vector2D,
    color: GameColor,
    direction: Vector2D,
    radius: u32,
    speed: f32,
    buffs: Vec<PlayerBuff>,
    trace: Vec<CurveSegment>,
    pub score: i32,
}

impl Player {
    pub fn new(left_input_box: AABBox, right_input_box: AABBox, color: GameColor,
               start_pos: (f32, f32), radius: u32, angle: f32) -> Self {
        let a = angle * PI / 180.0;
        let pos = Vector2D {x: start_pos.0, y: start_pos.1};
        let mut trace: Vec<CurveSegment> = Vec::new();
        trace.push(CurveSegment{start:pos, end: pos, radius});

        Self {
            input_left: InputRegion::new(left_input_box),
            input_right: InputRegion::new(right_input_box),
            pos,
            color,
            direction: Vector2D{x: 1.0, y: 0.0}.rotate(a),
            speed: 1.0,
            radius,
            buffs: Vec::new(),
            trace,
            score: 0,
        }
    }

    pub fn get_player_input(&self, touches: &[Coord]) -> PlayerInput {
        let push_left = self.input_left.is_active(touches);
        let push_right = self.input_right.is_active(touches);

        match (push_left, push_right) {
            (true, true) => PlayerInput::Both,
            (true, false) => PlayerInput::Left,
            (false, true) => PlayerInput::Right,
            (false, false) => PlayerInput::None,
        }
    }

    pub fn draw<D: Drawing<GameColor>>(&self, display: &mut D) {
        let color = self.buffs
                        .iter()
                        .fold(self.color, |acc, func| (func.change_color)(acc));
        let radius = self.buffs
                         .iter()
                         .fold(self.radius as f32, |acc, func| (func.change_radius)(acc));

        let circle_iter =  Circle::new(Coord::new(self.pos.x as i32,
                                                  self.pos.y as i32),
                                       libm::roundf(radius) as u32)
                                .with_stroke(Some(color))
                                .with_fill(Some(color))
                                .into_iter();
                                
        if cfg!(debug_assertions) {
            let seg = self.trace.last().unwrap();
            display.draw(Line::new(Coord::new(seg.start.x as i32,
                                              seg.start.y as i32),
                                   Coord::new(seg.end.x as i32,
                                              seg.end.y as i32))
                            .with_stroke(Some(GameColor{value: 0xFF_0000}))
                            .with_fill(Some(GameColor{value: 0xFF_0000}))
                            .into_iter() );
        } else {
            display.draw(circle_iter);
        }
    }

    fn update_pos(&mut self, mut new_trace_segment: bool) {
        let speed = self.buffs
                        .iter()
                        .fold(self.speed, |acc, func| (func.change_speed)(acc));
        let mut new_x = (self.pos.x + self.direction.x * speed) as f32;
        let mut new_y = (self.pos.y + self.direction.y * speed) as f32;
        if new_x < PAD_LEFT {
            new_x = WIDTH as f32 - PAD_RIGHT;
            new_trace_segment = true;
        } else if new_x > (WIDTH as f32 - PAD_RIGHT) {
            new_x = PAD_LEFT + 1_f32;
            new_trace_segment = true;
        }
        if new_y < 0.0 {
            new_y = HEIGHT as f32 - 1_f32;
            new_trace_segment = true;
        } else if new_y > HEIGHT as f32 {
            new_trace_segment = true;
            new_y = 0.5;
        }
        self.pos = Vector2D{x: new_x, y: new_y};
        if new_trace_segment {
            self.trace.push(CurveSegment{start: self.pos, end: self.pos,
                                         radius: self.radius});
        } else {
            let mut last = self.trace.last_mut().unwrap();
            last.end = self.pos;
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

    pub fn act(&mut self, touches: &[Coord]) {
        let d = self.buffs.iter().fold(5.0, |acc, func| (func.change_rotation)(acc));
        let a = d * (PI) / 180.0;
        let mut new_trace_segment = false;
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
        let last_seg = self.trace.last().unwrap();
        new_trace_segment &= (last_seg.start - last_seg.end).length() > 2_f32;

        self.update_pos(new_trace_segment);
        self.update_buffs();
    }

    pub fn add_buff(&mut self, buff: PlayerBuff) {
        self.buffs.push(buff);
    }

    pub fn clear_trace(&mut self) {
        self.trace.clear();
        self.trace.push(CurveSegment{start: self.pos, end: self.pos, radius:
                        self.radius});
    }

    fn has_collision(&self, trace: &[CurveSegment]) -> bool {
        // credit to: http://www.sunshine2k.de/coding/java/PointOnLine/PointOnLine.html
        for seg in trace {
            let e1 = seg.start - seg.end;
            let e2 = self.pos - seg.start;
            let val_dp = e1.dot(e2);
            let len2 = e1.dot(e1);
            let proj_p = Vector2D {
                x: seg.start.x + (val_dp * e1.x) / len2,
                y: seg.start.y + (val_dp * e1.y) / len2,
            };

            if val_dp < 0_f32 || val_dp > len2 {
                // projection not on line segment
                let dist_start = self.pos.distance(seg.start);
                let dist_end = self.pos.distance(seg.end);
                let min_dist = dist_end.min(dist_start);
                if min_dist < (self.radius + seg.radius) as f32 {
                    if cfg!(debug_assertions) {println!("collision1");}
                    return true;
                }
            } else if proj_p.distance(self.pos) < (self.radius + seg.radius) as f32 {
                if cfg!(debug_assertions) {
                    println!("collision2 {} {:?} {:?}", proj_p.distance(self.pos), proj_p, seg);}
                return true;
            }
        }
        false
    }
}

impl CollideSelf for Player {
    fn collides(&self) -> bool {
        let take = self.trace.len().checked_sub((self.radius * 2) as usize);
        match take {
            Some(t) => self.has_collision(&self.trace[..t]),
            None => false,
        }
    }
}

impl Collide<Box<Buff>> for Player {
    fn collides_with(&self, incoming: &Box<Buff>) -> bool {
        let b_pos = (*incoming).get_pos();
        let dist_x = libm::fabsf(self.pos.x - b_pos[0] as f32);
        let dist_y = libm::fabsf(self.pos.y - b_pos[1] as f32);
        let dist = libm::sqrtf(dist_x*dist_x + dist_y*dist_y);

        ((self.radius + 10) as f32) >= dist
    }
}

impl Collide<Player> for Player {
    fn collides_with(&self, incoming: &Player) -> bool {
        self.has_collision(&incoming.trace)
    }
}