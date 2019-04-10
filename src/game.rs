use alloc::{
    vec::Vec,
    boxed::Box,
};
use stm32f7_discovery::{
    lcd::{HEIGHT, WIDTH},
    random::Rng,
};
use embedded_graphics::{
    prelude::*,
    primitives::Rect,
};

use crate::{
    display::GameColor,
    player::{PAD_LEFT, PAD_RIGHT},
    buffs::{
        Buff, BigBuff, SmallBuff, FastPlayerBuffSprite, SlowBuff, ChangeDirBuff,
        ClearBuff, ColorBuff
    },
    BOTTOM_MID, LEFT_MID, TOP_LEFT, RIGHT_MID, TOP_MID, BOTTOM_RIGHT, MID_MID,
    get_rand_num, to_coord,
    geometry::AABBox,
    border::Border,
    player::{Curve, Collide, CollideSelf, PlayerInput},
};

const CURVE_RADIUS: u32 = 3;

pub struct InputRegion {
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

pub struct Player {
    pub score: u32,
    curve: Curve,
    color: GameColor,
    input_left: InputRegion,
    input_right: InputRegion,
}

impl Player {
    pub fn new(color: GameColor, rng: &mut Rng, input_left: AABBox,
               input_right: AABBox) -> Self {
        Self {
            score: 0,
            color,
            curve: Curve::new(color, rand_pos(rng), CURVE_RADIUS,
                              (get_rand_num(rng) % 360) as f32),
            input_left: InputRegion::new(input_left),
            input_right: InputRegion::new(input_right),
        }
    }

    pub fn clear_trace(&mut self) {
        self.curve.clear_trace();
    }

    pub fn reset(&mut self, rng: &mut Rng) {
        self.curve = Curve::new(self.color, rand_pos(rng), CURVE_RADIUS,
                                (get_rand_num(rng) % 360) as f32);
    }

    pub fn act(&mut self, touches: &[Coord]) {
        match (self.input_left.is_active(touches),
               self.input_right.is_active(touches)) {
            (true, true) => self.curve.act(PlayerInput::Both),
            (false, false) => self.curve.act(PlayerInput::None),
            (true, false) => self.curve.act(PlayerInput::Left),
            (false, true) => self.curve.act(PlayerInput::Right),
        }
    }

