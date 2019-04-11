use crate::player::{Curve};
use crate::display::{
    GameColor
};
use embedded_graphics::{
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
const IMG_BORDER: [u8; 10*10*3] = *include_bytes!("border.data");


pub trait Buff {
    fn apply_player(&self, _player: &mut Curve, _collector: bool) {}
    fn apply_border(&self, _border: &mut Border) {}
    fn clear_screen(&self) -> bool { false }
    fn draw(&self) -> ImgIterator;
    fn aabb(&self) -> (Coord, Coord);
    fn get_pos(&self) -> Coord;
}

pub struct PlayerBuff {
    pub timeout: u32,
    pub change_rotation: fn(u32, f32) -> f32,
    pub change_speed: fn(u32, f32) -> f32,
    pub change_color: fn(u32, GameColor) -> GameColor,
    pub change_radius: fn(u32, f32) -> f32,
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
    fn apply_player(&self, player: &mut Curve, collector: bool) {
        if collector {
            fn change_color(_time: u32, color: GameColor) -> GameColor {color}
            fn change_rotation(_time: u32, rotation: f32) -> f32 {rotation}
            fn change_speed(_time: u32, speed: f32) -> f32 {speed + 1.0}
            fn change_radius(_time: u32, r: f32) -> f32 {r}

            player.add_buff(PlayerBuff{
                timeout: 60*30,//30 sec
                change_rotation,
                change_color,
                change_speed,
                change_radius,
            });
        }
    }

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

pub struct ClearBuffSprite {
    pos: Coord,
}

impl ClearBuffSprite {
    pub fn new(pos: Coord) -> Self {
        ClearBuffSprite {
            pos,
        }
    }

}

impl Buff for ClearBuffSprite {
    fn apply_player(&self, player: &mut Curve, _collector: bool) {
        player.clear_trace();
    }

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

pub struct ChangeDirBuffSprite {
    pos: Coord,
}

impl ChangeDirBuffSprite {
    pub fn new(pos: Coord) -> Self {
        ChangeDirBuffSprite {pos}
    }
}

impl Buff for ChangeDirBuffSprite {

    fn apply_player(&self, player: &mut Curve, collector: bool) {
        if !collector {
            fn change_color(_time: u32, color: GameColor) -> GameColor {color}
            fn change_rotation(_time: u32, rotation: f32) -> f32 {360_f32-rotation}
            fn change_speed(_time: u32, speed: f32) -> f32 {speed}
            fn change_radius(_time: u32, r: f32) -> f32 {r}

            player.add_buff(PlayerBuff {
                timeout: 100*5,
                change_rotation,
                change_color,
                change_speed,
                change_radius
            });
        }
    }

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

pub struct SlowBuffSprite {
    pos: Coord,
}

impl SlowBuffSprite {
    pub fn new(pos: Coord) -> Self {
        SlowBuffSprite {pos}
    }
}

impl Buff for SlowBuffSprite {
    fn apply_player(&self, player: &mut Curve, collector: bool) {
        if collector {
            fn change_color(_time: u32, color: GameColor) -> GameColor {color}
            fn change_rotation(_time: u32, rotation: f32) -> f32 {rotation}
            fn change_speed(_time: u32, speed: f32) -> f32 {speed * 0.5}
            fn change_radius(_time: u32, r: f32) -> f32 {r}

            player.add_buff(PlayerBuff {
                timeout: 100*10,
                change_rotation,
                change_color,
                change_speed,
                change_radius,
            });
        }
    }

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

pub struct BigBuffSprite {
    pos: Coord,
}

impl BigBuffSprite {
    pub fn new(pos: Coord) -> Self {
        BigBuffSprite {pos}
    }
}

impl Buff for BigBuffSprite {
    fn apply_player(&self, player: &mut Curve, collector: bool) {
        if collector {
            fn change_color(_time: u32, color: GameColor) -> GameColor {color}
            fn change_rotation(_time: u32, rotation: f32) -> f32 {rotation}
            fn change_speed(_time: u32, speed: f32) -> f32 {speed}
            fn change_radius(_time: u32, r: f32) -> f32 {r * 1.5}

            player.add_buff(PlayerBuff {
                timeout: 60*30,
                change_rotation,
                change_color,
                change_speed,
                change_radius,
            });
        }
    }

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

pub struct SmallBuffSprite {
    pos: Coord,
}

impl SmallBuffSprite {
    pub fn new(pos: Coord) -> Self {
        SmallBuffSprite {pos}
    }
}

impl Buff for SmallBuffSprite {
    fn apply_player(&self, player: &mut Curve, collector: bool) {
        if collector {
            fn change_color(_time: u32, color: GameColor) -> GameColor {color}
            fn change_rotation(_time: u32, rotation: f32) -> f32 {rotation}
            fn change_speed(_time: u32, speed: f32) -> f32 {speed}
            fn change_radius(_time: u32, r: f32) -> f32 {r * 0.5}

            player.add_buff(PlayerBuff{
                timeout: 60*30,// 5 secs
                change_rotation,
                change_color,
                change_speed,
                change_radius,
            });
        }
    }

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

pub struct ColorBuffSprite {
    pos: Coord,
}

impl ColorBuffSprite {
    pub fn new(pos: Coord) -> Self {
        ColorBuffSprite {pos}
    }
}

impl Buff for ColorBuffSprite {
    fn apply_player(&self, player: &mut Curve, _collector: bool) {
        fn change_color(_time: u32, mut color: GameColor) -> GameColor {
            if color.value == 0xFF_0000 { color.value = 0xFF_FF00; }
            else if color.value == 0xFF_FF00 { color.value = 0x00_00FF; }
            else if color.value == 0x00_00FF { color.value = 0x00_FF00; }
            else { color.value = 0xFF_FF00; }
            color
        }
        fn change_rotation(_time: u32, rotation: f32) -> f32 {rotation}
        fn change_speed(_time: u32, speed: f32) -> f32 {speed}
        fn change_radius(_time: u32, r: f32) -> f32 {r}

        player.add_buff(PlayerBuff{
            timeout: 60*60,// 5 secs
            change_rotation,
            change_color,
            change_speed,
            change_radius,
        });
    }

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

pub struct BorderBuffSprite {
    pos: Coord,
}

impl BorderBuffSprite {
    pub fn new(pos: Coord) -> Self {
        BorderBuffSprite {pos}
    }
}

impl Buff for BorderBuffSprite {

    fn apply_border(&self, border: &mut Border) {
        border.active = !border.active;
        border.drawn = false;
    }

    fn draw(&self) -> ImgIterator {
        ImgIterator::new(&IMG_BORDER, 10, self.pos)
    }

    fn aabb(&self) -> (Coord, Coord){
        let low_right = Coord::new(self.pos[0]+10, self.pos[1]+10);
        (self.pos, low_right)
    }

    fn get_pos(&self) -> Coord {
        self.pos
    }
}

// Drunken Player Buff

pub struct DrunkenBuffSprite {
    pos: Coord,
}

impl DrunkenBuffSprite {
    pub fn new(pos: Coord) -> Self {
        DrunkenBuffSprite {pos}
    }
}

impl Buff for DrunkenBuffSprite {
    fn apply_player(&self, player: &mut Curve, collector: bool) {
        if !collector {
            fn change_color(_time: u32, color: GameColor) -> GameColor {color}
            fn change_rotation(time: u32, rotation: f32) -> f32 {
                rotation + 5_f32 - (time as f32 % 10_f32)
            }
            fn change_speed(_time: u32, speed: f32) -> f32 {speed}
            fn change_radius(_time: u32, r: f32) -> f32 {r}

            player.add_buff(PlayerBuff{
                timeout: 60*10,// 5 secs
                change_rotation,
                change_color,
                change_speed,
                change_radius,
            });
        }
    }

    fn draw(&self) -> ImgIterator {
        ImgIterator::new(&IMG_BORDER, 10, self.pos)
    }

    fn aabb(&self) -> (Coord, Coord){
        let low_right = Coord::new(self.pos[0]+10, self.pos[1]+10);
        (self.pos, low_right)
    }

    fn get_pos(&self) -> Coord {
        self.pos
    }
}

