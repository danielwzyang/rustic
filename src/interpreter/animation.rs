use super::parser::Command;
use std::{
    sync::LazyLock,
    error::Error,
    collections::HashMap,
    fs::File
};
use image::{ImageBuffer, RgbaImage, Frame, Delay, codecs::gif::{GifEncoder, Repeat}};
use crate::{
    constants::BEZIER,
    matrix::multiply,
};

struct CubicBezierEasing {
    cx: [f32; 4],
    cy: [f32; 4],
}

impl CubicBezierEasing {
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        let mut g = vec![
            [0.0, x1, x2, 1.0],
            [0.0, y1, y2, 1.0],
        ];

        multiply(&BEZIER, &mut g);

        Self {
            cx: g[0],
            cy: g[1],
        }
    }

    fn plug(c: &[f32; 4], t: f32) -> f32 {
        ((c[0] * t + c[1]) * t + c[2]) * t + c[3]
    }

    pub fn eval(&self, x: f32) -> f32 {
        // basically find a value t such that the bezier_x(t) = x and return the y coord
        // use newtons method!!
        let mut t = x.clamp(0.0, 1.0);

        // 5 iterations
        for _ in 0..5 {
            let x_t = Self::plug(&self.cx, t);

            let dx = (3.0*self.cx[0]*t + 2.0*self.cx[1])*t + self.cx[2];

            t -= (x_t - x) / dx;
            t = t.clamp(0.0, 1.0);
        }

        Self::plug(&self.cy, t)
    }
}

static EASING_FUNCTIONS: LazyLock<HashMap<&str, CubicBezierEasing>> = LazyLock::new(|| {
    let mut map = HashMap::new();

    map.insert("easeInCubic", CubicBezierEasing::new(0.33, 0.0, 0.68, 1.0));
    map.insert("easeOutCubic", CubicBezierEasing::new(0.33, 1.0, 0.68, 1.0));
    map.insert("easeInExpo", CubicBezierEasing::new(0.7, 0.0, 0.84, 0.0));
    map.insert("easeOutExpo", CubicBezierEasing::new(0.16, 1.0, 0.3, 1.0));

    map
});

pub fn first_pass(commands: &Vec<Command>) -> Result<(usize, String), Box<dyn Error>> {
    let mut frames: usize = 0;
    let mut basename = String::new();

    let mut contains_frames = false;
    let mut contains_vary = false;
    let mut contains_basename = false;
    let mut contains_tween = false;

    for command in commands {
        match command {
            Command::SetBaseName { name } => { basename = name.clone(); contains_basename = true; }
            Command::Tween { .. } => { contains_tween = true; }
            Command::SetFrames { num_frames } => { frames = *num_frames; contains_frames = true; }
            Command::VaryKnob { .. } => { contains_vary = true; }
            _ => {}
        }
    }

    if (contains_vary || contains_tween || contains_basename) && !contains_frames {
        Err("Animation was detected but the number of frames wasn't set.".into())
    } else if contains_frames && !contains_basename {
        println!("Number of frames was set but basename wasn't. 'frame' was chosen by default.");
        Ok((frames, String::from("frame")))
    } else {
        Ok((frames, basename))
    }
}

