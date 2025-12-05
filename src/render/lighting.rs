type Vector = [f32; 3];

use crate::{
    constants::SPECULAR_EXPONENT,
    vector::{normalize_vector, dot_product}
};

pub struct LightingConfig {
    pub ambient_light_color: Vector,
    pub point_lights: Vec<[Vector; 2]>,
    // note: viewer vector is always <0, 0, 1> so all the math for backface culling and lighting is hardcoded
}

#[derive(Clone, Copy)]
pub struct ReflectionConstants {
    pub ambient: Vector,
    pub diffuse: Vector,
    pub specular: Vector,
}

pub fn get_illumination(normal: &Vector, config: &LightingConfig, constants: &ReflectionConstants) -> (usize, usize, usize) {
    let normal = &normalize_vector(&normal);

    let ambient = get_ambient(&config.ambient_light_color, &constants.ambient);
    let diffuse = get_diffuse(normal, &config.point_lights, &constants.diffuse);
    let specular = get_specular(normal, &config.point_lights, &constants.specular);

    clamp_color([
        ambient[0] + diffuse[0] + specular[0],
        ambient[1] + diffuse[1] + specular[1],
        ambient[2] + diffuse[2] + specular[2],
    ])
}

pub fn get_ambient(ambient_light_color: &Vector, ambient_constant: &Vector) -> Vector {
    // i_ambient = ambient color * ambient reflection constant
    [
        ambient_light_color[0] * ambient_constant[0],
        ambient_light_color[1] * ambient_constant[1],
        ambient_light_color[2] * ambient_constant[2],
    ]
}

pub fn get_diffuse(normal: &Vector, point_lights: &Vec<[Vector; 2]>, diffuse_constant: &Vector) -> Vector {
    // i_diffuse = point color * diffuse reflection constant * (normalized normal dot normalized light)
    let mut diffuse = [0.0, 0.0, 0.0];
    for [light_color, light_vector] in point_lights {
        let n_dot_l = f32::max(0.0, dot_product(normal, light_vector));
        diffuse[0] += light_color[0] * diffuse_constant[0] * n_dot_l;
        diffuse[1] += light_color[1] * diffuse_constant[1] * n_dot_l;
        diffuse[2] += light_color[2] * diffuse_constant[2] * n_dot_l;
    }
    diffuse
}

pub fn get_specular(normal: &Vector, point_lights: &Vec<[Vector; 2]>, specular_constant: &Vector) -> Vector {
    // i_specular = point color * specular reflection constant * (normalized reflection dot view)^exp
    // where exp > 1
    // normalized reflection = [2 * normalized normal * (normalized normal dot normalized light) - normalized light]
    
    // for normalized reflection dot view:
    // since view just <0, 0, 1>, we can be lazy and take the z value, raise it to exp, and call it r_z
    let mut specular = [0.0, 0.0, 0.0];
    for [light_color, light_vector] in point_lights {
        let n_dot_l = f32::max(0.0, dot_product(normal, light_vector));
        let r_z = f32::max(0.0, 2.0 * normal[2] * n_dot_l - light_vector[2]).powf(SPECULAR_EXPONENT);

        specular[0] += light_color[0] * specular_constant[0] * r_z;
        specular[1] += light_color[1] * specular_constant[1] * r_z;
        specular[2] += light_color[2] * specular_constant[2] * r_z;
    }
    specular
}

fn clamp_color(vector: Vector) -> (usize, usize, usize) {
    (
        vector[0].clamp(0.0, 255.0) as usize,
        vector[1].clamp(0.0, 255.0) as usize,
        vector[2].clamp(0.0, 255.0) as usize,
    )
}
