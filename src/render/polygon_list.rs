use rand::Rng;

type PolygonList = Vec<[f32; 4]>;
type Vector = [f32; 3];

use std::{
    f32::consts::PI,
    collections::HashMap,
};

use crate::{
    constants::{CUBE, ENABLE_BACK_FACE_CULLING, PARAMETRIC_STEPS, ShadingMode},
    matrix::add_point,
    vector::{add_vectors, cross_product, normalize_vector}
};
use super::{
    scan_line,
    Picture, LightingConfig, ReflectionConstants, get_illumination,
};

fn vector_to_key(vector: &[f32; 4]) -> (isize, isize, isize) {
    (vector[0].round() as isize, vector[1].round() as isize, vector[2].round() as isize)
}

pub fn add_polygon(m: &mut PolygonList, x0: f32, y0: f32, z0: f32, x1: f32, y1: f32, z1: f32, x2: f32, y2: f32, z2: f32) {
    add_point(m, x0, y0, z0, 1.0);
    add_point(m, x1, y1, z1, 1.0);
    add_point(m, x2, y2, z2, 1.0);
}

pub fn render_polygons(
    m: &PolygonList, picture: &mut Picture, color: &(usize, usize, usize),
    shading_mode: &ShadingMode, lighting_config: &LightingConfig, reflection_constants: &ReflectionConstants
) {
    // for gouraud and phong shading
    // we need to keep a hash to get the average normal for every polygon that contains this vertex
    // instead of getting averages we can sum up all the vectors and then normalize it at the end
    // we need them to be normalized for lighting anyway
    let mut vertex_normals: HashMap<(isize, isize, isize), Vector> = HashMap::new();

    match shading_mode {
        ShadingMode::Gouraud | ShadingMode::Phong => {
            for polygon in m.chunks(3) {
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

                // calculate the normal for backface culling using the cross product of two edges
                let normal = cross_product(&a, &b);

                for vertex in polygon {
                    let entry = vertex_normals.entry(vector_to_key(&vertex)).or_insert([0.0, 0.0, 0.0]);

                    *entry = add_vectors(&entry, &normal);
                }

                for normal in vertex_normals.values_mut() {
                    *normal = normalize_vector(normal);
                }
            }
        }
        _ => {}
    }

    for polygon in m.chunks(3) {
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

        // calculate the normal for backface culling using the cross product of two edges
        // normal = < aybz - azby, azbx - axbz, axby - aybx >
        let normal: Vector = [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ];

        /*
            if the angle between the normal and the viewer is between -90 and 90, the polygon is facing the viewer
            we can find the angle between the normal and the viewer using this formula
            |n||v|cos(theta) = dot product of n and v
            we can use the fact that cos() will be (+) for the angle we need
            |n||v| will always be (+) so we can just see if the dot product of n and v is (+) to see if cos is (+)
            we will set v to <0, 0, 1> so the magnitude and dot products are easy to compute
            the dot product of n and v is just the z component of n
        */

        if normal[2] > 0.0 && ENABLE_BACK_FACE_CULLING {
            match shading_mode {
                ShadingMode::Wireframe => {
                    picture.draw_line(
                        polygon[0][0] as isize, polygon[0][1] as isize, polygon[0][2],
                        polygon[1][0] as isize, polygon[1][1] as isize, polygon[1][2],
                        &color,
                    );
                    picture.draw_line(
                        polygon[2][0] as isize, polygon[2][1] as isize, polygon[2][2],
                        polygon[1][0] as isize, polygon[1][1] as isize, polygon[1][2],
                        &color,
                    );
                    picture.draw_line(
                        polygon[0][0] as isize, polygon[0][1] as isize, polygon[0][2],
                        polygon[2][0] as isize, polygon[2][1] as isize, polygon[2][2],
                        &color,
                    );
                },
                ShadingMode::FlatRandom => {
                    let mut rng = rand::rng();
                    scan_line::flat(
                        picture,
                        polygon,
                        &(rng.random::<u8>() as usize, rng.random::<u8>() as usize, rng.random::<u8>() as usize)
                    );
                },
                ShadingMode::Flat => {
                    scan_line::flat(
                        picture,
                        polygon,
                        &get_illumination(&normalize_vector(&normal), lighting_config, reflection_constants)
                    );
                },
                ShadingMode::Gouraud => {
                    let normals = [
                        *vertex_normals.get(&vector_to_key(&polygon[0])).unwrap(),
                        *vertex_normals.get(&vector_to_key(&polygon[1])).unwrap(),
                        *vertex_normals.get(&vector_to_key(&polygon[2])).unwrap(),
                    ];

                    scan_line::gouraud(picture, polygon, normals, lighting_config, reflection_constants);
                }
                ShadingMode::Phong => {
                    let normals = [
                        *vertex_normals.get(&vector_to_key(&polygon[0])).unwrap(),
                        *vertex_normals.get(&vector_to_key(&polygon[1])).unwrap(),
                        *vertex_normals.get(&vector_to_key(&polygon[2])).unwrap(),
                    ];

                    scan_line::phong(picture, polygon, normals, lighting_config, reflection_constants);
                }
            }
        }
    }
}