pub fn second_pass(commands: &Vec<Command>, frames: &usize) -> Result<Vec<HashMap<String, f32>>, Box<dyn Error>> {
    let mut frame_knobs: Vec<HashMap<String, f32>> = vec![HashMap::new(); *frames];
    let mut saved_knobs: HashMap<String, HashMap<String, f32>> = HashMap::new();

    for command in commands {
        match command {
            Command::VaryKnob { knob, start_frame, end_frame, start_val, end_val, easing } => {
                if *start_frame >= *frames || *end_frame >= *frames {
                    return Err(format!("Vary command has frames outside range: {} to {}.", start_frame, end_frame).into());
                }
                
                if start_frame > end_frame {
                    return Err(format!("Vary command has start_frame > end_frame: {} > {}.", start_frame, end_frame).into());
                }

                let num_frames = (end_frame - start_frame) as f32;
                let delta = (end_val - start_val) / num_frames;

                for frame in *start_frame..=*end_frame {
                    let x = start_val + delta * ((frame - start_frame) as f32);
                    
                    if let Some(easing) = easing {
                        if let Some(func) = EASING_FUNCTIONS.get(easing.as_str()) {
                            frame_knobs[frame].insert(knob.clone(), func.eval(x));
                        } else {
                            return Err(format!("Easing function {} not recognized.", easing).into());
                        }
                    } else {
                        frame_knobs[frame].insert(knob.clone(), x);
                    }
                }
            }

            Command::SaveKnobList { name } => {
                if !frame_knobs.is_empty() {
                    saved_knobs.insert(name.clone(), frame_knobs[0].clone());
                }
            }

            Command::Tween { start_frame, end_frame, knoblist0, knoblist1, easing } => {
                if *start_frame >= *frames || *end_frame >= *frames {
                    return Err(format!("Tween command has frames outside range: {} to {}.", start_frame, end_frame).into());
                }
                
                if start_frame > end_frame {
                    return Err(format!("Tween command has start_frame > end_frame: {} > {}.", start_frame, end_frame).into());
                }

                let knobs0 = saved_knobs.get(knoblist0)
                    .ok_or_else(|| format!("Knoblist '{}' not found", knoblist0))?;
                let knobs1 = saved_knobs.get(knoblist1)
                    .ok_or_else(|| format!("Knoblist '{}' not found", knoblist1))?;

                let num_frames = (end_frame - start_frame) as f32;

                let mut all_knobs: std::collections::HashSet<String> = std::collections::HashSet::new();
                for knob in knobs0.keys() {
                    all_knobs.insert(knob.clone());
                }
                for knob in knobs1.keys() {
                    all_knobs.insert(knob.clone());
                }

                for knob in all_knobs {
                    let start_val = *knobs0.get(&knob).unwrap_or(&0.0);
                    let end_val = *knobs1.get(&knob).unwrap_or(&0.0);
                    let delta = (end_val - start_val) / num_frames;
                    for frame in *start_frame..=*end_frame {
                        let x = start_val + delta * ((frame - start_frame) as f32);
                        
                        if let Some(easing) = easing {
                            if let Some(func) = EASING_FUNCTIONS.get(easing.as_str()) {
                                frame_knobs[frame].insert(knob.clone(), func.eval(x));
                            } else {
                                return Err(format!("Easing function {} not recognized.", easing).into());
                            }
                        } else {
                            frame_knobs[frame].insert(knob.clone(), x);
                        }
                    }
                }
            }

            _ => {}
        }
    }

    Ok(frame_knobs)
}

pub struct Animation {
    pub frames: Vec<Vec<u8>>,
    pub width: usize,
    pub height: usize,
}

impl Animation {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            frames: Vec::new(),
            width,
            height,
        }
    }

    pub fn add_frame(&mut self, rgb_buffer: &Vec<u8>) {
        self.frames.push(rgb_buffer.clone());
    }

    pub fn save_as_file(&self, filename: String, delay: u32) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(&filename)?;
        let mut encoder = GifEncoder::new(file);
        encoder.set_repeat(Repeat::Infinite)?;

        for frame_data in &self.frames {
            let mut rgba = Vec::with_capacity(self.width * self.height * 4);

            // convert to rgba because the Frame struct expects rgba
            for chunk in frame_data.chunks(3) {
                rgba.push(chunk[0]);
                rgba.push(chunk[1]);
                rgba.push(chunk[2]);
                rgba.push(255);
            }

            let img: RgbaImage = ImageBuffer::from_raw(
                self.width as u32,
                self.height as u32,
                rgba,
            ).ok_or("Failed to create RGBA frame")?;

            let frame = Frame::from_parts(img, 0, 0, Delay::from_numer_denom_ms(delay, 1));
            encoder.encode_frame(frame)?;
        }

        println!("{} created.", filename);

        Ok(())
    }
}
