use super::read_lines;
use image::ImageReader;
use stl_io::read_stl;
use crate::{render::{polygon_list::add_polygon, texture::{MTL, render_textured_polygon}, LightingConfig}, picture::Picture};
use std::{
    collections::HashMap, error::Error, fs::OpenOptions, path::{Path, PathBuf}
};

type Matrix = Vec<[f32; 4]>;

pub fn handle_mesh(
    polygons: &mut Matrix,
    file_path: String,
    lighting_config: &LightingConfig
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
            if line.starts_with("//") || line.starts_with("#") || line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();

            match parts[0] {
                "mtllib" => mtl_path = Some(parts[1].to_string()),
                "usemtl" => current_mtl = parts[1].to_string(),
                "v" => vertices.push([parts[1].parse()?, parts[2].parse()?, parts[3].parse()?]),
                "vt" => vertex_textures.push([parts[1].parse()?, parts[2].parse()?]),
                "f" => {
                    let parse_v = |s: &str| s.split('/').next().unwrap().parse::<usize>().unwrap() - 1;

                    let v0 = parse_v(parts[1]);
                    let v1 = parse_v(parts[2]);
                    let v2 = parse_v(parts[3]);

                    add_polygon(
                        polygons,
                        vertices[v0][0], vertices[v0][1], vertices[v0][2],
                        vertices[v1][0], vertices[v1][1], vertices[v1][2],
                        vertices[v2][0], vertices[v2][1], vertices[v2][2],
                    );

                    let is_quad = parts.len() == 5;
                    if is_quad {
                        let v3 = parse_v(parts[4]);
                        add_polygon(
                            polygons,
                            vertices[v0][0], vertices[v0][1], vertices[v0][2],
                            vertices[v2][0], vertices[v2][1], vertices[v2][2],
                            vertices[v3][0], vertices[v3][1], vertices[v3][2],
                        );
                    }

                    // Only parse vt indices if MTL is being used
                    if mtl_path.is_some() {
                        let parse_vt = |s: &str| s.split('/').nth(1).unwrap().parse::<usize>().unwrap() - 1;

                        let vt0 = parse_vt(parts[1]);
                        let vt1 = parse_vt(parts[2]);
                        let vt2 = parse_vt(parts[3]);

                        polygon_info.push((current_mtl.clone(), [vt0, vt1, vt2]));

                        if is_quad {
                            let vt3 = parse_vt(parts[4]);
                            polygon_info.push((current_mtl.clone(), [vt0, vt2, vt3]));
                        }
                    }
                }

                _ => {}
            }
        }

        // if mtl is enabled render here otherwise allow the handle_mesh function that calls this
        if let Some(mtl_path) = mtl_path {
            let mtls = parse_mtl_from_obj(file, mtl_path.as_str())?;

            let mut polygon_index = 0;

            for (mtl, [vt0, vt1, vt2]) in polygon_info.iter() {
                let triangle_slice: &[[f32; 4]; 3] = polygons[polygon_index..polygon_index + 3]
                    .try_into()
                    .expect("Polygon slice must be exactly 3 vertices");

                render_textured_polygon(
                    triangle_slice,
                    [vertex_textures[*vt0], vertex_textures[*vt1], vertex_textures[*vt2]],
                    mtls.get(mtl).unwrap(),
                    &lighting_config.point_light_vector,
                );

                polygon_index += 3; // move to next triangle
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

pub fn parse_mtl_from_obj(obj_path: &Path, mtl_relative_path: &str) -> Result<HashMap<String, MTL>, Box<dyn Error>> {
    let obj_dir = obj_path.parent().unwrap_or_else(|| Path::new("."));
    let mtl_path = obj_dir.join(mtl_relative_path);

    let mut mtls = HashMap::new();
    let mut current_name = String::new();
    let mut current_kd = (255, 255, 255);
    let mut current_texture: Option<PathBuf> = None;

    for line in read_lines(&mtl_path)?.map_while(Result::ok) {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        let parts: Vec<&str> = line.split_whitespace().collect();

        match parts[0] {
            "newmtl" => {
                if !current_name.is_empty() {
                    let mtl = load_texture(&current_texture.unwrap(), current_kd);
                    mtls.insert(current_name.clone(), mtl);
                }
                current_name = parts[1].to_string();
                current_kd = (255, 255, 255);
                current_texture = None;
            }
            "Kd" => {
                let r = (parts[1].parse::<f32>()? * 255.0) as usize;
                let g = (parts[2].parse::<f32>()? * 255.0) as usize;
                let b = (parts[3].parse::<f32>()? * 255.0) as usize;
                current_kd = (r, g, b);
            }
            "map_Kd" => {
                current_texture = Some(obj_dir.join(parts[1]));
            }
            _ => {}
        }
    }

    // save the last mtl
    if !current_name.is_empty() {
        let mtl = load_texture(&current_texture.unwrap(), current_kd);
        mtls.insert(current_name.clone(), mtl);
    }

    Ok(mtls)
}

fn load_texture(path: &Path, kd: (usize, usize, usize)) -> MTL{
    let img = ImageReader::open(path).unwrap().decode().unwrap().to_rgba8();
    let (width, height) = img.dimensions();
    MTL {
        kd,
        data: img.into_vec(),
        width: width as isize,
        height: height as isize,
    }
}
