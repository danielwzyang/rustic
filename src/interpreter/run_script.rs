#![allow(dead_code)]

use std::{
    collections::HashMap, error::Error, vec
};

use crate::{
    constants::{
        DEFAULT_ANIMATION_DELAY_MS, DEFAULT_BACKGROUND_COLOR, DEFAULT_FOREGROUND_COLOR, DEFAULT_PICTURE_DIMENSIONS, DEFAULT_REFLECTION_CONSTANTS, DEFAULT_SHADING_MODE, GENERATE_TEMPORARY_FRAME_FILES, ShadingMode
    }, interpreter::animation::Animation, matrix, render::{
        LightingConfig,
        Picture,
        ReflectionConstants,
        edge_list::{add_bezier_curve, add_circle, add_edge, add_hermite_curve, render_edges},
        polygon_list::{add_box, add_polygon, add_sphere, add_torus, render_polygons},
        texture::{MTL, render_textured_polygon},
    }, vector::{cross_product, dot_product, normalize_vector, subtract_vectors}
};
use super::{
    coordinate_stack::CoordinateStack,
    parser::Command,
    animation,
    mesh::handle_mesh,
};

type Matrix = Vec<[f32; 4]>;

enum Symbol {
    Constants(ReflectionConstants),
    Knob(f32),
}

enum CachedMesh {
    NoTexture(Matrix),
    Texture((Matrix, Vec<(String, [[f32; 2]; 3])>, HashMap<String, MTL>)),
}

struct ScriptContext {
    picture: Picture,
    edges: Matrix,
    polygons: Matrix,
    coordinate_stack: CoordinateStack,
    shading_mode: ShadingMode,
    lighting_config: LightingConfig,
    reflection_constants: ReflectionConstants,
    camera_matrix: Matrix,
    symbols: HashMap<String, Symbol>,
    mesh_cache: HashMap<String, CachedMesh>,
}

impl ScriptContext {
    fn new() -> Self {
        Self {
            picture: Picture::new(DEFAULT_PICTURE_DIMENSIONS.0, DEFAULT_PICTURE_DIMENSIONS.1, 255, &DEFAULT_BACKGROUND_COLOR),
            edges: matrix::new(),
            polygons: matrix::new(),
            coordinate_stack: CoordinateStack::new(),
            shading_mode: DEFAULT_SHADING_MODE,
            lighting_config: LightingConfig {
                ambient_light_color: [50.0, 50.0, 50.0],
                point_lights: vec![[[255.0, 255.0, 255.0], [0.0, 0.0, 1.0]]],
            },
            reflection_constants: DEFAULT_REFLECTION_CONSTANTS,
            camera_matrix: matrix::identity(),
            symbols: HashMap::new(),
            mesh_cache: HashMap::new(),
        }
    }

    fn frame_reset(&mut self) {
        self.picture = Picture::new(DEFAULT_PICTURE_DIMENSIONS.0, DEFAULT_PICTURE_DIMENSIONS.1, 255, &DEFAULT_BACKGROUND_COLOR);
        self.edges = matrix::new();
        self.polygons = matrix::new();
        self.coordinate_stack = CoordinateStack::new();
    }

    fn render_edges(&mut self) {
        matrix::multiply(&self.coordinate_stack.peek(), &mut self.edges);
        render_edges(&self.edges, &mut self.picture, &DEFAULT_FOREGROUND_COLOR);
        self.edges = matrix::new();
    }

    fn render_polygons(&mut self, constants: &Option<String>) {
        let mut reflection_constants = &self.reflection_constants;

        if let Some(name) = constants && let Some(symbol) = self.symbols.get(name) {
            match symbol {
                Symbol::Constants(constants) => reflection_constants = constants,
                _ => panic!("Expected symbol to be lighting constants: {}", name)
            }
        }

        matrix::multiply(&self.coordinate_stack.peek(), &mut self.polygons);
        matrix::multiply(&self.camera_matrix, &mut self.polygons);

        render_polygons(&self.polygons, &mut self.picture, &DEFAULT_FOREGROUND_COLOR, &self.shading_mode, &self.lighting_config, reflection_constants);
        self.polygons = matrix::new();
    }

