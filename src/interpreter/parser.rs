#![allow(dead_code)]

use std::{
    collections::VecDeque, error::Error
};

use crate::{
    constants::ShadingMode,
    matrix::Rotation,
};
use super::tokens::{Token, TokenType, Function};

// file paths + identifiers stored as String
#[derive(Clone, Debug)]
pub enum Command {
    Display,
    Save { file_path: String },
    Clear,
    Push,
    Pop,
    Move { a: f32, b: f32, c: f32, knob: Option<String> },
    Scale { a: f32, b: f32, c: f32, knob: Option<String> },
    Rotate { axis: Rotation, degrees: f32, knob: Option<String> },
    Line {  x0: f32, y0: f32, z0: f32, coord_system0: Option<String>, x1: f32, y1: f32, z1: f32, coord_system1: Option<String> },
    Circle { x: f32, y: f32, z: f32, r: f32 },
    Hermite { x0: f32, y0: f32, x1: f32, y1: f32, rx0: f32, ry0: f32, rx1: f32, ry1: f32 },
    Bezier { x0: f32, y0: f32, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32 },
    Polygon { x0: f32, y0: f32, z0: f32, x1: f32, y1: f32, z1: f32, x2: f32, y2: f32, z2: f32 },
    Box { constants: Option<String>, x: f32, y: f32, z: f32, w: f32, h: f32, d: f32, coord_system: Option<String> },
    Sphere { constants: Option<String>, x: f32, y: f32, z: f32, r: f32, coord_system: Option<String> },
    Torus { constants: Option<String>, x: f32, y: f32, z: f32, r0: f32, r1: f32, coord_system: Option<String> },
    Cylinder { constants: Option<String>, x: f32, y: f32, z: f32, r: f32, h: f32, coord_system: Option<String> },
    Cone { constants: Option<String>, x: f32, y: f32, z: f32, r: f32, h: f32, coord_system: Option<String> },
    Mesh { constants: Option<String>, file_path: String, coord_system: Option<String> },
    ClearLights,
    AddLight { r: f32, g: f32, b: f32, x: f32, y: f32, z: f32 },
    SetAmbient { r: f32, g: f32, b: f32 },
    DefineConstants { name: String, kar: f32, kdr: f32, ksr: f32, kag: f32, kdg: f32, ksg: f32, kab: f32, kdb: f32, ksb: f32 },
    SetShading { shading_mode: ShadingMode },
    SetCamera { eye_x: f32, eye_y: f32, eye_z: f32, aim_x: f32, aim_y: f32, aim_z: f32 },
    SetBaseName { name: String },
    SetKnob { name: String, value: f32 },
    SaveKnobList { name: String },
    Tween { start_frame: usize, end_frame: usize, knoblist0: String, knoblist1: String, easing: Option<String> },
    SetFrames { num_frames: usize },
    VaryKnob { knob: String, start_frame: usize, end_frame: usize, start_val: f32, end_val: f32, easing: Option<String> },
    SetAllKnobs { value: f32 },
    SaveCoordSystem { name: String },
    GenerateRayFiles,
    SetFocalLength { length: f32 },
}

pub struct Parser {
    stack: VecDeque<Token>,
}

impl Parser {
    pub fn new() -> Self {
        Self { stack: VecDeque::new() }
    }
    
    fn pop_optional_type(&mut self, token_type: TokenType) -> Option<String> {
        if let Some(token) = self.stack.front() && token.token_type == token_type {
            let token = self.stack.pop_front().unwrap();
            return Some(token.value.clone())
        }

        None
    }

    fn pop(&mut self) -> Result<Token, Box<dyn Error>> {
        if let Some(token) = self.stack.pop_front() {
            Ok(token)
        } else {
            Err("Expected token but stack was empty.".into())
        }
    }

