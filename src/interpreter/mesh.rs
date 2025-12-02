use super::read_lines;
use stl_io::read_stl;
use crate::{render::{polygon_list::add_polygon, texture::{MTL, render_textured_polygon}}};
use std::{
    error::Error, path::Path, fs::OpenOptions, collections::HashMap,
};

type Matrix = Vec<[f32; 4]>;

pub fn handle_mesh(
    polygons: &mut Matrix,
    file_path: String,
) -> Result<bool, Box<dyn Error>> {
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
        return Err(format!("Mesh file extension '.{}' not supported", extension).into());
    }

    if extension == "obj" {
        let mut vertices: Vec<[f32; 3]> = vec![];
        let mut vertex_textures: Vec<[f32; 2]> = vec![];
        let mut mtl_path: Option<String> = None;
        let mut current_mtl: String = String::new(); 
        let mut polygon_info: Vec<(String, [usize; 3])> = vec![];

        for line in read_lines(&file_path)?.map_while(Result::ok) {
            let line = line.trim();
            let parts: Vec<&str> = line.split_whitespace().collect();

            match parts[0] {
                "mtllib" => mtl_path = Some(parts[1].to_string()),
                "usemtl" => current_mtl = parts[1].to_string(),
                "v" => vertices.push([parts[1].parse()?, parts[2].parse()?, parts[3].parse()?]),
                "vt" => vertex_textures.push([parts[1].parse()?, parts[2].parse()?]),
                "f" => {
                    let v0 = parts[1].split('/').next().unwrap().parse::<usize>()? - 1;
                    let v1 = parts[2].split('/').next().unwrap().parse::<usize>()? - 1;
                    let v2 = parts[3].split('/').next().unwrap().parse::<usize>()? - 1;

                    add_polygon(
                        polygons,
                        vertices[v0][0], vertices[v0][1], vertices[v0][2],
                        vertices[v1][0], vertices[v1][1], vertices[v1][2],
                        vertices[v2][0], vertices[v2][1], vertices[v2][2],
                    );

                    let is_quad = parts.len() > 4;

                    if is_quad {
                        let v3 = parts[4].split('/').next().unwrap().parse::<usize>()? - 1;

                        add_polygon(
                            polygons,
                            vertices[v0][0], vertices[v0][1], vertices[v0][2],
                            vertices[v2][0], vertices[v2][1], vertices[v2][2],
                            vertices[v3][0], vertices[v3][1], vertices[v3][2],
                        );
                    }

                    if mtl_path.is_some() {
                        if current_mtl.is_empty() {
                            return Err(format!("MTL file is used but no material selected for face in OBJ {}", file_path).into());
                        }

                        let parse_vt = |s: &str| -> Result<usize, Box<dyn Error>> {
                            let mut split = s.split('/');
                            split.next(); // skip vertex
                            match split.next() {
                                Some(t) if !t.is_empty() => Ok(t.parse::<usize>()? - 1),
                                _ => Err(format!("Face missing texture index in OBJ {}", file_path).into()),
                            }
                        };

                        let vt0 = parse_vt(parts[1])?;
                        let vt1 = parse_vt(parts[2])?;
                        let vt2 = parse_vt(parts[3])?;

                        polygon_info.push((current_mtl.clone(), [vt0, vt1, vt2]));

                        if is_quad {
                            let vt3 = parse_vt(parts[4])?;
                            
                            polygon_info.push((current_mtl.clone(), [vt0, vt2, vt3]));
                        }
                    }
                }
                _ => {}
            }
        }

        // if mtl is enabled render here otherwise allow the handle_mesh function that calls this
        if !mtl_path.is_none() {
            // mtl: kd (r g b) and texture
            let mut mtls: HashMap<String, MTL> = HashMap::new();

            // parse mtl

            for (i, polygon) in polygons.iter().enumerate() {
                let (mtl, [vt0, vt1, vt2]) = &polygon_info[i];
                render_textured_polygon(
                    polygon, 
                    [vertex_textures[*vt0], vertex_textures[*vt1], vertex_textures[*vt2]], 
                    mtls.get(mtl).unwrap()
                );
            }

            polygons.clear();

            Ok(false)
        } else {
            Ok(true)
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

        Ok(true)
    }
}
