use crate::player::Player;
use crate::display::{
    GameColor, LcdDisplay
};
use stm32f7_discovery::lcd::Framebuffer;
use embedded_graphics::coord::Coord;


const IMG_FAST :[u8; 10*10*3] = *include_bytes!("fast.data");
const IMG_CLEAR :[u8; 10*10*3] = *include_bytes!("clear.data");
const IMG_CH_DIR :[u8; 10*10*3] = *include_bytes!("dir_change.data");
const IMG_SLOW :[u8; 10*10*3] = *include_bytes!("slow.data");


pub trait Buff {
    fn apply_player(&self, player: &mut Player);
    fn apply_display<F: Framebuffer>(&self, display: &mut LcdDisplay<F>);
    fn draw<F: Framebuffer>(&self, display: &mut LcdDisplay<F>);
}

pub struct PlayerBuff {
    pub timeout: u32,
    pub change_rotation: fn(f32) -> f32,
    pub change_speed: fn(f32) -> f32,
    pub change_color: fn(GameColor) -> GameColor,
}

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

    fn apply_display<F: Framebuffer>(&self, display: &mut LcdDisplay<F>) {
    }

    fn draw<F: Framebuffer>(&self, display: &mut LcdDisplay<F>) {
        display.draw_bmp_rgb8(self.pos, 10, 10, &IMG_FAST);
    }
}

pub struct FastPlayerBuff {}

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
    fn apply_player(&self, player: &mut Player) {}

    fn apply_display<F: Framebuffer>(&self, display: &mut LcdDisplay<F>) {
        display.clear();
    }

    fn draw<F: Framebuffer>(&self, display: &mut LcdDisplay<F>) {
        display.draw_bmp_rgb8(self.pos, 10, 10, &IMG_CLEAR);
    }
}

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

    fn apply_display<F: Framebuffer>(&self, display: &mut LcdDisplay<F>) {}

    fn draw<F: Framebuffer>(&self, display: &mut LcdDisplay<F>) {
        display.draw_bmp_rgb8(self.pos, 10, 10, &IMG_CH_DIR);
    }
}

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

    fn apply_display<F: Framebuffer>(&self, display: &mut LcdDisplay<F>) {}

    fn draw<F: Framebuffer>(&self, display: &mut LcdDisplay<F>) {
        display.draw_bmp_rgb8(self.pos, 10, 10, &IMG_SLOW);
    }
}
