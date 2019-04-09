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
    primitives::{Rect}
};

use crate::{
    display::GameColor,
    player::{PAD_LEFT, PAD_RIGHT},
    buffs::{
        Buff, BigBuff, SmallBuff, FastPlayerBuffSprite, SlowBuff, ChangeDirBuff,
        ClearBuff, ColorBuff
    },
    BOTTOM_MID, LEFT_MID, TOP_LEFT, RIGHT_MID, TOP_MID, BOTTOM_RIGHT, MID_MID,
    get_rand_num, C_PLAYER_A, C_PLAYER_B,
    geometry::AABBox,
    player::{Player, Collide, CollideSelf},
};

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
}

impl Game {

    pub fn new(player_count: u32, rng: &mut Rng) -> Self {
        let pos_a = (
            PAD_LEFT + get_rand_num(rng) as f32 % (WIDTH as f32 - PAD_LEFT - PAD_RIGHT),
            get_rand_num(rng) as f32 % HEIGHT as f32,
        );
        let pos_b = (
            PAD_LEFT + get_rand_num(rng) as f32 % (WIDTH as f32 - PAD_LEFT - PAD_RIGHT),
            get_rand_num(rng) as f32 % HEIGHT as f32,
        );
        let angle_a = get_rand_num(rng) as f32 % 360_f32;
        let angle_b = get_rand_num(rng) as f32 % 360_f32;
        fn to_coord(t:(i32, i32)) -> Coord {Coord::new(t.0, t.1)};
        let player_a = Player::new(
            AABBox::new(to_coord(MID_MID), to_coord(BOTTOM_RIGHT)),
            AABBox::new(to_coord(TOP_MID), to_coord(RIGHT_MID)),
            C_PLAYER_A,
            pos_a,
            2,
            angle_a,
        );
        let player_b = Player::new(
            AABBox::new(to_coord(TOP_LEFT), to_coord(MID_MID)),
            AABBox::new(to_coord(LEFT_MID), to_coord(BOTTOM_MID)),
            C_PLAYER_B,
            pos_b,
            2,
            angle_b,
        );
        let buffs: Vec<Box<Buff>> = Vec::new();
        let mut players: Vec<Player> = Vec::new();
        players.push(player_a);
        players.push(player_b);
        Self {
            players,
            buffs,
            tt_update: 0,
            last_time_update: 0,
            tt_new_buff: 0,
            state: GameState::Playing,
        }
    }

    pub fn new_game(&mut self) {
        self.tt_update = 0;
        self.last_time_update = 0;
        self.tt_new_buff = 0;
        for p in &mut self.players {
            p.clear_trace();
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

            if pi.collides() {
                if cfg!(debug_assertions) {println!("self collision {}", i);}
                return Some(i);
            } else  { 
                for (h, pj) in pjs.iter().enumerate() {
                    if pi.collides_with(pj) {
                        if cfg!(debug_assertions) {println!("collision i {}", i);}
                        return Some(i)
                    } else if pj.collides_with(pi) {
                        if cfg!(debug_assertions) {println!("collision j {}", h+i+1);}
                        return Some(h+i+1)
                    }
                }
            }
        }
        None
    }

    fn player_buff_collision<D>(&mut self, display: &mut D)
    where D: Drawing<GameColor> {
        let mut clear_all = false;
        for p in &mut self.players {
            let mut i: usize = 0;
            while i < self.buffs.len() {
                if p.collides_with(&self.buffs[i]) {
                    self.buffs[i].apply_player(p);
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

        self.tt_update -= dt as isize;
        if self.tt_update < 0 {
            self.tt_update = 3;

            self.act(touches, dt);
            self.player_buff_collision(display);
            if let Some(i) = self.player_player_collision() {
                self.player_lost(i);
                self.state = GameState::Finished;
            }
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

fn new_rand_buff(rng: &mut Rng) -> Box<Buff + 'static> {
    let pos_buff = (
        (PAD_LEFT + get_rand_num(rng) as f32 
            % (WIDTH as f32 - PAD_LEFT - PAD_RIGHT)) as i32,
        (get_rand_num(rng) as f32 % HEIGHT as f32) as i32,
    );
    let rand = get_rand_num(rng);
    match (rand % 7) +3 {
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
