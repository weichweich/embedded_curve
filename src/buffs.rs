use crate::player::{Curve};
use crate::display::{
    GameColor
};
use embedded_graphics::{
    prelude::*,
    coord::Coord,
};
use crate::geometry::ImgIterator;
use crate::border::Border;

const IMG_FAST: [u8; 10*10*3] = *include_bytes!("fast.data");
const IMG_CLEAR: [u8; 10*10*3] = *include_bytes!("clear.data");
const IMG_CH_DIR: [u8; 10*10*3] = *include_bytes!("dir_change.data");
const IMG_SLOW: [u8; 10*10*3] = *include_bytes!("slow.data");
const IMG_SMALL: [u8; 10*10*3] = *include_bytes!("smaller.data");
const IMG_BIG: [u8; 10*10*3] = *include_bytes!("bigger.data");
const IMG_COLOR: [u8; 10*10*3] = *include_bytes!("color.data");


pub trait Buff {
    fn apply_player(&self, player: &mut Curve);
    fn apply_border(&self, border: &mut Border);
    fn clear_screen(&self) -> bool;
    fn draw(&self) -> ImgIterator;
    fn aabb(&self) -> (Coord, Coord);
    fn get_pos(&self) -> Coord;
}

pub struct PlayerBuff {
    pub timeout: u32,
    pub change_rotation: fn(f32) -> f32,
    pub change_speed: fn(f32) -> f32,
    pub change_color: fn(GameColor) -> GameColor,
    pub change_radius: fn(f32) -> f32,
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
    fn apply_player(&self, player: &mut Curve) {
        fn change_color(color: GameColor) -> GameColor {color}
        fn change_rotation(rotation: f32) -> f32 {rotation}
        fn change_speed(speed: f32) -> f32 {speed + 1.0}
        fn change_radius(r: f32) -> f32 {r}

        player.add_buff(PlayerBuff{
            timeout: 60*30,//30 sec
            change_rotation,
            change_color,
            change_speed,
            change_radius,
        });
    }

    fn apply_border(&self, border: &mut Border) {}

    fn clear_screen(&self) -> bool { false }

    fn draw(&self) -> ImgIterator {
        ImgIterator::new(&IMG_FAST, 10, self.pos)
    }

    fn aabb(&self) -> (Coord, Coord){
        let low_right = Coord::new(self.pos[0]+10, self.pos[1]+10);
        (self.pos, low_right)
    }