    pub fn generate_command_list(&mut self, tokens: VecDeque<Token>) -> Result<Vec<Command>, Box<dyn Error>> {
        let mut commands: Vec<Command> = vec![];

        self.stack = tokens;

        while !self.stack.is_empty() {
            let token = self.pop()?;

            match token.token_type {
                TokenType::Command(function) => {
                    commands.push(
                        match function {
                            Function::Display => { Command::Display }
                            Function::Save => { self.handle_save()? }
                            Function::Clear => { Command::Clear }
                            Function::Push => { Command::Push }
                            Function::Pop => { Command::Pop }
                            Function::Move => { self.handle_move()? }
                            Function::Scale => { self.handle_scale()? }
                            Function::Rotate => { self.handle_rotate()? }
                            Function::Line => { self.handle_line()? }
                            Function::Circle => { self.handle_circle()? }
                            Function::Hermite => { self.handle_hermite()? }
                            Function::Bezier => { self.handle_bezier()? }
                            Function::Polygon => { self.handle_polygon()? }
                            Function::Box => { self.handle_box()? }
                            Function::Sphere => { self.handle_sphere()? }
                            Function::Torus => { self.handle_torus()? }
                            Function::Cylinder => { self.handle_cylinder()? }
                            Function::Cone => { self.handle_cone()? }
                            Function::Mesh => { self.handle_mesh()? }
                            Function::ClearLights => { Command::ClearLights }
                            Function::AddLight => { self.handle_add_light()? }
                            Function::SetAmbient => { self.handle_set_ambient()? }
                            Function::DefineConstants => { self.handle_define_constants()? }
                            Function::SetShading => { self.handle_set_shading()? }
                            Function::SetCamera => { self.handle_set_camera()? }
                            Function::SetBaseName => { self.handle_set_base_name()? }
                            Function::SetKnob => { self.handle_set_knob()? }
                            Function::SaveKnobList => { self.handle_save_knob_list()? }
                            Function::Tween => { self.handle_tween()? }
                            Function::SetFrames => { self.handle_set_frames()? }
                            Function::VaryKnob => { self.handle_vary_knob()? }
                            Function::SetAllKnobs => { self.handle_set_all_knobs()? }
                            Function::SaveCoordSystem => { self.handle_save_coord_system()? }
                            Function::GenerateRayFiles => { Command::GenerateRayFiles }
                            Function::SetFocalLength => { self.handle_set_focal_length()? }
                        }
                    )
                }

                _ => {
                    return Err(format!("{} -> Unexpected token: {} ({:?})", token.info, token.value, token.token_type).into())
                }
            }
        }

        Ok(commands)
    }

    fn handle_save(&mut self) -> Result<Command, Box<dyn Error>> {
        let file_path = self.pop()?.value;

        Ok(Command::Save { file_path })
    }

    fn handle_move(&mut self) -> Result<Command, Box<dyn Error>> {
        let a = Parser::convert_to_f32(self.pop()?.value)?;
        let b = Parser::convert_to_f32(self.pop()?.value)?;
        let c = Parser::convert_to_f32(self.pop()?.value)?;
        let knob = self.pop_optional_type(TokenType::Identifier);

        Ok(Command::Move { a, b, c, knob })
    }

    fn handle_scale(&mut self) -> Result<Command, Box<dyn Error>> {
        let a = Parser::convert_to_f32(self.pop()?.value)?;
        let b = Parser::convert_to_f32(self.pop()?.value)?;
        let c = Parser::convert_to_f32(self.pop()?.value)?;
        let knob = self.pop_optional_type(TokenType::Identifier);

        Ok(Command::Scale { a, b, c, knob })
    }

    fn handle_rotate(&mut self) -> Result<Command, Box<dyn Error>> {
        let axis_str = self.pop()?.value.to_lowercase();
        let axis = match axis_str.as_str() {
            "x" => Rotation::X,
            "y" => Rotation::Y,
            "z" => Rotation::Z,
            _ => return Err(format!("Invalid rotation axis: {}", axis_str).into()),
        };
        let degrees = Parser::convert_to_f32(self.pop()?.value)?;
        let knob = self.pop_optional_type(TokenType::Identifier);

        Ok(Command::Rotate { axis, degrees, knob })
    }

    fn handle_line(&mut self) -> Result<Command, Box<dyn Error>> {
        let _ = self.pop_optional_type(TokenType::Identifier); // constants
        let x0 = Parser::convert_to_f32(self.pop()?.value)?;
        let y0 = Parser::convert_to_f32(self.pop()?.value)?;
        let z0 = Parser::convert_to_f32(self.pop()?.value)?;
        let coord_system0 = self.pop_optional_type(TokenType::Identifier);
        let x1 = Parser::convert_to_f32(self.pop()?.value)?;
        let y1 = Parser::convert_to_f32(self.pop()?.value)?;
        let z1 = Parser::convert_to_f32(self.pop()?.value)?;
        let coord_system1 = self.pop_optional_type(TokenType::Identifier);

        Ok(Command::Line { x0, y0, z0, coord_system0, x1, y1, z1, coord_system1 })
    }

