#[derive(Clone,Copy)]
pub struct Character {

    min_u: f32,
    min_v: f32,
    max_u: f32,
    max_v: f32,

    width: u32
}

impl Character {

    pub fn new(texture_width: u32, texture_height: u32, min_x: u32,  min_y: u32, max_x: u32, max_y: u32) -> Character {
        let float_width = texture_width as f32 + 1.0;
        let float_height = texture_height as f32 + 1.0;
        Character {
            min_u: min_x as f32 / float_width,
            min_v: max_y as f32 / float_height,
            max_u: max_x as f32 / float_width,
            max_v: min_y as f32 / float_height,
            width: max_x - min_x + 1
        }
    }

    pub fn get_left_u(&self) -> f32 {
        self.min_u
    }

    pub fn get_bottom_v(&self) -> f32 {
        self.min_v
    }

    pub fn get_right_u(&self) -> f32 {
        self.max_u
    }

    pub fn get_top_v(&self) -> f32 {
        self.max_v
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }
}