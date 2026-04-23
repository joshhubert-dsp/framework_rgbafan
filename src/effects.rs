use crate::consts::{N_LEDS, OFF};
use framework_lib::chromium_ec::commands::RgbS;

const SPINFADE_SCALES: [f32; N_LEDS] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.25, 0.5, 0.75];
// const SPINFADE_SCALES: [f32; N_LEDS] = [0.0, 0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875];

/// brightness effects can be applied to all animations
pub enum BrightnessEffect {
    Blink {
        period: usize, // unit in ticks
        idx: usize,
        on: bool,
    },
    Pulse {
        period: usize, // unit in ticks
        half_period: f32,
        idx: usize,
    },
    CwFade {
        period: usize, // unit in ticks
        idx: usize,
    },
    CcwFade {
        period: usize, // unit in ticks
        idx: usize,
    },
    CwCcwFade {
        period: usize, // unit in ticks
        idx: usize,
        cw: bool,
    },
}

pub fn opt_brightness_effect_from_cli(
    name: Option<String>,
    period: usize,
) -> Option<BrightnessEffect> {
    match name {
        Some(name) if name == "blink" => Some(BrightnessEffect::Blink {
            period: period,
            idx: 0,
            on: true,
        }),
        Some(name) if name == "pulse" => Some(BrightnessEffect::Pulse {
            period: period,
            half_period: (period >> 1) as f32,
            idx: 0,
        }),
        Some(name) if name == "cwfade" => Some(BrightnessEffect::CwFade {
            period: period,
            idx: 0,
        }),
        Some(name) if name == "ccwfade" => Some(BrightnessEffect::CcwFade {
            period: period,
            idx: 0,
        }),
        Some(name) if name == "cwccwfade" => Some(BrightnessEffect::CwCcwFade {
            period: period,
            idx: 0,
            cw: false,
        }),
        Some(_) => None,
        None => None,
    }
}

impl BrightnessEffect {
    pub fn step(&mut self, leds: &mut [RgbS]) {
        match self {
            BrightnessEffect::Blink { period, idx, on } => {
                if !*on {
                    for led in leds {
                        *led = OFF;
                    }
                }
                *idx += 1;
                *idx %= *period;
                if *idx == 0 {
                    *on = !*on;
                }
            }
            BrightnessEffect::Pulse {
                period,
                half_period,
                idx,
            } => {
                // 0.-1.
                let dim_scale: f32 = ((*idx as f32 - *half_period) / *half_period).abs();
                for led in leds {
                    led.r = (dim_scale * led.r as f32) as u8;
                    led.g = (dim_scale * led.g as f32) as u8;
                    led.b = (dim_scale * led.b as f32) as u8;
                }
                *idx += 1;
                *idx %= *period;
            }
            BrightnessEffect::CwFade { period, idx } => {
                spinfade(leds, period, idx, true);
            }
            BrightnessEffect::CcwFade { period, idx } => {
                spinfade(leds, period, idx, false)
            }
            BrightnessEffect::CwCcwFade { period, idx, cw } => {
                spinfade(leds, period, idx, *cw);
                if *idx == 0 {
                    *cw = !*cw;
                }
            }
        }
    }
}

fn spinfade(leds: &mut [RgbS], period: &usize, idx: &mut usize, cw: bool) {
    let mut offset: i8 = if *period == 0 {
        0
    } else {
        (((*idx * N_LEDS) / *period) % N_LEDS) as i8
    };
    if !cw {
        offset *= -1;
    }

    for (i, led) in leds.iter_mut().enumerate() {
        let dim_scale =
            SPINFADE_SCALES[((i as i8 + offset).rem_euclid(N_LEDS as i8)) as usize];
        led.r = (dim_scale * led.r as f32) as u8;
        led.g = (dim_scale * led.g as f32) as u8;
        led.b = (dim_scale * led.b as f32) as u8;
    }

    if *period != 0 {
        *idx = (*idx + 1) % *period;
    }
}