pub fn add_box(m: &mut PolygonList, x: f32, y: f32, z: f32, w: f32, h: f32, d: f32) {
    /*
        4 ---- 5
      / |    / |
    0 ---- 1   | h
    |   |  |   |
    |   7 -|-- 6
    | /    | /  d
    3 ---- 2
       w
    */

    let vertices = [
        [x, y, z],
        [x + w, y, z],
        [x + w, y - h, z],
        [x, y - h, z],
        [x, y, z - d],
        [x + w, y, z - d],
        [x + w, y - h, z - d],
        [x, y - h, z - d],
    ];

    for polygon in CUBE {
        let (a, b, c) = polygon;
        add_polygon(m,
            vertices[a][0], vertices[a][1], vertices[a][2],
            vertices[b][0], vertices[b][1], vertices[b][2],
            vertices[c][0], vertices[c][1], vertices[c][2],
        );
    }
}

pub fn add_sphere(m: &mut PolygonList, cx: f32, cy: f32, cz: f32, r: f32) {
    let points = generate_sphere_points(cx, cy, cz, r);

    // we do PARAMETRIC_STEPS + 1 because the semicircle generates one extra point for the south pole the way I coded it
    // e.g. a PARAMETRIC_STEPS of 10 results in 11 points per semicircle

    let get = |longitude: i32, latitude: i32| -> Vector {
        points[(longitude * (PARAMETRIC_STEPS + 1) + latitude) as usize]
    };

    for longitude in 0..PARAMETRIC_STEPS {
        let next = if longitude == PARAMETRIC_STEPS { 0 } else { longitude + 1 };
        // this is for all the polygons that aren't on the poles
        for latitude in 1..PARAMETRIC_STEPS-1 {
            let p1 = get(longitude, latitude);
            let p2 = get(longitude, latitude + 1);
            let p1_across = get(next, latitude);
            let p2_across = get(next, latitude + 1);

            // p1, p2, p2_across
            add_polygon(m,
                p1[0], p1[1], p1[2],
                p2[0], p2[1], p2[2],
                p2_across[0], p2_across[1], p2_across[2],
            );

            // p1, p2_across, p1_across
            add_polygon(m,
                p1[0], p1[1], p1[2],
                p2_across[0], p2_across[1], p2_across[2],
                p1_across[0], p1_across[1], p1_across[2],
            );
        }
        // two triangles at the poles

        // pole, p1, p1_across
        let pole = get(longitude, 0);
        let p = get(longitude, 1);
        let p_across = get(next, 1);
        add_polygon(m,
            pole[0], pole[1], pole[2],
            p[0], p[1], p[2],
            p_across[0], p_across[1], p_across[2],
        );

        // pole, pminus1_across, pminus1
        let pole = get(longitude, PARAMETRIC_STEPS);
        let p = get(longitude, PARAMETRIC_STEPS - 1);
        let p_across = get(next, PARAMETRIC_STEPS - 1);
        add_polygon(m,
            pole[0], pole[1], pole[2],
            p_across[0], p_across[1], p_across[2],
            p[0], p[1], p[2],
        );
    }
}

fn generate_sphere_points(cx: f32, cy: f32, cz: f32, r: f32) -> Vec<Vector> {
    // not using run_parametric because this parametric is nested but the logic is the same
    let x = |cir: f32| r * (PI * cir).cos() + cx;
    let y = |rot: f32, cir: f32| r * (PI * cir).sin() * (2.0 * PI * rot).cos() + cy;
    let z = |rot: f32, cir: f32| r * (PI * cir).sin() * (2.0 * PI * rot).sin() + cz;

    let mut point_list: Vec<Vector> = vec![];

    for i in 0..=PARAMETRIC_STEPS {
        let rot = i as f32 / PARAMETRIC_STEPS as f32;
        for j in 0..=PARAMETRIC_STEPS {
            let cir = j as f32 / PARAMETRIC_STEPS as f32;
            point_list.push([x(cir), y(rot, cir), z(rot, cir)]);
        }
    }

    point_list
}

pub fn add_torus(m: &mut PolygonList, cx: f32, cy: f32, cz: f32, r1: f32, r2: f32) {
    let points = generate_torus_points(cx, cy, cz, r1, r2);
    // around is which circle of the torus we're currently on
    // on is which part of the circle we're currently on
    // kind of silly names but longitude and latitude didn't make sense so i had to freestyle it
    // for the torus we can just use PARAMETRIC_STEPS i.e. PARAMETRIC_STEPS of 10 gives 10 points on each circle
    let get = |around: i32, on: i32| -> Vector {
        points[(around * (PARAMETRIC_STEPS + 1) + on) as usize]
    };

    for around in 0..PARAMETRIC_STEPS {
        let next = if around == PARAMETRIC_STEPS { 0 } else { around + 1 };
        for on in 0..PARAMETRIC_STEPS {
            let p1 = get(around, on);
            let p2 = get(around, on + 1);
            let p1_across = get(next, on);
            let p2_across = get(next, on + 1);

            // p1, p2_across, p2
            add_polygon(m,
                p1[0], p1[1], p1[2],
                p2_across[0], p2_across[1], p2_across[2],
                p2[0], p2[1], p2[2],
            );

            // p1, p1_across, p2_across
            add_polygon(m,
                p1[0], p1[1], p1[2],
                p1_across[0], p1_across[1], p1_across[2],
                p2_across[0], p2_across[1], p2_across[2],
            );
        }
    }
}

