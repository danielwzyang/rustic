use super::parser::Command;
use std::{
    error::Error,
    collections::HashMap,
    fs::File
};
use image::{ImageBuffer, RgbaImage, Frame, Delay, codecs::gif::{GifEncoder, Repeat}};

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
            Command::VaryKnob { knob, start_frame, end_frame, start_val, end_val } => {
                if *start_frame >= *frames || *end_frame >= *frames {
                    return Err(format!("Vary command has frames outside range: {} to {}.", start_frame, end_frame).into());
                }
                
                if start_frame > end_frame {
                    return Err(format!("Vary command has start_frame > end_frame: {} > {}.", start_frame, end_frame).into());
                }

                let num_frames = (end_frame - start_frame) as f32;
                let delta = (end_val - start_val) / num_frames;

                for frame in *start_frame..=*end_frame {
                    let value = start_val + delta * ((frame - start_frame) as f32);
                    frame_knobs[frame].insert(knob.clone(), value);
                }
            }

            Command::SaveKnobList { name } => {
                if !frame_knobs.is_empty() {
                    saved_knobs.insert(name.clone(), frame_knobs[0].clone());
                }
            }

            Command::Tween { start_frame, end_frame, knoblist0, knoblist1 } => {
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

                for knob_name in all_knobs {
                    let start_val = *knobs0.get(&knob_name).unwrap_or(&0.0);
                    let end_val = *knobs1.get(&knob_name).unwrap_or(&0.0);
                    let delta = (end_val - start_val) / num_frames;

                    for frame in *start_frame..=*end_frame {
                        let value = start_val + delta * ((frame - start_frame) as f32);
                        frame_knobs[frame].insert(knob_name.clone(), value);
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
