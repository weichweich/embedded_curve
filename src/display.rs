use embedded_graphics::{
    Drawing,
    drawable::Pixel,
    pixelcolor::PixelColor,
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

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct GameColor {
    pub value: u32,
}

impl PixelColor for GameColor {}

impl From<u8> for GameColor {
    fn from(other: u8) -> Self {
        GameColor {
            value: u32::from(other),
        }
    }
}