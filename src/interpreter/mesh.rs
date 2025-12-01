use super::read_lines;
use stl_io::read_stl;
use crate::render::polygon_list::add_polygon;
use std::{
    error::Error, path::Path, fs::OpenOptions,
};

type Matrix = Vec<[f32; 4]>;

pub fn handle_mesh(
    polygons: &mut Matrix,
    file_path: String,
    mtl_path: Option<String>,
) -> Result<(), Box<dyn Error>> {
    let file = Path::new(&file_path);

    if !file.exists() {
        return Err(format!("Mesh file '{}' not found", file_path).into());
    }

    let extension = file
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if extension != "obj" && extension != "stl" {
        return Err(format!("Mesh file extension '.{}' not supported", file_path).into());
    }

    if extension == "obj" {
        let mut vertices: Vec<[f32; 3]> = vec![];
        for line in read_lines(&file_path)?.map_while(Result::ok) {
            let line = line.trim();
            let parts: Vec<&str> = line.split_whitespace().collect();

            if line.starts_with("v ") {
                vertices.push([parts[1].parse::<f32>()?, parts[2].parse::<f32>()?, parts[3].parse::<f32>()?]);
            } else if line.starts_with("f ") {
                let a = parts[1].parse::<usize>()? - 1;
                let b = parts[2].parse::<usize>()? - 1;
                let c = parts[3].parse::<usize>()? - 1;

                add_polygon(
                    polygons,
                    vertices[a][0], vertices[a][1], vertices[a][2],
                    vertices[b][0], vertices[b][1], vertices[b][2],
                    vertices[c][0], vertices[c][1], vertices[c][2],
                );
            }
        }
    } else {
        // i originally had this hand parsed using ascii along with the .obj, but i wanted more flexibility and binary stls are annoying to parse
        let mut file = OpenOptions::new().read(true).open(file_path).unwrap();
        let mesh = read_stl(&mut file)?;

        for polygon in mesh.into_triangle_vec() {
            add_polygon(
                polygons,
                polygon.vertices[0][0], polygon.vertices[0][1], polygon.vertices[0][2],
                polygon.vertices[1][0], polygon.vertices[1][1], polygon.vertices[1][2],
                polygon.vertices[2][0], polygon.vertices[2][1], polygon.vertices[2][2],
            );
        }
    }

    Ok(())
}
