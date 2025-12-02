pub struct MTL {
    pub kd: (usize, usize, usize),
    pub data: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

pub fn render_textured_polygon(polygon: &[f32; 4], vt: [[f32; 2]; 3], mtl: &MTL) {

}