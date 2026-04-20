use std::num::ParseIntError;
use std::thread;
use std::time::Duration;

use framework_lib::chromium_ec::commands::RgbS;
use framework_lib::chromium_ec::{CrosEc, EcResult};

mod animations;
mod consts;
mod mpd_visualizer;

use animations::Animation;
use consts::{DEFAULT_TICK_TIME_MS, N_LEDS, OFF, SOLID_TICK_TIME_MS};

use clap::{ColorChoice, Parser};

/// Animate your Framework computer RGB fan!
#[derive(Parser, Debug)]
#[command(version, about, color = ColorChoice::Auto, long_about = None)]
struct Args {
    /// Avaiable modes: solid, blink, spin, smoothspin, rainbowspin, mpd
    #[arg(required = true)]
    mode: String,

    /// Integer number of milliseconds between updates, for all modes besides solid
    #[arg(default_value_t=DEFAULT_TICK_TIME_MS)]
    tick_ms: u16,

    /// List of 1-8 color hex strings, specified with 6 characters each or 0 for OFF.
    /// Only the first is used for solid, and none are used for rainbow.
    #[arg(short, long, num_args = 1..9, default_values_t = ["ff0000".to_string(), "00ff00".to_string(), "0000ff".to_string()])]
    colors: Vec<String>,
}

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
    if s == "0"{
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
    // println!("{:#?}", args);

    let colors: Vec<RgbS> = args_to_rgbs(args.colors)
        .unwrap_or_else(|e| panic!("Failed to parse color argument: {e:?}"));

    let mut animation = Animation::from_cli(&args.mode, colors);

    let sleep_time: u16 = match animation {
        Animation::Solid { color: _ } => SOLID_TICK_TIME_MS,
        _ => args.tick_ms,
    };

    let ec = CrosEc::new();

    let mut leds: [RgbS; N_LEDS] = [OFF; N_LEDS];

    loop {
        animation.step(&mut leds);

        if let Err(e) = ec.rgbkbd_set_color(0, leds.to_vec()) {
            eprintln!("Error setting lights: {:?}", e);
        }

        // NOTE: this is the only place the program sleeps now
        thread::sleep(Duration::from_millis(sleep_time.into()))
    }
}
