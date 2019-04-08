use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, Line};
use stm32f7_discovery::lcd::{Framebuffer, HEIGHT, WIDTH};
use core::f32::consts::PI;
use alloc::vec::Vec;
use nalgebra::{Vector2, dot, normalize};
use libm;

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
            
        // display.draw(circle_iter);
        playing_field.store(circle_iter, self.id);

        let n = self.trace.len();
        display.draw(Line::new(Coord::new(self.trace[n-2].0 as i32, self.trace[n-2].1 as i32),
                            Coord::new(self.trace[n-1].0 as i32, self.trace[n-1].1 as i32))
                            .with_stroke(Some(GameColor{value: 0xFF0000}))
                            .with_fill(Some(GameColor{value: 0xFF0000}))
                            .into_iter() );
    }

    fn update_pos(&mut self, mut new_trace_segment: bool) {
        let speed = self.buffs
                        .iter()
                        .fold(self.speed, |acc, func| (func.change_speed)(acc));
        let mut new_x = (self.pos.0 + self.direction.x * speed) as f32;
        let mut new_y = (self.pos.1 + self.direction.y * speed) as f32;
        
        if new_x < 0.0 {
            new_x = WIDTH as f32 - 1.0;
            new_trace_segment = true;
        } else if new_x > WIDTH as f32 {
            new_x = 0.5;
            new_trace_segment = true;
        }
        if new_y < 0.0 {
            new_y = HEIGHT as f32 - 1.0;
            new_trace_segment = true;
        } else if new_y > HEIGHT as f32 {
            new_trace_segment = true;
            new_y = 0.5;
        }
        self.pos = (new_x, new_y);
        self.update_trace(new_trace_segment);
    }

    fn update_trace(&mut self, new_trace_segment: bool) {
        let tracepoint = (self.pos.0, self.pos.1, self.radius);
        if new_trace_segment {
            self.trace.push( tracepoint );
            self.trace.push( tracepoint );
        } else {
            let n = self.trace.len();
            self.trace[n-1] = tracepoint;
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

    pub fn clear_trace(&mut self) {
        let tracepoint : (f32, f32, u32) = (self.pos.0, self.pos.1, self.radius);
        self.trace.clear();
        self.trace.push( tracepoint );
        self.trace.push( tracepoint );
    }
}

impl CollideSelf for Player {
    fn collides(&self) -> bool {
        // collides_with(&self)
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
        let trace1 = self.trace.last().unwrap();
        // let trace2 = incoming.trace.last().unwrap();

        let p1_pos = Vector2::new(trace1.0, trace1.1);
        // let p2_pos = (trace2.0, trace2.1);

        let p1_radius = trace1.2;
        // let p2_radius = trace2.2;
        let n = incoming.trace.len();
        for (ti, tj) in incoming.trace[1..].iter().zip(incoming.trace[..n-1].iter()) {
            let mut p2_radius = ti.2;
            let p2_i = Vector2::new(ti.0, ti.1);
            let p2_j = Vector2::new(tj.0, tj.1);


            if p2_i == p2_j {
                
                let p2_p1 = p1_pos - p2_j;
                let distance = p2_p1.dot(&p2_p1); //dir.dot(&dir);
                p2_radius = if p2_radius > tj.2 { p2_radius } else { tj.2 };
                let radius = if p1_radius > p2_radius { p1_radius } else { p2_radius };
                if distance <= radius as f32 {
                    println!("collision P1");
                    return true;

                }
            }  else {               //Segment has a length > 0
                let mut axis = p2_j - p2_i;
                let axis_length = axis.norm();
                axis = axis.normalize();

                let normal = Vector2::new(-axis.y, axis.x).normalize();
                let p2_p1 = p1_pos - p2_j ;
                let distance = libm::fabsf(libm::sqrtf(p2_p1.dot(&normal)));
                if distance <= p2_radius as f32 {
                    println!("collision ? : {} {} ", distance, p2_radius);
                    
                    let p1_projected = libm::sqrtf(axis.dot(&p2_p1))*axis;
                    let p1_projected_distance = p1_projected.norm();
                    println!("{} {} {} {} {}", axis_length, axis.norm(), p1_projected_distance, p2_p1.x, p2_p1.y);

                    // if p1_projected.dot(&axis) >= 0.0 && p1_projected.dot(&(-axis)) >= 0.0 {
                    if  p1_projected_distance >= -(p2_radius as f32) &&
                        p1_projected_distance <= axis_length + p2_radius as f32 {
                        println!("collision P2");
                        return true;
                    }
                }
            }
        }
        return false;
    }
}