    fn render_textured_polygons(&mut self, polygon_info: &Vec<(String, [[f32; 2]; 3])>, mtls: &HashMap<String, MTL>) {
        matrix::multiply(&self.coordinate_stack.peek(), &mut self.polygons);
        matrix::multiply(&self.camera_matrix, &mut self.polygons);
        
        let mut polygon_index = 0;

        for (mtl, [vt0, vt1, vt2]) in polygon_info.iter() {
            let triangle_slice: &[[f32; 4]; 3] = self.polygons[polygon_index..polygon_index + 3].try_into().unwrap();

            render_textured_polygon(
                &mut self.picture,
                triangle_slice,
                [*vt0, *vt1, *vt2],
                mtls.get(mtl).unwrap(),
                &self.lighting_config.point_lights[0][1], // too lazy to do multiple point lights for textures (might do later)
            );

            polygon_index += 3; 
        }

        self.polygons = matrix::new();
    }

    fn get_knob_value(&self, knob_name: &Option<String>) -> f32 {
        if let Some(name) = knob_name && let Some(Symbol::Knob(value)) = self.symbols.get(name) {
            *value
        } else {
            // if no knob is provided use 1.0 as a default value
            1.0
        }
    }

    fn set_knob(&mut self, name: String, value: f32) {
        self.symbols.insert(name, Symbol::Knob(value));
    }

    fn set_all_knobs(&mut self, value: f32) {
        for (_, symbol) in self.symbols.iter_mut() {
            match symbol {
                Symbol::Knob(old_value) => { *old_value = value }
                _ => {}
            }
        }
    }
}

pub fn evaluate_commands(commands: Vec<Command>) -> Result<(), Box<dyn Error>> {
    let mut context = ScriptContext::new();

    let (num_frames, basename) = animation::first_pass(&commands)?;

    if num_frames == 0 {
        for command in commands {
            execute_command(command, &mut context)?;
        }
    } else {
        let frame_knob_list = animation::second_pass(&commands, &num_frames)?;
        let mut gif = Animation::new(context.picture.xres, context.picture.yres);

        for frame in 0..num_frames {
            context.frame_reset();

            for (name, value) in &frame_knob_list[frame] {
                context.set_knob(name.clone(), *value);
            }

            for command in commands.clone() {
                execute_command(command, &mut context)?;
            }

            if GENERATE_TEMPORARY_FRAME_FILES {
                context.picture.save_as_file(format!("temp_frames/{}_{:03}.png", basename, frame).as_str())?;
            } else {
                gif.add_frame(&context.picture.data);
            }
        }

        if !GENERATE_TEMPORARY_FRAME_FILES {
            gif.save_as_file(format!("{}.gif", basename), DEFAULT_ANIMATION_DELAY_MS)?;
        }
    }

    Ok(())
}

