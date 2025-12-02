use crate::{
    picture::Picture,
    vector::{cross_product, dot_product, normalize_vector},
};

pub struct MTL {
    pub kd: (f32, f32, f32),
    pub data: Vec<u8>,
    pub width: isize,
    pub height: isize,
}

impl MTL {
    pub fn get_texture_color(&self, u: f32, v: f32) -> (u8, u8, u8) {
        let u_clamped = u.clamp(0.0, 1.0);
        let v_clamped = v.clamp(0.0, 1.0);
        let x = ((u_clamped * (self.width - 1) as f32).floor() as usize).min(self.width as usize - 1);
        let y = (((1.0 - v_clamped) * (self.height - 1) as f32).floor() as usize).min(self.height as usize - 1);
        let i = (y * self.width as usize + x) * 3;

        (
            self.data[i],
            self.data[i + 1],
            self.data[i + 2],
        )
    }
}

pub fn render_textured_polygon(picture: &mut Picture, polygon: &[[f32; 4]; 3], vt: [[f32; 2]; 3], mtl: &MTL, light_vector: &[f32; 3]) {
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

    let normal = normalize_vector(&cross_product(&a, &b));
    let light_vector = normalize_vector(&light_vector);
    let dot = f32::max(0.0, dot_product(&normal, &light_vector));
    
    let p0 = polygon[0];
    let p1 = polygon[1];
    let p2 = polygon[2];

    // sort three points by their y values so we have a bottom top and middle
    let mut b = [p0[0], p0[1], p0[2], vt[0][0], vt[0][1]];
    let mut m = [p1[0], p1[1], p1[2], vt[1][0], vt[1][1]];
    let mut t = [p2[0], p2[1], p2[2], vt[2][0], vt[2][1]];

    if b[1] > m[1] {
        std::mem::swap(&mut b, &mut m);
    }
    if m[1] > t[1] {
        std::mem::swap(&mut m, &mut t);
    }
    if b[1] > m[1] {
        std::mem::swap(&mut b, &mut m);
    }

    let y_start = b[1] as isize;
    let y_mid = m[1] as isize;
    let y_end = t[1] as isize;

    let distance0 = (y_end - y_start) as f32 + 1.0;
    let distance1 = (y_mid - y_start) as f32 + 1.0;
    let distance2 = (y_end - y_mid) as f32 + 1.0;

    let dx0 = (t[0] - b[0]) / distance0;
    let dz0 = (t[2] - b[2]) / distance0;
    let du0 = (t[3] - b[3]) / distance0;
    let dv0 = (t[4] - b[4]) / distance0;
    let mut dx1 = (m[0] - b[0]) / distance1;
    let mut dz1 = (m[2] - b[2]) / distance1;
    let mut du1 = (m[3] - b[3]) / distance1;
    let mut dv1 = (m[4] - b[4]) / distance1;

    let mut x0 = b[0];
    let mut z0 = b[2];
    let mut u0 = b[3];
    let mut v0 = b[4];
    let mut x1 = b[0];
    let mut z1 = b[2];
    let mut u1 = b[3];
    let mut v1 = b[4];

    let mut flip = false;
    let mut y = y_start;

    while y <= y_end {
        if !flip && y >= y_mid {
            flip = true;
            dx1 = (t[0] - m[0]) / distance2;
            dz1 = (t[2] - m[2]) / distance2;
            du1 = (t[3] - m[3]) / distance2;
            dv1 = (t[4] - m[4]) / distance2;
            x1 = m[0];
            z1 = m[2];
            u1 = m[3];
            v1 = m[4];
        }

        draw_scanline(
            picture,
            x0 as isize,
            x1 as isize,
            y,
            z0,
            z1,
            u0,
            u1,
            v0,
            v1,
            mtl,
            dot,
        );

        x0 += dx0;
        z0 += dz0;
        u0 += du0;
        v0 += dv0;
        x1 += dx1;
        z1 += dz1;
        u1 += du1;
        v1 += dv1;
        y += 1;
    }
}

fn draw_scanline(picture: &mut Picture, mut x0: isize, x1: isize, y: isize, mut z0: f32, z1: f32, mut u0: f32, u1: f32, mut v0: f32, v1: f32, mtl: &MTL, dot: f32) {
    let dx = (x1 - x0).abs();
    let step_x = if x0 < x1 { 1 } else { -1 };
    let step_z = (z1 - z0) / (dx as f32 + 1.0);
    let step_u = (u1 - u0) / (dx as f32 + 1.0);
    let step_v = (v1 - v0) / (dx as f32 + 1.0);

    loop {
        picture.plot(x0, y, z0, &get_color(u0, v0, mtl, dot));

        if x0 == x1 { return; }

        x0 += step_x;
        z0 += step_z;
        u0 += step_u;
        v0 += step_v;
    }
}

fn get_color(u0: f32, v0: f32, mtl: &MTL, dot: f32) -> (usize, usize, usize) {
    let texture_color = mtl.get_texture_color(u0, v0);
    (
        (texture_color.0 as f32 * mtl.kd.0 * dot).clamp(0.0, 255.0) as usize,
        (texture_color.1 as f32 * mtl.kd.1 * dot).clamp(0.0, 255.0) as usize,
        (texture_color.2 as f32 * mtl.kd.2 * dot).clamp(0.0, 255.0) as usize,
    )
}
