use framework_lib::chromium_ec::{CrosEc, CrosEcDriver};

// observed maximum cooler master mobius fan speed (rpm)
const MIN_FAN_RPM: f32 = 0.;
const MAX_FAN_RPM: f32 = 2400.;
const MIN_TICK_TIME_MS: u64 = 1;
const MAX_TICK_TIME_MS: u64 = 500;

// ripped from framework_lib/src/power.rs
const EC_MEMMAP_FAN: u16 = 0x10; // Fan speeds 0x10 - 0x17
const EC_FAN_SPEED_ENTRIES: usize = 4;
const EC_FAN_READ_BYTES: u16 = (EC_FAN_SPEED_ENTRIES as u16) * 2;
/// Used on old EC firmware (before 2023)
const EC_FAN_SPEED_STALLED_DEPRECATED: u16 = 0xFFFE;
const EC_FAN_SPEED_NOT_PRESENT: u16 = 0xFFFF;

#[derive(Debug)]
pub enum FanSpeedReadError {
    StalledDeprectated,
    NotPresent,
}

pub type FanSpeedResult = Result<u16, FanSpeedReadError>;

/// assuming we're only working with 1 fan on framework desktop
pub fn get_fan_speed(ec: &CrosEc) -> FanSpeedResult {
    let fan_bytes = ec.read_memory(EC_MEMMAP_FAN, EC_FAN_READ_BYTES).unwrap();
    let fan_int =
        u16::from_le_bytes([*fan_bytes.first().unwrap(), *fan_bytes.get(1).unwrap()]);

    match fan_int {
        EC_FAN_SPEED_STALLED_DEPRECATED => Err(FanSpeedReadError::StalledDeprectated),
        EC_FAN_SPEED_NOT_PRESENT => Err(FanSpeedReadError::NotPresent),
        rpm => Ok(rpm),
    }
}

/// converts fan rpm to a fraction of maximum
fn fan_speed_to_fraction(rpm: u16) -> f32 {
    let rpm = (rpm as f32).clamp(MIN_FAN_RPM, MAX_FAN_RPM);
    (rpm - MIN_FAN_RPM) / (MAX_FAN_RPM - MIN_FAN_RPM)
}

/// converts fan rpm to update tick time in ms, inverse linear relationship
pub fn fan_speed_to_tick_time(rpm: u16) -> u64 {
    let rpm_frac = fan_speed_to_fraction(rpm);
    let tick_frac = 1.0 - rpm_frac;
    let span = (MAX_TICK_TIME_MS - MIN_TICK_TIME_MS) as f32;
    MIN_TICK_TIME_MS + (span * tick_frac).round() as u64
}
