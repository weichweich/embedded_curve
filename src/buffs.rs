use crate::player::{Player};
use crate::display::{
    GameColor
};
use embedded_graphics::{
    coord::Coord,
};
use crate::geometry::ImgIterator;


const IMG_FAST :[u8; 10*10*3] = *include_bytes!("fast.data");
const IMG_CLEAR :[u8; 10*10*3] = *include_bytes!("clear.data");
const IMG_CH_DIR :[u8; 10*10*3] = *include_bytes!("dir_change.data");
const IMG_SLOW :[u8; 10*10*3] = *include_bytes!("slow.data");


pub trait Buff {
    fn apply_player(&self, player: &mut Player);
    fn clear_screen(&self) -> bool;
    fn draw(&self) -> ImgIterator;
}

pub struct PlayerBuff {
    pub timeout: u32,
    pub change_rotation: fn(f32) -> f32,
    pub change_speed: fn(f32) -> f32,
    pub change_color: fn(GameColor) -> GameColor,
}

// Fast Buff

pub struct FastPlayerBuffSprite {
    pos: Coord,
}

impl FastPlayerBuffSprite {
    pub fn new(pos: Coord) -> Self {
        FastPlayerBuffSprite {
            pos,
        }
    }
}

impl Buff for FastPlayerBuffSprite {
    fn apply_player(&self, player: &mut Player) {
        fn change_color(color: GameColor) -> GameColor {color}
        fn change_rotation(rotation: f32) -> f32 {rotation}
        fn change_speed(speed: f32) -> f32 {speed + 1.0}

        player.add_buff(PlayerBuff{
            timeout: 60*30,//30 sec
            change_rotation,
            change_color,
            change_speed,
        });
    }

    fn clear_screen(&self) -> bool { false }

    fn draw(&self) -> ImgIterator {
        ImgIterator::new(&IMG_FAST, 10, self.pos)
    }
}

// Clear Buff

pub struct ClearBuff {
    pos: Coord,
}

impl ClearBuff {
    pub fn new(pos: Coord) -> Self {
        Self {
            pos,
        }
    }

}

impl Buff for ClearBuff {
    fn apply_player(&self, _player: &mut Player) {}

    fn clear_screen(&self) -> bool { true }

    fn draw(&self) -> ImgIterator {
        ImgIterator::new(&IMG_CLEAR, 10, self.pos)
    }
}

// Change direction Buff

pub struct ChangeDirBuff {
    pos: Coord,
}

impl ChangeDirBuff {
    pub fn new(pos: Coord) -> Self {
        Self {pos}
    }
}

impl Buff for ChangeDirBuff {

    fn apply_player(&self, player: &mut Player) {
        fn change_color(color: GameColor) -> GameColor {color}
        fn change_rotation(rotation: f32) -> f32 {360_f32-rotation}
        fn change_speed(speed: f32) -> f32 {speed}

        player.add_buff(PlayerBuff{
            timeout: 60*10,//10 secs
            change_rotation,
            change_color,
            change_speed,
        });
    }

    fn clear_screen(&self) -> bool { false }

    fn draw(&self) -> ImgIterator {
        ImgIterator::new(&IMG_CH_DIR, 10, self.pos)
    }
}

// Slow Player Buff

pub struct SlowBuff {
    pos: Coord,
}

impl SlowBuff {
    pub fn new(pos: Coord) -> Self {
        Self {pos}
    }
}

impl Buff for SlowBuff {
    fn apply_player(&self, player: &mut Player) {
        fn change_color(color: GameColor) -> GameColor {color}
        fn change_rotation(rotation: f32) -> f32 {rotation}
        fn change_speed(speed: f32) -> f32 {speed - 0.5}

        player.add_buff(PlayerBuff{
            timeout: 60*5,// 5 secs
            change_rotation,
            change_color,
            change_speed,
        });
    }

    fn clear_screen(&self) -> bool { false }

    fn draw(&self) -> ImgIterator {
        ImgIterator::new(&IMG_SLOW, 10, self.pos)
    }
}
