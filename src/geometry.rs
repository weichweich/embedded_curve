#[derive(Copy,Clone)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}


pub struct Rect {
    top_left: Point,
    bottom_right: Point
}

impl Rect {
    pub fn new(top_left: Point, bottom_right: Point) -> Self {
        assert!(top_left.x < bottom_right.x, "Left point is more to the right than right point");
        assert!(top_left.y < bottom_right.y, "Top point is more to the bottom than bottom point");
        Self {
            top_left: top_left,
            bottom_right: bottom_right,
        }
    }

    pub fn inside(&self, point: Point) -> bool {
        (self.top_left.x <= point.x && self.top_left.y <= point.y
        && point.x <= self.bottom_right.x && point.y <= self.bottom_right.y)
    }
}