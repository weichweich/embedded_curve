use alloc::vec::Vec;
use embedded_graphics::{
    Drawing,
    drawable::Pixel,
    pixelcolor::PixelColor,
    coord::Coord,
};

use stm32f7_discovery::lcd::{Framebuffer, Layer, Color, WIDTH, HEIGHT};

pub struct LcdDisplay<'a, F: Framebuffer> {
    layer: &'a mut Layer<F>
}

impl <'a, F: Framebuffer> LcdDisplay<'a, F> {
    pub fn new(layer: &'a mut Layer<F>) -> Self {
        Self {
            layer
        }
    }

    pub fn draw_bmp_rgb8(&mut self, pos: Coord, width: u32, height: u32,
                         data: &[u8]) {
        let mut r = None;
        let mut g = None;
        let mut x:i32 = 0;
        let mut y:i32 = 0;
        for c in data {
            match (r, g) {
                (None, _) => r = Some(*c),
                (Some(_), None) => g = Some(*c),
                (Some(ri), Some(gi)) => {
                    let color = u32::from(ri).rotate_left(16) 
                                | u32::from(gi).rotate_left(8)
                                | u32::from(*c);
                    self.layer.print_point_color_at(
                        (pos[0]+x) as usize, (pos[1]+y)as usize, 
                        Color::from_rgb888(color));
                    x += 1;
                    if x >= width as i32 {
                        x = 0;
                        y += 1;
                    }
                    r = None;
                    g = None;
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.layer.clear();
    }
}

impl <'a, F: Framebuffer> Drawing<GameColor> for LcdDisplay<'a, F> {

    fn draw<T>(&mut self, item_pixels: T)
    where
        T: Iterator<Item = Pixel<GameColor>>
    {
        for Pixel(coord, color) in item_pixels {
            if coord.0 as usize >= WIDTH || coord.1 as usize >= HEIGHT {
                continue;
            }
            self.layer.print_point_color_at(coord.0 as usize, coord.1 as usize, 
                                            Color::from_hex(color.value));
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct GameColor {
    pub value: u32,
}

impl PixelColor for GameColor {}

impl From<u8> for GameColor {
    fn from(other: u8) -> Self {
        GameColor {
            value: other as u32,
        }
    }
}