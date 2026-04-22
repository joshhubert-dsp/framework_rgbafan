use crate::consts::{N_LEDS, RAINBOW, SPIN_PERIOD};
use crate::effects::BrightnessEffect;
use crate::mpd_visualizer::MpdVisualizer;
use framework_lib::chromium_ec::commands::RgbS;
use rand::{random, random_range};

pub enum Animation {
    Static {
        colors: Vec<RgbS>,
    },
    Sequence {
        colors: Vec<RgbS>,
        idx: usize, // current color idx
    },
    Random,
    RandomFromInput {
        colors: Vec<RgbS>,
    },
    QuadSpin {
        colors: Vec<RgbS>,
        idx: usize, // current idx on discrete spin
    },
    FullSpin {
        colors: Vec<RgbS>,
        idx: usize, // current idx on discrete spin
    },
    SmoothSpin {
        colors: Vec<RgbS>,
        period: u16,           // unit in ticks
        current_rotation: f32, // unit in leds
    },
    RainbowSpin {
        idx: usize, // current idx on rolling rainbow
    },
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
            "static" => Animation::Static { colors },
            "sequence" => Animation::Sequence { colors, idx: 0 },
            "random" => Animation::Random,
            "randominput" => Animation::RandomFromInput { colors },
            "quadspin" => Animation::QuadSpin { colors, idx: 0 },
            "fullspin" => Animation::FullSpin { colors, idx: 0 },
            "smoothspin" => Animation::SmoothSpin {
                colors,
                period: SPIN_PERIOD,
                current_rotation: 0.0,
            },
            "rainbowspin" => Animation::RainbowSpin { idx: 0 },
            "mpd" => Animation::Mpd {
                visualizer: MpdVisualizer::new(colors, SPIN_PERIOD),
            },
            _ => panic!("Unknown animation mode."),
        }
    }

    pub fn step_smoothspin(
        leds: &mut [RgbS; N_LEDS],
        current_rotation: &mut f32,
        gradient: &[RgbS],
        period: u16,
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

    // stepper function
    pub fn step(
        &mut self,
        leds: &mut [RgbS; N_LEDS],
        effect: Option<&mut BrightnessEffect>,
    ) {
        match self {
            Animation::Static { colors } => {
                map_colors_to_led_range(leds, &colors, 0);
            }
            Animation::Sequence { colors, idx } => {
                for led in leds.iter_mut() {
                    *led = colors[*idx];
                }
                *idx += 1;
                *idx %= colors.len();
            }
            Animation::Random => {
                for led in leds.iter_mut() {
                    led.r = random::<u8>();
                    led.g = random::<u8>();
                    led.b = random::<u8>();
                }
            }
            Animation::RandomFromInput { colors } => {
                for led in leds.iter_mut() {
                    let rf = random::<f32>();
                    let rc = colors[random_range(..colors.len())];
                    led.r = (rc.r as f32 * rf) as u8;
                    led.g = (rc.g as f32 * rf) as u8;
                    led.b = (rc.b as f32 * rf) as u8;
                }
            }
            Animation::QuadSpin { colors, idx } => {
                for i in 0..4 as usize {
                    let color = colors[(*idx + i) % colors.len()];
                    leds[2 * i] = color;
                    leds[2 * i + 1] = color;
                }
                *idx += 1;
                *idx %= colors.len();
            }
            Animation::FullSpin { colors, idx } => {
                map_colors_to_led_range(leds, &colors, *idx);
                *idx += 1;
                *idx %= N_LEDS;
            }
            Animation::SmoothSpin {
                colors,
                period,
                current_rotation,
            } => {
                Animation::step_smoothspin(leds, current_rotation, colors, *period);
            }
            Animation::RainbowSpin { idx } => {
                for (i, led) in leds.iter_mut().enumerate() {
                    *led = RAINBOW[(*idx + i) % N_LEDS];
                }
                *idx += 1;
                *idx %= N_LEDS;
            }
            Animation::Mpd { visualizer } => {
                visualizer.tick(leds);
            }
        }

        if let Some(effect) = effect {
            effect.step(leds);
        }
    }
}

/// expands N colors discretely over M leds
fn map_colors_to_led_range(
    leds: &mut [RgbS; N_LEDS],
    colors: &[RgbS],
    led_offset: usize,
) {
    if colors.len() == 1 {
        for led in leds {
            *led = colors[0];
        }
        return;
    }

    let color_step = colors.len() as f32 / N_LEDS as f32;

    for (i, led) in leds.iter_mut().enumerate() {
        let led_idx = (i + led_offset) % N_LEDS;
        let color_idx = (led_idx as f32 * color_step).floor() as usize;
        *led = colors[color_idx % colors.len()];
    }
}

/// linear interpolation between two colors
fn lerp(a: RgbS, b: RgbS, t: f32) -> RgbS {
    RgbS {
        r: (a.r as f32 + (b.r as f32 - a.r as f32) * t).round() as u8,
        g: (a.g as f32 + (b.g as f32 - a.g as f32) * t).round() as u8,
        b: (a.b as f32 + (b.b as f32 - a.b as f32) * t).round() as u8,
    }
}

/// samples the true color gradient wheel at an led
fn sample_gradient(colors: &[RgbS], pos: f32, slices: usize) -> RgbS {
    let n = colors.len();

    let scaled = pos * n as f32 / slices as f32;
    let idx = scaled.floor() as usize % n;
    let next_idx = (idx + 1) % n;
    let t = scaled - scaled.floor();

    lerp(colors[idx], colors[next_idx], t)
}