    fn handle_circle(&mut self) -> Result<Command, Box<dyn Error>> {
        let x = Parser::convert_to_f32(self.pop()?.value)?;
        let y = Parser::convert_to_f32(self.pop()?.value)?;
        let z = Parser::convert_to_f32(self.pop()?.value)?;
        let r = Parser::convert_to_f32(self.pop()?.value)?;

        Ok(Command::Circle { x, y, z, r })
    }

    fn handle_hermite(&mut self) -> Result<Command, Box<dyn Error>> {
        let x0 = Parser::convert_to_f32(self.pop()?.value)?;
        let y0 = Parser::convert_to_f32(self.pop()?.value)?;
        let x1 = Parser::convert_to_f32(self.pop()?.value)?;
        let y1 = Parser::convert_to_f32(self.pop()?.value)?;
        let rx0 = Parser::convert_to_f32(self.pop()?.value)?;
        let ry0 = Parser::convert_to_f32(self.pop()?.value)?;
        let rx1 = Parser::convert_to_f32(self.pop()?.value)?;
        let ry1 = Parser::convert_to_f32(self.pop()?.value)?;

        Ok(Command::Hermite { x0, y0, x1, y1, rx0, ry0, rx1, ry1 })
    }

    fn handle_bezier(&mut self) -> Result<Command, Box<dyn Error>> {
        let x0 = Parser::convert_to_f32(self.pop()?.value)?;
        let y0 = Parser::convert_to_f32(self.pop()?.value)?;
        let x1 = Parser::convert_to_f32(self.pop()?.value)?;
        let y1 = Parser::convert_to_f32(self.pop()?.value)?;
        let x2 = Parser::convert_to_f32(self.pop()?.value)?;
        let y2 = Parser::convert_to_f32(self.pop()?.value)?;
        let x3 = Parser::convert_to_f32(self.pop()?.value)?;
        let y3 = Parser::convert_to_f32(self.pop()?.value)?;

        Ok(Command::Bezier { x0, y0, x1, y1, x2, y2, x3, y3 })
    }

    fn handle_polygon(&mut self) -> Result<Command, Box<dyn Error>> {
        let x0 = Parser::convert_to_f32(self.pop()?.value)?;
        let y0 = Parser::convert_to_f32(self.pop()?.value)?;
        let z0 = Parser::convert_to_f32(self.pop()?.value)?;
        let x1 = Parser::convert_to_f32(self.pop()?.value)?;
        let y1 = Parser::convert_to_f32(self.pop()?.value)?;
        let z1 = Parser::convert_to_f32(self.pop()?.value)?;
        let x2 = Parser::convert_to_f32(self.pop()?.value)?;
        let y2 = Parser::convert_to_f32(self.pop()?.value)?;
        let z2 = Parser::convert_to_f32(self.pop()?.value)?;

        Ok(Command::Polygon { x0, y0, z0, x1, y1, z1, x2, y2, z2 })
    }

    fn handle_box(&mut self) -> Result<Command, Box<dyn Error>> {
        let constants = self.pop_optional_type(TokenType::Identifier);
        let x = Parser::convert_to_f32(self.pop()?.value)?;
        let y = Parser::convert_to_f32(self.pop()?.value)?;
        let z = Parser::convert_to_f32(self.pop()?.value)?;
        let w = Parser::convert_to_f32(self.pop()?.value)?;
        let h = Parser::convert_to_f32(self.pop()?.value)?;
        let d = Parser::convert_to_f32(self.pop()?.value)?;
        let coord_system = self.pop_optional_type(TokenType::Identifier);

        Ok(Command::Box { constants, x, y, z, w, h, d, coord_system })
    }

    fn handle_sphere(&mut self) -> Result<Command, Box<dyn Error>> {
        let constants = self.pop_optional_type(TokenType::Identifier);
        let x = Parser::convert_to_f32(self.pop()?.value)?;
        let y = Parser::convert_to_f32(self.pop()?.value)?;
        let z = Parser::convert_to_f32(self.pop()?.value)?;
        let r = Parser::convert_to_f32(self.pop()?.value)?;
        let coord_system = self.pop_optional_type(TokenType::Identifier);

        Ok(Command::Sphere { constants, x, y, z, r, coord_system })
    }

