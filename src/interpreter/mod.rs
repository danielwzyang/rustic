mod lexer;
mod tokens;
mod parser;
mod run_script;
mod animation;
mod coordinate_stack;
mod mesh;

use std::{
    error::Error,
    collections::HashMap,
    sync::LazyLock,
    io::{self, BufRead},
    fs::File,
    path::Path,
};

use parser::Parser;
use run_script::evaluate_commands;
use tokens::{TokenType, Function};

static KEYWORDS: LazyLock<HashMap<&str, TokenType>> = LazyLock::new(|| {
    let mut map = HashMap::new();

    map.insert("display", TokenType::Command(Function::Display));
    map.insert("save", TokenType::Command(Function::Save));
    map.insert("clear", TokenType::Command(Function::Clear));
    map.insert("camera", TokenType::Command(Function::SetCamera));

    map.insert("push", TokenType::Command(Function::Push));
    map.insert("pop", TokenType::Command(Function::Pop));
    
    map.insert("move", TokenType::Command(Function::Move));
    map.insert("scale", TokenType::Command(Function::Scale));
    map.insert("rotate", TokenType::Command(Function::Rotate));
    map.insert("x", TokenType::AxisOfRotation);
    map.insert("y", TokenType::AxisOfRotation);
    map.insert("z", TokenType::AxisOfRotation);
    
    map.insert("line", TokenType::Command(Function::Line));
    map.insert("circle", TokenType::Command(Function::Circle));
    map.insert("hermite", TokenType::Command(Function::Hermite));
    map.insert("bezier", TokenType::Command(Function::Bezier));

    map.insert("polygon", TokenType::Command(Function::Polygon));
    map.insert("box", TokenType::Command(Function::Box));
    map.insert("sphere", TokenType::Command(Function::Sphere));
    map.insert("torus", TokenType::Command(Function::Torus));
    map.insert("mesh", TokenType::Command(Function::Mesh));

    map.insert("light", TokenType::Command(Function::AddLight));
    map.insert("clear_lights", TokenType::Command(Function::ClearLights));
    map.insert("ambient", TokenType::Command(Function::SetAmbient));
    map.insert("constants", TokenType::Command(Function::SetConstants));
    map.insert("shading", TokenType::Command(Function::SetShading));

    map.insert("basename", TokenType::Command(Function::SetBaseName));
    map.insert("set", TokenType::Command(Function::SetKnob));
    map.insert("save_knobs", TokenType::Command(Function::SaveKnobList));
    map.insert("tween", TokenType::Command(Function::Tween));
    map.insert("frames", TokenType::Command(Function::SetFrames));
    map.insert("vary", TokenType::Command(Function::VaryKnob));
    map.insert("setknobs", TokenType::Command(Function::SetAllKnobs));

    // unimplemented but recognized commands
    map.insert("save_coord_system", TokenType::Command(Function::SaveCoordSystem));
    map.insert("generate_rayfiles", TokenType::Command(Function::GenerateRayFiles));
    map.insert("focal", TokenType::Command(Function::SetFocalLength));

    map
});

pub fn run_script(path: &str) -> Result<(), Box<dyn Error>> {
    let tokens = lexer::tokenize(path, KEYWORDS.clone())?;

    let commands = Parser::new().generate_command_list(tokens)?;

    evaluate_commands(commands)?;

    Ok(())
}

fn read_lines<P>(file_path: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path> {
    let file = File::open(file_path)?;
    Ok(io::BufReader::new(file).lines())
}
