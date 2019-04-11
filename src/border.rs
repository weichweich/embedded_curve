use stm32f7_discovery::{
    lcd::{HEIGHT, WIDTH},
};

use embedded_graphics::{
    prelude::*,
    coord::Coord,
    primitives::Rect,
};

use crate::display::GameColor;

use crate::player::{PAD_LEFT, PAD_RIGHT, PAD_BOTTOM, PAD_TOP};

pub struct Border {
    pub top_left : Coord,
    pub bottom_right : Coord,
    pub active : bool,
    pub drawn : bool,
}

impl Border {
    pub fn new() -> Self {
        Border {
            top_left : Coord::new(PAD_LEFT as i32, PAD_TOP as i32),
            bottom_right : Coord::new((WIDTH-1) as i32 - PAD_RIGHT as i32, 
                                    (HEIGHT-1) as i32 - PAD_BOTTOM as i32),
            active : false,
            drawn : false,
        }
    }
 
    pub fn draw<D: Drawing<GameColor>>(&mut self, display: &mut D){
        if self.drawn { return; }
        if self.active { 
            display.draw(Rect::new(self.top_left, self.bottom_right)
                .with_stroke(Some(GameColor{value: 0xFF_FFFF}))
                .with_stroke_width(1)
                .into_iter() );
        } else {
            display.draw(Rect::new(self.top_left, self.bottom_right)
                .with_stroke(Some(GameColor{value: 0x00_0000}))
                .with_stroke_width(1)
                .into_iter() );
        }
        self.drawn = true;
    }
}