    fn handle_torus(&mut self) -> Result<Command, Box<dyn Error>> {
        let constants = self.pop_optional_type(TokenType::Identifier);
        let x = Parser::convert_to_f32(self.pop()?.value)?;
        let y = Parser::convert_to_f32(self.pop()?.value)?;
        let z = Parser::convert_to_f32(self.pop()?.value)?;
        let r0 = Parser::convert_to_f32(self.pop()?.value)?;
        let r1 = Parser::convert_to_f32(self.pop()?.value)?;
        let coord_system = self.pop_optional_type(TokenType::Identifier);

        Ok(Command::Torus { constants, x, y, z, r0, r1, coord_system })
    }

    fn handle_cylinder(&mut self) -> Result<Command, Box<dyn Error>> {
        let constants = self.pop_optional_type(TokenType::Identifier);
        let x = Parser::convert_to_f32(self.pop()?.value)?;
        let y = Parser::convert_to_f32(self.pop()?.value)?;
        let z = Parser::convert_to_f32(self.pop()?.value)?;
        let r = Parser::convert_to_f32(self.pop()?.value)?;
        let h = Parser::convert_to_f32(self.pop()?.value)?;
        let coord_system = self.pop_optional_type(TokenType::Identifier);

        Ok(Command::Cylinder { constants, x, y, z, r, h, coord_system })
    }

    fn handle_cone(&mut self) -> Result<Command, Box<dyn Error>> {
        let constants = self.pop_optional_type(TokenType::Identifier);
        let x = Parser::convert_to_f32(self.pop()?.value)?;
        let y = Parser::convert_to_f32(self.pop()?.value)?;
        let z = Parser::convert_to_f32(self.pop()?.value)?;
        let r = Parser::convert_to_f32(self.pop()?.value)?;
        let h = Parser::convert_to_f32(self.pop()?.value)?;
        let coord_system = self.pop_optional_type(TokenType::Identifier);

        Ok(Command::Cone { constants, x, y, z, r, h, coord_system })
    }

    fn handle_mesh(&mut self) -> Result<Command, Box<dyn Error>> {
        let constants = self.pop_optional_type(TokenType::Identifier);
        let file_path = self.pop()?.value;
        let coord_system = self.pop_optional_type(TokenType::Identifier);

        Ok(Command::Mesh { constants, file_path, coord_system }) 
    }

    fn handle_add_light(&mut self) -> Result<Command, Box<dyn Error>> {
        let r = Parser::convert_to_f32(self.pop()?.value)?;
        let g = Parser::convert_to_f32(self.pop()?.value)?;
        let b = Parser::convert_to_f32(self.pop()?.value)?;
        let x = Parser::convert_to_f32(self.pop()?.value)?;
        let y = Parser::convert_to_f32(self.pop()?.value)?;
        let z = Parser::convert_to_f32(self.pop()?.value)?;

        Ok(Command::AddLight { r, g, b, x, y, z })
    }

    fn handle_set_ambient(&mut self) -> Result<Command, Box<dyn Error>> {
        let r = Parser::convert_to_f32(self.pop()?.value)?;
        let g = Parser::convert_to_f32(self.pop()?.value)?;
        let b = Parser::convert_to_f32(self.pop()?.value)?;

        Ok(Command::SetAmbient { r, g, b })
    }

    fn handle_define_constants(&mut self) -> Result<Command, Box<dyn Error>> {
        let name = self.pop()?.value;
        let kar = Parser::convert_to_f32(self.pop()?.value)?;
        let kdr = Parser::convert_to_f32(self.pop()?.value)?;
        let ksr = Parser::convert_to_f32(self.pop()?.value)?;
        let kag = Parser::convert_to_f32(self.pop()?.value)?;
        let kdg = Parser::convert_to_f32(self.pop()?.value)?;
        let ksg = Parser::convert_to_f32(self.pop()?.value)?;
        let kab = Parser::convert_to_f32(self.pop()?.value)?;
        let kdb = Parser::convert_to_f32(self.pop()?.value)?;
        let ksb = Parser::convert_to_f32(self.pop()?.value)?;
        let _ = self.pop_optional_type(TokenType::Number); // r intensity
        let _ = self.pop_optional_type(TokenType::Number); // g intensity
        let _ = self.pop_optional_type(TokenType::Number); // b intensity

        Ok(Command::DefineConstants { name, kar, kdr, ksr, kag, kdg, ksg, kab, kdb, ksb })
    }

