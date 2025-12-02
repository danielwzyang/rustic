use crate::{
    picture::Picture,
    vector::{cross_product, dot_product},
};

pub struct MTL {
    pub kd: (usize, usize, usize),
    pub data: Vec<u8>,
    pub width: isize,
    pub height: isize,
}

impl MTL {
    pub fn get_texture_color(&self, i: isize) -> (u8, u8, u8) {
        let i = i as usize;
        (
            self.data[i],
            self.data[i + 1],
            self.data[i + 2],
        )
    }
}

pub fn render_textured_polygon(polygon: &[[f32; 4]; 3], vt: [[f32; 2]; 3], mtl: &MTL, light_vector: &[f32; 3]) {
    let a = [
        polygon[1][0] - polygon[0][0],
        polygon[1][1] - polygon[0][1],
        polygon[1][2] - polygon[0][2],
    ];

    let b = [
        polygon[2][0] - polygon[0][0],
        polygon[2][1] - polygon[0][1],
        polygon[2][2] - polygon[0][2],
    ];

    let normal = cross_product(&a, &b);
    let dot = f32::max(0.0, dot_product(&normal, &light_vector));
}

fn draw_scanline(picture: &mut Picture, mut x0: isize, x1: isize, y: isize, mut z0: f32, z1: f32, mut u0: f32, u1: f32, mut v0: f32, v1: f32, mtl: &MTL, dot: f32) {
    let dx = (x1 - x0).abs();
    let step_x = if x0 < x1 { 1 } else { -1 };
    let step_z = (z1 - z0) / (dx as f32 + 1.0);
    let step_u = (u1 - u0) / (dx as f32 + 1.0);
    let step_v = (v1 - v0) / (dx as f32 + 1.0);
    let mut i = y * mtl.width + x0;

    loop {
        picture.plot(x0, y, z0, &get_color(i, mtl, dot));

        if x0 == x1 { return; }

        x0 += step_x;
        z0 += step_z;
        u0 += step_u;
        v0 += step_v;
        i += step_x;
    }
}

fn get_color(i: isize, mtl: &MTL, dot: f32) -> (usize, usize, usize) {
   let texture_color = mtl.get_texture_color(i);
   
   (
       (texture_color.0 as f32 * mtl.kd.0 as f32 * dot) as usize,
       (texture_color.1 as f32 * mtl.kd.1 as f32 * dot) as usize,
       (texture_color.2 as f32 * mtl.kd.2 as f32 * dot) as usize,
   )
}
