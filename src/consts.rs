use framework_lib::chromium_ec::commands::RgbS;

// Milliseconds per second
pub const UNIT_MS: u16 = 1000;
pub const N_LEDS: u8 = 8;
pub const TICKRATE: u16 = 32;
// how often to refresh in case computer enters sleep mode or something
pub const REFRESH_PERIOD: u16 = 5 * UNIT_MS;
pub const BLINK_PERIOD: u16 = UNIT_MS / TICKRATE;
pub const SPIN_PERIOD: u16 = (UNIT_MS / TICKRATE) * 5;

pub const OFF: RgbS = RgbS{ r: 0, g: 0, b: 0 };

// MPD
//pub const SAMPLE_RATE: f32 = 44100.0;
pub const FFT_SIZE: usize = 1024;
pub const FIFO_PATH: &str = "/tmp/rgb.fifo";
pub const MPD_QUIET_TIMEOUT: u8 = 5;