    fn get_pos(&self) -> Coord {
        self.pos
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
    fn apply_player(&self, _player: &mut Curve) {
        _player.clear_trace();
    }

    fn apply_border(&self, border: &mut Border) {}

    fn clear_screen(&self) -> bool { true }

    fn draw(&self) -> ImgIterator {
        ImgIterator::new(&IMG_CLEAR, 10, self.pos)
    }

    fn aabb(&self) -> (Coord, Coord){
        let low_right = Coord::new(self.pos[0]+10, self.pos[1]+10);
        (self.pos, low_right)
    }

    fn get_pos(&self) -> Coord {
        self.pos
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

    fn apply_player(&self, player: &mut Curve) {
        fn change_color(color: GameColor) -> GameColor {color}
        fn change_rotation(rotation: f32) -> f32 {360_f32-rotation}
        fn change_speed(speed: f32) -> f32 {speed}
        fn change_radius(r: f32) -> f32 {r}

        player.add_buff(PlayerBuff {
            timeout: 60*10,//10 secs
            change_rotation,
            change_color,
            change_speed,
            change_radius
        });
    }

    fn apply_border(&self, border: &mut Border) {}

    fn clear_screen(&self) -> bool { false }

    fn draw(&self) -> ImgIterator {
        ImgIterator::new(&IMG_CH_DIR, 10, self.pos)
    }

    fn aabb(&self) -> (Coord, Coord){
        let low_right = Coord::new(self.pos[0]+10, self.pos[1]+10);
        (self.pos, low_right)
    }

    fn get_pos(&self) -> Coord {
        self.pos
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
    fn apply_player(&self, player: &mut Curve) {
        fn change_color(color: GameColor) -> GameColor {color}
        fn change_rotation(rotation: f32) -> f32 {rotation}
        fn change_speed(speed: f32) -> f32 {speed - 0.5}
        fn change_radius(r: f32) -> f32 {r}

        player.add_buff(PlayerBuff {
            timeout: 60 * 15,// 5 secs
            change_rotation,
            change_color,
            change_speed,
            change_radius,
        });
    }

    fn apply_border(&self, border: &mut Border) {}

    fn clear_screen(&self) -> bool { false }

    fn draw(&self) -> ImgIterator {
        ImgIterator::new(&IMG_SLOW, 10, self.pos)
    }

    fn aabb(&self) -> (Coord, Coord){
        let low_right = Coord::new(self.pos[0]+10, self.pos[1]+10);
        (self.pos, low_right)
    }

    fn get_pos(&self) -> Coord {
        self.pos
    }
}

// BIG Player Buff

pub struct BigBuff {
    pos: Coord,
}

impl BigBuff {
    pub fn new(pos: Coord) -> Self {
        Self {pos}
    }
}

impl Buff for BigBuff {
    fn apply_player(&self, player: &mut Curve) {
        fn change_color(color: GameColor) -> GameColor {color}
        fn change_rotation(rotation: f32) -> f32 {rotation}
        fn change_speed(speed: f32) -> f32 {speed}
        fn change_radius(r: f32) -> f32 {r * 1.5}

        player.add_buff(PlayerBuff {
            timeout: 60*30,
            change_rotation,
            change_color,
            change_speed,
            change_radius,
        });
    }

    fn apply_border(&self, border: &mut Border) {}

    fn clear_screen(&self) -> bool { false }

    fn draw(&self) -> ImgIterator {
        ImgIterator::new(&IMG_BIG, 10, self.pos)
    }

    fn aabb(&self) -> (Coord, Coord){
        let low_right = Coord::new(self.pos[0]+10, self.pos[1]+10);
        (self.pos, low_right)
    }

    fn get_pos(&self) -> Coord {
        self.pos
    }
}

// Small Player Buff

pub struct SmallBuff {
    pos: Coord,
}

impl SmallBuff {
    pub fn new(pos: Coord) -> Self {
        Self {pos}
    }
}

impl Buff for SmallBuff {
    fn apply_player(&self, player: &mut Curve) {
        fn change_color(color: GameColor) -> GameColor {color}
        fn change_rotation(rotation: f32) -> f32 {rotation}
        fn change_speed(speed: f32) -> f32 {speed}
        fn change_radius(r: f32) -> f32 {r * 0.5}

        player.add_buff(PlayerBuff{
            timeout: 60*30,// 5 secs
            change_rotation,
            change_color,
            change_speed,
            change_radius,
        });
    }

    fn apply_border(&self, border: &mut Border) {}

    fn clear_screen(&self) -> bool { false }

    fn draw(&self) -> ImgIterator {
        ImgIterator::new(&IMG_SMALL, 10, self.pos)
    }

    fn aabb(&self) -> (Coord, Coord){
        let low_right = Coord::new(self.pos[0]+10, self.pos[1]+10);
        (self.pos, low_right)
    }

    fn get_pos(&self) -> Coord {
        self.pos
    }
}

// Color Player Buff

pub struct ColorBuff {
    pos: Coord,
}

impl ColorBuff {
    pub fn new(pos: Coord) -> Self {
        Self {pos}
    }
}

impl Buff for ColorBuff {
    fn apply_player(&self, player: &mut Curve) {
        fn change_color(mut color: GameColor) -> GameColor {
            if color.value == 0xFF_0000 { color.value = 0xFF_FF00; }
            else if color.value == 0xFF_FF00 { color.value = 0x00_00FF; }
            else if color.value == 0x00_00FF { color.value = 0x00_FF00; }
            else { color.value = 0xFF_FF00; }
            return color;
        }
        fn change_rotation(rotation: f32) -> f32 {rotation}
        fn change_speed(speed: f32) -> f32 {speed}
        fn change_radius(r: f32) -> f32 {r}

        player.add_buff(PlayerBuff{
            timeout: 60*60,// 5 secs
            change_rotation,
            change_color,
            change_speed,
            change_radius,
        });
    }

    fn apply_border(&self, border: &mut Border) {}

    fn clear_screen(&self) -> bool { false }

    fn draw(&self) -> ImgIterator {
        ImgIterator::new(&IMG_COLOR, 10, self.pos)
    }

    fn aabb(&self) -> (Coord, Coord){
        let low_right = Coord::new(self.pos[0]+10, self.pos[1]+10);
        (self.pos, low_right)
    }

    fn get_pos(&self) -> Coord {
        self.pos
    }
}

// Border Player Buff

pub struct BorderBuff {
    pos: Coord,
}

impl BorderBuff {
    pub fn new(pos: Coord) -> Self {
        Self {pos}
    }
}

impl Buff for BorderBuff {
    fn apply_player(&self, player: &mut Curve) {
        fn change_color(color: GameColor) -> GameColor {color}
        fn change_rotation(rotation: f32) -> f32 {rotation}
        fn change_speed(speed: f32) -> f32 {speed}
        fn change_radius(r: f32) -> f32 {r}

        player.add_buff(PlayerBuff{
            timeout: 60*30,// 5 secs
            change_rotation,
            change_color,
            change_speed,
            change_radius,
        });
    }
 
    fn apply_border(&self, border: &mut Border) {
        border.active = !border.active;
    }

    fn clear_screen(&self) -> bool { false }

    fn draw(&self) -> ImgIterator {
        ImgIterator::new(&IMG_SMALL, 10, self.pos)
    }

    fn aabb(&self) -> (Coord, Coord){
        let low_right = Coord::new(self.pos[0]+10, self.pos[1]+10);
        (self.pos, low_right)
    }

    fn get_pos(&self) -> Coord {
        self.pos
    }
}

