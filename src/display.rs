use embedded_graphics::{
    Drawing,
    drawable::Pixel
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
}

impl <'a, F: Framebuffer> Drawing<u8> for LcdDisplay<'a, F> {

    fn draw<T>(&mut self, item_pixels: T)
    where
        T: Iterator<Item = Pixel<u8>>
    {
        for Pixel(coord, color) in item_pixels {
            if coord.0 as usize >= WIDTH && coord.1 as usize >= HEIGHT {
                continue;
            }
            self.layer.print_point_color_at(coord.0 as usize, coord.1 as usize, 
                                            Color::from_hex(0xFF0000));
        }
    }
}