    pub fn draw<D: Drawing<GameColor>>(&self, display: &mut D) {
        self.curve.draw(display);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GameState {
    Playing,
    Finished,
}


pub struct Game {
    pub players: Vec<Player>,
    buffs: Vec<Box<Buff>>,
    tt_update: isize,
    tt_new_buff: isize,
    last_time_update: isize,
    state: GameState,
    border: Border,
}

impl Game {

    pub fn new(player_colors: &[GameColor], rng: &mut Rng) -> Self {
        let buffs: Vec<Box<Buff>> = Vec::new();
        let mut players: Vec<Player> = Vec::new();
        for (i, c) in player_colors.iter().enumerate() {
            match i % 2 {
                0 => players.push(Player::new(*c, rng,
                    AABBox::new(to_coord(MID_MID), to_coord(BOTTOM_RIGHT)),
                    AABBox::new(to_coord(TOP_MID), to_coord(RIGHT_MID)),
                )),
                1 => players.push(Player::new(*c, rng,
                    AABBox::new(to_coord(TOP_LEFT), to_coord(MID_MID)),
                    AABBox::new(to_coord(LEFT_MID), to_coord(BOTTOM_MID)),
                )),
                _ => {},
            }
        }
        Self {
            players,
            buffs,
            tt_update: 0,
            last_time_update: 0,
            tt_new_buff: 0,
            state: GameState::Playing,
            border: Border::new(),
        }
    }

    pub fn new_game(&mut self, rng: &mut Rng) {
        self.tt_update = 0;
        self.last_time_update = 0;
        self.tt_new_buff = 0;
        for p in &mut self.players {
            p.reset(rng);
        }
        self.buffs.clear();
        self.state = GameState::Playing;
    }

    fn update_buffs(&mut self, rng: &mut Rng, dt: usize) {
        self.tt_new_buff -= dt as isize;

        if self.tt_new_buff < 0 {
            self.tt_new_buff = (get_rand_num(rng) % (100 * 60)) as isize;
            self.buffs.push(new_rand_buff(rng));
        }
    }

    fn act(&mut self, touches: &[Coord], _dt:usize) {
        for p in &mut self.players {
            p.act(touches);
        }
    }

    fn player_player_collision(&self) -> Option<usize> {
        for i in 0..self.players.len() {
            let (pis, pjs) = self.players.split_at(i+1);
            let pi = pis.last().unwrap();

            if pi.curve.collides() {
                if cfg!(debug_assertions) {println!("self collision {}", i);}
                return Some(i);
            } else  { 
                for (h, pj) in pjs.iter().enumerate() {
                    if pi.curve.collides_with(&pj.curve) {
                        if cfg!(debug_assertions) {println!("collision i {}", i);}
                        return Some(i)
                    } else if pj.curve.collides_with(&pi.curve) {
                        if cfg!(debug_assertions) {println!("collision j {}", h+i+1);}
                        return Some(h+i+1)
                    }
                }
            }
        }
        None
    }

    fn player_border_collision(&mut self) {
        let mut loosers = Vec::new();
        for (i, p) in self.players.iter().enumerate() {
            if p.collides_with(&(self.border) ) { 
                loosers.push(i);
            }
        }
        for looser in loosers {
            self.player_lost(looser);
            self.state = GameState::Finished;
        }
    }

    fn player_buff_collision<D>(&mut self, display: &mut D)
    where D: Drawing<GameColor> {
        let mut clear_all = false;
        for p in &mut self.players {
            let mut i: usize = 0;
            while i < self.buffs.len() {
                if p.curve.collides_with(&self.buffs[i]) {
                    self.buffs[i].apply_player(&mut p.curve);
                    clear_all |= self.buffs[i].clear_screen();
                    let aabb = self.buffs[i].aabb();
                    display.draw(Rect::new(aabb.0, aabb.1)
                                    .with_fill(Some(GameColor{value: 0x00_0000}))
                                    .into_iter());
                    self.buffs.remove(i);
                } else {
                    i += 1;
                }
            }
        }
        if clear_all {
            // display.clear();
            // TODO: clear players
        }
    }

    fn player_lost(&mut self, loser_i: usize) {
        for (j, p) in self.players.iter_mut().enumerate() {
            if loser_i != j {p.score += 1;}
        }
    }

    pub fn step<D>(&mut self, rng: &mut Rng, display: &mut D, touches: &[Coord], dt: usize) -> GameState
    where D: Drawing<GameColor> {
        match self.state {
            GameState::Playing => {},
            GameState::Finished => return GameState::Finished,
        }

        self.update_buffs(rng, dt);
        self.border.draw(display);

        self.tt_update -= dt as isize;
        if self.tt_update < 0 {
            self.tt_update = 3;

            self.act(touches, dt);
            self.player_buff_collision(display);
            if let Some(i) = self.player_player_collision() {
                self.player_lost(i);
                self.state = GameState::Finished;
            }
            self.player_border_collision();
            for p in &mut self.buffs {
                display.draw(p.draw());
            }
            for p in &mut self.players {
                p.draw(display);
            }
        }
        self.state
    }
}

fn rand_pos(rng: &mut Rng) -> (f32, f32) {
    (
        PAD_LEFT + get_rand_num(rng) as f32 % (WIDTH as f32 - PAD_LEFT - PAD_RIGHT),
        get_rand_num(rng) as f32 % HEIGHT as f32,
    )
}

fn new_rand_buff(rng: &mut Rng) -> Box<Buff + 'static> {
    let pos_buff = (
        (PAD_LEFT + get_rand_num(rng) as f32 
            % (WIDTH as f32 - PAD_LEFT - PAD_RIGHT)) as i32,
        (get_rand_num(rng) as f32 % HEIGHT as f32) as i32,
    );
    let rand = get_rand_num(rng);
    match 4  { //(rand % 7)
        0 => Box::new(FastPlayerBuffSprite::new(Coord::new(pos_buff.0, pos_buff.1))),
        1 => Box::new(ClearBuff::new(Coord::new(pos_buff.0, pos_buff.1))),
        2 => Box::new(ChangeDirBuff::new(Coord::new(pos_buff.0, pos_buff.1))),
        3 => Box::new(SlowBuff::new(Coord::new(pos_buff.0, pos_buff.1))),
        4 => Box::new(ColorBuff::new(Coord::new(pos_buff.0, pos_buff.1))),
        5 => Box::new(BigBuff::new(Coord::new(pos_buff.0, pos_buff.1))),
        6 => Box::new(SmallBuff::new(Coord::new(pos_buff.0, pos_buff.1))),
        _ => Box::new(SlowBuff::new(Coord::new(pos_buff.0, pos_buff.1))),
    }
}
