mod constants;
mod matrix;
mod picture;
mod render;
mod interpreter;
mod vector;

use std::{error::Error, env};
#[show_image::main]
fn main() -> Result<(), Box<dyn Error>> {
    let arguments: Vec<String> = env::args().collect();

    if arguments.len() < 2 {
        println!("A path to a script wasn't provided. '{}' was chosen by default.", &constants::DEFAULT_SCRIPT);
    } else {
        for path in &arguments[1..] {
            println!("Running script '{}'.", path);
            interpreter::run_script(path)?;
        }
    }

    Ok(())
}