fn execute_command(command: Command, context: &mut ScriptContext) -> Result<(), Box<dyn Error>> {
    match command {
        Command::Display => {
            context.picture.display()?
        }

        Command::Save { file_path } => {
            context.picture.save_as_file(&file_path)?
        }

        Command::Clear => {
            context.picture.clear();
        }

        Command::Push => {
            context.coordinate_stack.push();
        }

        Command::Pop => {
            context.coordinate_stack.pop();
        }

        Command::Move { a, b, c, knob } => {
            let multiplier = context.get_knob_value(&knob);
            context.coordinate_stack.apply_transformation(matrix::translation(a * multiplier, b * multiplier, c * multiplier));
        }

        Command::Scale { a, b, c, knob } => {
            let multiplier = context.get_knob_value(&knob);
            // we need to make sure this goes from 1.0 -> a
            let scale_a = 1.0 + (a - 1.0) * multiplier;
            let scale_b = 1.0 + (b - 1.0) * multiplier;
            let scale_c = 1.0 + (c - 1.0) * multiplier;
            context.coordinate_stack.apply_transformation(matrix::dilation(scale_a, scale_b, scale_c));
        }

        Command::Rotate { axis, degrees, knob } => {
            let multiplier = context.get_knob_value(&knob);
            context.coordinate_stack.apply_transformation(matrix::rotation(axis, degrees * multiplier));
        }

        Command::Line { x0, y0, z0, x1, y1, z1 } => {
            add_edge(&mut context.edges, x0, y0, z0, x1, y1, z1);
            context.render_edges();
        }

        Command::Circle { x, y, z, r } => {
            add_circle(&mut context.edges, x, y, z, r);
            context.render_edges();
        }

        Command::Hermite { x0, y0, x1, y1, rx0, ry0, rx1, ry1 } => {
            add_hermite_curve(&mut context.edges, x0, y0, x1, y1, rx0, ry0, rx1, ry1);
            context.render_edges();
        }

        Command::Bezier { x0, y0, x1, y1, x2, y2, x3, y3 } => {
            add_bezier_curve(&mut context.edges, x0, y0, x1, y1, x2, y2, x3, y3);
            context.render_edges();
        }

        Command::Polygon { x0, y0, z0, x1, y1, z1, x2, y2, z2 } => {
            add_polygon(&mut context.polygons, x0, y0, z0, x1, y1, z1, x2, y2, z2);
            context.render_polygons(&None);
        }

        Command::Box { constants, x, y, z, w, h, d } => {
            add_box(&mut context.polygons, x, y, z, w, h, d);
            context.render_polygons(&constants);
        }

        Command::Sphere { constants, x, y, z, r } => {
            add_sphere(&mut context.polygons, x, y, z, r);
            context.render_polygons(&constants);
        }

        Command::Torus { constants, x, y, z, r0, r1 } => {
            add_torus(&mut context.polygons, x, y, z, r0, r1);
            context.render_polygons(&constants);
        }

        Command::Mesh { constants, file_path } => {
            let mut polygons: Matrix = vec![];
            let mut polygon_info: Vec<(String, [[f32; 2]; 3])> = vec![];
            let mut mtls: HashMap<String, MTL> = HashMap::new();
            if let Some(cache) = context.mesh_cache.get(&file_path) {
                match cache {
                    CachedMesh::NoTexture(cache) => polygons = cache.clone(), 
                    CachedMesh::Texture(cache) => {
                        polygons = cache.0.clone();
                        polygon_info = cache.1.clone();
                        mtls = cache.2.clone();
                    }
                }
            }

            if !polygons.is_empty() {
                context.polygons = polygons.clone(); 
                if !polygon_info.is_empty() {
                    context.render_textured_polygons(&polygon_info, &mtls);
                } else {
                    context.render_polygons(&constants);
                }
            } else if let Some((polygon_info, mtls)) = handle_mesh(&mut context.polygons, &file_path)? {
                polygons = context.polygons.clone();
                context.render_textured_polygons(&polygon_info, &mtls);
                context.mesh_cache.insert(
                    file_path,
                    CachedMesh::Texture((
                        polygons,
                        polygon_info,
                        mtls,
                    ))
                );
            } else {
                polygons = context.polygons.clone();
                context.render_polygons(&constants);
                context.mesh_cache.insert(
                    file_path,
                    CachedMesh::NoTexture(polygons)
                );
            }
        }

        Command::SetLight { r, g, b, x, y, z } => {
            context.lighting_config.point_lights.push([[r, g, b], normalize_vector(&[x, y, z])]);
        }

        Command::SetAmbient { r, g, b } => {
            context.lighting_config.ambient_light_color = [r, g, b];
        }

        Command::SetConstants { name, kar, kdr, ksr, kag, kdg, ksg, kab, kdb, ksb } => {
            let constants = ReflectionConstants {
                ambient: [kar, kag, kab],
                diffuse: [kdr, kdg, kdb],
                specular: [ksr, ksg, ksb],
            };

            context.symbols.insert(name, Symbol::Constants(constants));
        }

        Command::SetShading { shading_mode } => {
            context.shading_mode = shading_mode.clone();
        }

        Command::SetCamera { eye_x, eye_y, eye_z, aim_x, aim_y, aim_z } => {
            // based on opengl's camera transformation matrix
            // keeps the viewing vector for the math at a consistent <0, 0, 1>
            let eye = [eye_x, eye_y, eye_z];
            let aim = [aim_x, aim_y, aim_z];
            let forward = normalize_vector(&subtract_vectors(&aim, &eye));
            let up = [0.0, 1.0, 0.0];

            let right = normalize_vector(&cross_product(&forward, &up));
            let up_new = cross_product(&right, &forward);

            let ex = -dot_product(&right, &eye);
            let ey = -dot_product(&up_new, &eye);
            let ez =  dot_product(&forward, &eye);

            context.camera_matrix = vec![
                [ right[0], right[1], right[2], 0.0 ],
                [ up_new[0], up_new[1], up_new[2], 0.0 ],
                [ -forward[0], -forward[1], -forward[2], 0.0 ],
                [ ex, ey, ez, 1.0 ],
            ];
        }

        Command::SetKnob { name, value } => {
            context.set_knob(name, value);
        }

        Command::SetAllKnobs { value } => {
            context.set_all_knobs(value);
        }

        _ => { }
    }

    Ok(())
}

