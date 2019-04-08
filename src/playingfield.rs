extern crate alloc;

use alloc::vec::Vec;
use stm32f7_discovery::{
    // system_clock::self,
    lcd::{HEIGHT, WIDTH}
};
use embedded_graphics::{
    drawable::Pixel,
};
// use crate::geometry::Point;
use crate::display::GameColor;

pub struct PlayingField{
    id_field: [[ (u8); HEIGHT]; WIDTH],
    pub collision : bool,
    // collision_ignore : [[ (bool); HEIGHT]; WIDTH],            
    collisions : Vec<Collision>,
}

pub struct Collision{
    old_id: u8,
    new_id: u8,
}

impl PlayingField{
    pub fn new() -> Self {
        Self{
            id_field: [[0; HEIGHT]; WIDTH],
            // collision_ignore: [[false; HEIGHT]; WIDTH],
            collision: false,
            collisions: Vec::new(),
            // ticks_field : [[0; HEIGHT]; WIDTH],
            // ticks : v_ticks,
        }
    }

    pub fn store<T>(&mut self, item_pixels: T, id: u8)
    where
        T: Iterator<Item = Pixel<GameColor>>
    {   
        // let ticks = system_clock::ticks();

        for Pixel(coord, _) in item_pixels {
            if coord.0 as usize >= WIDTH || coord.1 as usize >= HEIGHT {
                continue;
            }

            let old_id = self.id_field[coord.0 as usize][coord.1 as usize];
            let mut old_ids: Vec<u8> = Vec::new();
            if !old_ids.contains(&old_id) && old_id != id && old_id != 0 {
                //self.ticks_field[coord.0 as usize][coord.1 as usize] + 10 < ticks as u8 {
                
                self.collision = true;
                    if cfg!(debug_assertions) {println!("collision");}
                // old_ids.push(old_id);
                // self.collisions.push( Collision{old_id, new_id} );
            } 
            
            self.id_field[coord.0 as usize][coord.1 as usize] = id ;
            // self.collision_ignore[coord.0 as usize][coord.1 as usize] = true;
            // self.ticks[coord.0 as usize][coord.1 as usize] = ticks_ as u8;
        }

    }

    pub fn reset_collisions(&mut self){
        self.collisions.clear();
    }

    pub fn clear(&mut self){
        self.id_field = [[0; HEIGHT]; WIDTH];
    }

}