fn generate_torus_points(cx: f32, cy: f32, cz: f32, r1: f32, r2: f32) -> Vec<Vector> {
    // r1 is the radius of the circle that makes up the torus
    // r2 is the radius of the entire torus (translation factor)
    let x = |rot: f32, cir: f32| (2.0 * PI * rot).cos() * (r1 * (2.0 * PI * cir).cos() + r2) + cx;
    let y = |cir: f32| r1 * (2.0 * PI * cir).sin() + cy;
    let z = |rot: f32, cir: f32| -1.0 * (2.0 * PI * rot).sin() * (r1 * (2.0 * PI * cir).cos() + r2) + cz;

    let mut point_list: Vec<Vector> = vec![];

    for i in 0..=PARAMETRIC_STEPS {
        let rot = i as f32 / PARAMETRIC_STEPS as f32;
        for j in 0..=PARAMETRIC_STEPS {
            let cir = j as f32 / PARAMETRIC_STEPS as f32;
            point_list.push([x(rot, cir), y(cir), z(rot, cir)]);
        }
    }

    point_list
}

pub fn add_cylinder(m: &mut PolygonList, cx: f32, cy: f32, cz: f32, r: f32, h: f32) {
    let points = generate_cylinder_points(cx, cy, cz, r, h);
    let length = (PARAMETRIC_STEPS * 2) as usize;

    for i in 0..length {
        if i % 2 == 0 {
            // bottom and sides
            /*
                3   1


                2   0   
            */

            let p0 = points[i];
            let p1 = points[((i + 1) % length) as usize];
            let p2 = points[((i + 2) % length) as usize];
            let p3 = points[((i + 3) % length) as usize];

            // 0 1 2
            add_polygon(m,
                p0[0], p0[1], p0[2],
                p1[0], p1[1], p1[2],
                p2[0], p2[1], p2[2],
            );

            // 1 3 2
            add_polygon(m,
                p1[0], p1[1], p1[2],
                p3[0], p3[1], p3[2],
                p2[0], p2[1], p2[2],
            );

            // bottom from top view
            /*
                C
                
                    0
                2
            */

            // ccw from bottom 0 2 C
            add_polygon(m,
                p0[0], p0[1], p0[2],
                p2[0], p2[1], p2[2],
                cx, cy - h, cz,
            );
        } else {
            // top
            let p1 = points[i];
            let p3 = points[((i + 2) % length) as usize];

            // top from top view
            /*
                C
                
                    1
                3
            */

            add_polygon(m,
                p1[0], p1[1], p1[2],
                cx, cy, cz,
                p3[0], p3[1], p3[2],
            );
        }
    }
} 

fn generate_cylinder_points(cx: f32, cy: f32, cz: f32, r: f32, h: f32) -> Vec<Vector> {
    // x(t) = rcos(2 * pi * t) + cx
    // z(t) = rsin(2 * pi * t) + cz
    let x = |t: f32| r * (2.0 * PI * t).cos() + cx;
    let z = |t: f32| r * (2.0 * PI * t).sin() + cz;

    let mut point_list: Vec<Vector> = vec![];

    for i in 0..PARAMETRIC_STEPS {
        let t = i as f32 / PARAMETRIC_STEPS as f32;

        point_list.push([x(t), cy - h, z(t)]);
        point_list.push([x(t), cy, z(t)]);
    }

    point_list
}

pub fn add_cone(m: &mut PolygonList, cx: f32, cy: f32, cz: f32, r: f32, h: f32) {
    let points = generate_cone_points(cx, cy, cz, r);
    let length = PARAMETRIC_STEPS as usize;
    
    for i in 0..length {
        let p0 = points[i];
        let p1 = points[(i + 1) % length];
        
        // triangle going to top
        // 0 Ct 1
        add_polygon(m,
            p0[0], p0[1], p0[2],
            cx, cy + h, cz,
            p1[0], p1[1], p1[2],
        );

        // bottom
        // 0 1 Cb
        add_polygon(m,
            p0[0], p0[1], p0[2],
            p1[0], p1[1], p1[2],
            cx, cy, cz,
        );
    }
}

fn generate_cone_points(cx: f32, cy: f32, cz: f32, r: f32) -> Vec<Vector> {
    // x(t) = rcos(2 * pi * t) + cx
    // z(t) = rsin(2 * pi * t) + cz
    let x = |t: f32| r * (2.0 * PI * t).cos() + cx;
    let z = |t: f32| r * (2.0 * PI * t).sin() + cz;
    
    let mut point_list: Vec<Vector> = vec![];

    for i in 0..PARAMETRIC_STEPS {
        let t = i as f32 / PARAMETRIC_STEPS as f32;

        point_list.push([x(t), cy, z(t)]);
    }

    point_list
}
