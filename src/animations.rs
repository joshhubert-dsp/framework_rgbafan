use framework_lib::chromium_ec::commands::RgbS;

use crate::consts::{
    N_LEDS,
    SPIN_PERIOD,
    OFF,
    RAINBOW
};

use crate::mpd_visualizer::MpdVisualizer;

pub enum Animation {
    Solid { color: RgbS },
    Blink {
        colors: Vec<RgbS>,
        current_color_index: u8,
        on: bool,
    },
    Spin { 
        colors: Vec<RgbS>,
        idx: usize,    // current idx on discrete spin
    },
    SmoothSpin {
        colors: Vec<RgbS>,
        period: u16,              // unit in ticks
        current_rotation: f32,    // unit in leds
    },
    RainbowSpin {
        idx: usize,    // current idx on rolling rainbow
    },
    // RainbowSmoothSpin {
    //     period: u16,              // unit in ticks
    //     current_rotation: f32,    // unit in leds
    // },
    Mpd {
        visualizer: MpdVisualizer,
    },
}


impl Animation {
    pub fn from_cli(modestr: &str, colors: Vec<RgbS>) -> Self {
        if colors.len() > N_LEDS {
            panic!("There can't be more colors than LEDS!")
        }
        
        match modestr {
            "solid" => {
                let color = colors
                    .first()
                    .copied()
                    .expect("Solid mode requires color");
                Animation::Solid{ color }
            },
            "blink" => {
                Animation::Blink{
                    colors,
                    current_color_index: 0,
                    on: false,
                }
            },
            "spin" => Animation::Spin { 
                colors,
                idx: 0,
            },
            "smoothspin" => Animation::SmoothSpin {
                colors,
                period: SPIN_PERIOD,
                current_rotation: 0.0,
            },
            "rainbowspin" => Animation::RainbowSpin { 
                idx: 0
            },
            // "rainbowsmoothspin" => Animation::RainbowSmoothSpin {
            //     period: SPIN_PERIOD,
            //     current_rotation: 0.0,
            // },
            "mpd" => Animation::Mpd {
                visualizer: MpdVisualizer::new(colors, SPIN_PERIOD),
            },
            _ => panic!("Unknown animation mode."),
        }
    }
    
    pub fn step_discrete_spin(leds: &mut [RgbS; N_LEDS], colors: &Vec<RgbS>, rotation_idx: &mut usize) {
        *rotation_idx = (*rotation_idx + 1) % N_LEDS;
        let color_step = colors.len() as f32 / N_LEDS as f32;
    
        for (i, led) in leds.iter_mut().enumerate() {
            let led_idx = (i + *rotation_idx) % N_LEDS;
            let color_idx = (led_idx as f32 * color_step) as usize;
            *led = colors[color_idx % colors.len()];
        }
    }

    pub fn step_smoothspin(leds: &mut [RgbS; N_LEDS],
                       current_rotation: &mut f32,
                       gradient: &[RgbS],
                       period: u16
    ) {
        let step = if period == 0 {
            0.0
        } else {
            N_LEDS as f32 / period as f32
        };
        
        *current_rotation = (*current_rotation + step) % N_LEDS as f32;

        Animation::map_gradient(leds, gradient, *current_rotation);
    }
    
    pub fn map_gradient(samples: &mut [RgbS; N_LEDS], gradient: &[RgbS], rotation: f32) {
        for (i, sample) in samples.iter_mut().enumerate() {
            let sample_pos = (rotation + i as f32) % N_LEDS as f32;
            *sample = sample_gradient(gradient, sample_pos, N_LEDS);
        }
    }
     
    pub fn step_rainbow_spin(leds: &mut [RgbS; N_LEDS], rotation_idx: &mut usize) {
        *rotation_idx = (*rotation_idx + 1) % N_LEDS;
    
        for (i, led) in leds.iter_mut().enumerate() {
            *led = RAINBOW[(*rotation_idx+i) % N_LEDS];
        }
    }


    // stepper function
    pub fn step(&mut self, leds: &mut [RgbS; N_LEDS]) {
        match self {
            Animation::Solid { color } => {
                for led in leds {
                    *led = color.clone();
                }
            },
            Animation::Blink {
                colors,
                current_color_index,
                on,
            } => {
                if *on {
                    for led in leds {
                        *led = OFF;
                    }
                    
                    if (*current_color_index as usize) >= (*colors).len() - 1 {
                        *current_color_index = 0;
                    } else {
                        *current_color_index += 1;
                    }
                    
                } else {
                    let current_color: RgbS = match colors.get_mut(*current_color_index as usize) {
                        Some(color) => *color,
                        None => panic!("Index {} is out of bounds", *current_color_index),
                    };
                    
                    for led in leds {                        
                        *led = current_color.clone();
                    }
                }

                *on = !*on;
                
            },
            Animation::Spin {colors, idx} => {
                Animation::step_discrete_spin(leds, colors, idx);
            },
            Animation::SmoothSpin {
                colors,
                period,
                current_rotation,  
            } => {
                Animation::step_smoothspin(leds, current_rotation, colors, *period);
            },
            Animation::RainbowSpin {idx} => {
                Animation::step_rainbow_spin(leds, idx);
            },
            // Animation::RainbowSmoothSpin {
            //     period,
            //     current_rotation,  
            // } => {
            //     Animation::step_smoothspin(leds, current_rotation, &RAINBOW, *period);
            // },
            Animation::Mpd {
                visualizer
            } => {
                visualizer.tick(leds);
            },
        }
    }
}


// linear interpolation between two colors
fn lerp(a: RgbS, b: RgbS, t: f32) -> RgbS {
    RgbS {
        r: (a.r as f32 + (b.r as f32 - a.r as f32) * t).round() as u8,
        g: (a.g as f32 + (b.g as f32 - a.g as f32) * t).round() as u8,
        b: (a.b as f32 + (b.b as f32 - a.b as f32) * t).round() as u8,
    }
}

// samples the true color gradient wheel at an led
fn sample_gradient(colors: &[RgbS], pos: f32, slices: usize) -> RgbS {
    let n = colors.len();

    let scaled = pos * n as f32 / slices as f32;
    let idx = scaled.floor() as usize % n;
    let next_idx = (idx + 1) % n;
    let t = scaled - scaled.floor();

    lerp(colors[idx], colors[next_idx], t)
}
