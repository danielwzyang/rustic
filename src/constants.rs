#![allow(dead_code)]
use crate::render::ReflectionConstants;

/* CONFIG */
pub const DEFAULT_SCRIPT: &str = "scripts/stonehenge.mdl";
pub const DEFAULT_PICTURE_DIMENSIONS: (usize, usize) = (500, 500);
pub const DEFAULT_BACKGROUND_COLOR: (usize, usize, usize) = WHITE;
pub const DEFAULT_FOREGROUND_COLOR: (usize, usize, usize) = BLUE;
pub const PARAMETRIC_STEPS: i32 = 20;
pub const ENABLE_BACK_FACE_CULLING: bool = true;
pub const ENABLE_Z_BUFFER: bool = true;
pub const DEFAULT_REFLECTION_CONSTANTS: ReflectionConstants = ReflectionConstants {
    ambient: [0.2, 0.2, 0.2],
    diffuse: [0.5, 0.5, 0.5],
    specular: [0.5, 0.5, 0.5],
};
#[derive(Clone, Copy, Debug)]
pub enum ShadingMode {
    Wireframe,
    FlatRandom,
    Flat,
    Gouraud,
    Phong,
}
pub const DEFAULT_SHADING_MODE: ShadingMode = ShadingMode::Flat;
pub const SPECULAR_EXPONENT: f32 = 5.0;
pub const GENERATE_TEMPORARY_FRAME_FILES: bool = false;
pub const DEFAULT_ANIMATION_DELAY_MS: u32 = 20; // for some reason when this is set to 10 ms it becomes really slow

/* COLORS */
pub const WHITE: (usize, usize, usize) = (255, 255, 255);
pub const BLACK: (usize, usize, usize) = (0, 0, 0);
pub const RED: (usize, usize, usize) = (255, 0, 0);
pub const GREEN: (usize, usize, usize) = (0, 255, 0);
pub const BLUE: (usize, usize, usize) = (0, 0, 255);
pub const CYAN: (usize, usize, usize) = (0, 255, 255);
pub const YELLOW: (usize, usize, usize) = (255, 255, 0);
pub const MAGENTA: (usize, usize, usize) = (255, 0, 255);

/* USEFUL MATH STUFF */
pub const HERMITE: [[f32; 4]; 4] = [
    [2.0, -3.0, 0.0, 1.0],
    [-2.0, 3.0, 0.0, 0.0],
    [1.0, -2.0, 1.0, 0.0],
    [1.0, -1.0, 0.0, 0.0],
];
pub const BEZIER: [[f32; 4]; 4] = [
    [-1.0, 3.0, -3.0, 1.0],
    [3.0, -6.0, 3.0, 0.0],
    [-3.0, 3.0, 0.0, 0.0],
    [1.0, 0.0, 0.0, 0.0],
];
pub const CUBE: [(usize, usize, usize); 12] = [
    (0, 2, 1),
    (0, 3, 2),
    (4, 1, 5),
    (4, 0, 1),
    (7, 0, 4),
    (7, 3, 0),
    (6, 3, 7),
    (6, 2, 3),
    (5, 2, 6),
    (5, 1, 2),
    (7, 5, 6),
    (7, 4, 5),
];
