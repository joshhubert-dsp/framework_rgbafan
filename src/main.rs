mod animations;
mod cli;
mod consts;
mod effects;
mod fan_speed;
mod mpd_visualizer;

use crate::animations::Animation;
use crate::cli::Args;
use crate::consts::{N_LEDS, OFF};
use crate::effects::{BrightnessEffect, opt_brightness_effect_from_cli};
use crate::fan_speed::{fan_speed_to_tick_time, get_fan_speed};

use clap::Parser;
use framework_lib::chromium_ec::commands::RgbS;
use framework_lib::chromium_ec::{CrosEc, EcResult};
use std::num::ParseIntError;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
enum HexParseError {
    WrongLength,
    InvalidDigit,
}

impl From<ParseIntError> for HexParseError {
    fn from(_: ParseIntError) -> Self {
        HexParseError::InvalidDigit
    }
}

fn parse_hex(s: &str) -> Result<RgbS, HexParseError> {
    if s == "0" {
        return Ok(OFF);
    }
    if s.len() != 6 {
        return Err(HexParseError::WrongLength);
    }

    let r = u8::from_str_radix(&s[0..2], 16)?;
    let g = u8::from_str_radix(&s[2..4], 16)?;
    let b = u8::from_str_radix(&s[4..6], 16)?;

    Ok(RgbS { r, g, b })
}

fn args_to_rgbs(args: Vec<String>) -> Result<Vec<RgbS>, HexParseError> {
    args.into_iter().map(|s| parse_hex(&s)).collect()
}

fn main() -> EcResult<()> {
    let args = Args::parse();
    println!("{:#?}", args);

    let colors: Vec<RgbS> = args_to_rgbs(args.colors)
        .unwrap_or_else(|e| panic!("Failed to parse color argument: {e:?}"));

    let mut animation = Animation::from_cli(&args.mode, colors);
    let mut opt_effect: Option<BrightnessEffect> =
        opt_brightness_effect_from_cli(args.effect, args.effect_period);

    let mut tick_time: u64 = args.tick_ms;

    let ec = CrosEc::new();

    let mut leds: [RgbS; N_LEDS] = [OFF; N_LEDS];

    let mut fan_rpm: u16;

    loop {
        animation.step(&mut leds);
        if let Some(effect) = opt_effect.as_mut() {
            effect.step(&mut leds);
        }

        if let Err(e) = ec.rgbkbd_set_color(0, leds.to_vec()) {
            eprintln!("Error setting lights: {:?}", e);
        }

        if args.speed_from_fan {
            fan_rpm = get_fan_speed(&ec).unwrap();
            tick_time = fan_speed_to_tick_time(fan_rpm);
            // println!("  Fan Speed:  {:>4} RPM", fan_rpm);
            // println!("  Tick Time:  {:>4} ms", tick_time);
        }
        // NOTE: this is the only place the program sleeps now
        thread::sleep(Duration::from_millis(tick_time))
    }
}
