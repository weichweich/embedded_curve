extern crate alloc;

use stm32f7_discovery::{
    lcd::{Framebuffer, Layer, Color},
};

use alloc::vec::Vec;
use bresenham::{
    Bresenham
};

use crate::geometry::{
    Point
};

pub fn draw_line<F: Framebuffer>(start: Point, end: Point, layer: &mut Layer<F>, color: Color) {
    let bi = Bresenham::new((start.x as isize, start.y as isize), 
                            (end.x as isize, end.y as isize));
    for p in bi {
        layer.print_point_color_at(p.0 as usize, p.1 as usize, color);
    }
}

//Scanline Polygon Rendering - Sorted hashset
pub fn draw_triangle<F: Framebuffer>(points: [Point; 3], layer: &mut Layer<F>, color: Color) {

    // draw_line(points[0], points[1], layer, color);
    // draw_line(points[1], points[2], layer, color);
    // draw_line(points[2], points[0], layer, color);

    let mut sorted_rows : Vec<Vec<Point>> = Vec::new();
    let i_y_min: usize;
    let i_y_max: usize;

    //calculate index for y_min
    if points[0].y < points[1].y {
        if points[0].y < points[2].y {
            i_y_min = 0
        } else {
            i_y_min = 2;
        }
    } else if points[1].y < points[2].y {
        i_y_min = 1;
    } else {
        i_y_min = 2;
    }

    //calculate index for y_max
    if points[0].y > points[1].y {
        if points[0].y > points[2].y {
            i_y_max = 0
        } else {
            i_y_max = 2;
        }
    } else if points[1].y > points[2].y {
        i_y_max = 1;
    } else {
        i_y_max = 2;
    }

    let y_offset = points[i_y_min].y;
    let range = (points[i_y_max].y as isize - points[i_y_min].y as isize) as usize;
    sorted_rows.resize(range+1, Vec::new() );
    
    // println!("{}", range);
    // println!("{}", y_offset );

    //Sort Bresenham pixels according to y and store in sorted_rows    
    for i in 0..3 {
        let b = Bresenham::new((points[i].x as isize, points[i].y as isize), 
                    (points[(i+1) % 3].x as isize, points[(i+1) % 3].y as isize));

        for (x_, y_) in b {
            let j : usize = (y_ - y_offset as isize) as usize ;
            assert!(j <= range, "Array out of bounds: sorted_rows!");
            sorted_rows[j].push( Point{x: x_ as usize, y: y_ as usize } );

        }
    }

    //Draw lines/rows pixelwise
    for row in sorted_rows {
        let y_ = row[0].y;
        let (x_min, x_max) = if row[0].x < row[row.len()-1].x {
            (row[0].x, row[row.len()-1].x)
        } else {
            (row[row.len()-1].x, row[0].x)
        };
        for x_ in x_min..=x_max {
            layer.print_point_color_at(x_ as usize, y_ as usize, color);
        }
       // draw_line(row[0], row[row.len()-1], layer, color);
    }
}

