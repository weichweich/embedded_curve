extern crate alloc;

use stm32f7_discovery::{
    system_clock::{self, Hz},
    lcd::{ HEIGHT, WIDTH}
};

// use crate::geometry::{
//     AABBox, Point, Vector2D
// };

use embedded_graphics::{
    drawable::Pixel,
};

use crate::display::GameColor;

pub struct PlayingField{
    id : [[ (u8); HEIGHT]; WIDTH],
    pub collision : bool,
    // ticks : Vec<Vec<u8>>,                        Option 1: 2D Array
    // ticks : [[ (u8); HEIGHT]; WIDTH],            Option 2: 2D Vectors
    // usize old_ticks : usize,
    // collisions : Vec<Collision>,
}

// pub struct Collision{
//     id : u32,
//     pos : Point,
// }

impl PlayingField{
    pub fn new() -> Self {
        // let mut v_ticks = Vec::new();
        // v_ticks.resize( HEIGHT, Vec::new() );
        // for i in 0..HEIGHT {
        //     v_ticks[i].resize(WIDTH, 0);
        // }
        // println!("...");

        Self{
            id : [[0; HEIGHT]; WIDTH],
            collision : false,
            // ticks : [[0; HEIGHT]; WIDTH],
            // ticks : v_ticks,
            // collisions : Vec::new(),

        }
    }

    pub fn store<T>(&mut self, item_pixels: T, id: u32)
    where
        T: Iterator<Item = Pixel<GameColor>>
    {   
        // let ticks_ = system_clock::ticks();

        for Pixel(coord, _) in item_pixels {
            if coord.0 as usize >= WIDTH || coord.1 as usize >= HEIGHT {
                continue;
            }
            // println!("test");

            if self.id[coord.0 as usize][coord.1 as usize] != id as u8 &&  //other object intersected
             self.id[coord.0 as usize][coord.1 as usize] != 0 {            //
                // && self.ticks[coord.0 as usize][coord.1 as usize] + 10 < ticks_ as u8 {
                
                self.collision = true;
                println!("collision");
                // self.collisions.push(Collision{id: id, pos: 
                //     Point(x: coord.0 as usize, y: coord.1 as usize)});
            } 
            
            self.id[coord.0 as usize][coord.1 as usize] = id as u8;
            // self.ticks[coord.0 as usize][coord.1 as usize] = ticks_ as u8;
        }

    }  

}