    fn handle_set_shading(&mut self) -> Result<Command, Box<dyn Error>> {
        let mode_str = self.pop()?.value.to_lowercase();
        let shading_mode = match mode_str.as_str() {
            "wireframe" => ShadingMode::Wireframe,
            "flat" => ShadingMode::Flat,
            "gouraud" => ShadingMode::Gouraud,
            "phong" => ShadingMode::Phong,
            "raytrace" => { println!("Raytracing shading is not supported. Using flat shading by default."); ShadingMode::Flat }
            _ => return Err(format!("Invalid shading mode: {}", mode_str).into()),
        };

        Ok(Command::SetShading { shading_mode })
    }

    fn handle_set_camera(&mut self) -> Result<Command, Box<dyn Error>> {
        let eye_x = Parser::convert_to_f32(self.pop()?.value)?;
        let eye_y = Parser::convert_to_f32(self.pop()?.value)?;
        let eye_z = Parser::convert_to_f32(self.pop()?.value)?;
        let aim_x = Parser::convert_to_f32(self.pop()?.value)?;
        let aim_y = Parser::convert_to_f32(self.pop()?.value)?;
        let aim_z = Parser::convert_to_f32(self.pop()?.value)?;

        Ok(Command::SetCamera { eye_x, eye_y, eye_z, aim_x, aim_y, aim_z })
    }

    fn handle_set_base_name(&mut self) -> Result<Command, Box<dyn Error>> {
        let name = self.pop()?.value;

        Ok(Command::SetBaseName { name })
    }

    fn handle_set_knob(&mut self) -> Result<Command, Box<dyn Error>> {
        let name = self.pop()?.value;
        let value = Parser::convert_to_f32(self.pop()?.value)?;

        Ok(Command::SetKnob { name, value })
    }

    fn handle_save_knob_list(&mut self) -> Result<Command, Box<dyn Error>> {
        let name = self.pop()?.value;

        Ok(Command::SaveKnobList { name })
    }

    fn handle_tween(&mut self) -> Result<Command, Box<dyn Error>> {
        let start_frame = Parser::convert_to_usize(self.pop()?.value)?;
        let end_frame = Parser::convert_to_usize(self.pop()?.value)?;
        let knoblist0 = self.pop()?.value;
        let knoblist1 = self.pop()?.value;
        let easing = self.pop_optional_type(TokenType::EasingFunction);

        Ok(Command::Tween { start_frame, end_frame, knoblist0, knoblist1, easing })
    }

    fn handle_set_frames(&mut self) -> Result<Command, Box<dyn Error>> {
        let num_frames = Parser::convert_to_usize(self.pop()?.value)?;

        Ok(Command::SetFrames { num_frames })
    }

    fn handle_vary_knob(&mut self) -> Result<Command, Box<dyn Error>> {
        let knob = self.pop()?.value;
        let start_frame = Parser::convert_to_usize(self.pop()?.value)?;
        let end_frame = Parser::convert_to_usize(self.pop()?.value)?;
        let start_val = Parser::convert_to_f32(self.pop()?.value)?;
        let end_val = Parser::convert_to_f32(self.pop()?.value)?;
        let easing = self.pop_optional_type(TokenType::EasingFunction);

        Ok(Command::VaryKnob { knob, start_frame, end_frame, start_val, end_val, easing })
    }

    fn handle_set_all_knobs(&mut self) -> Result<Command, Box<dyn Error>> {
        let value = Parser::convert_to_f32(self.pop()?.value)?;

        Ok(Command::SetAllKnobs { value })
    }

    fn handle_save_coord_system(&mut self) -> Result<Command, Box<dyn Error>> {
        let name = self.pop()?.value;

        Ok(Command::SaveCoordSystem { name })
    }

    fn handle_set_focal_length(&mut self) -> Result<Command, Box<dyn Error>> {
        let length = Parser::convert_to_f32(self.pop()?.value)?;

        Ok(Command::SetFocalLength { length })
    }

    fn convert_to_f32(parameter: String) -> Result<f32, Box<dyn Error>> {
        parameter.parse::<f32>().map_err(|_| format!("Error parsing f32: {}", parameter).into())
    }

    fn convert_to_usize(parameter: String) -> Result<usize, Box<dyn Error>> {
        parameter.parse::<usize>().map_err(|_| format!("Error parsing usize: {}", parameter).into())
    }
}
