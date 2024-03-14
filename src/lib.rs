#![no_std]

mod fusion_vector_impl;
mod fusion_quaternion_impl;
mod fusion_matrix_impl;
mod fusion_euler_impl;
mod fusion_impl;
mod fusion_ahrs_impl;
mod fusion_gyr_offset_impl;
mod nalgebra;

pub enum FusionConvention {
    /* North-West-Up */
    NWU,
    /* East-North-Up */
    ENU,
    /* North-East-Down */
    NED,
}

pub struct Fusion {
    pub gyr_misalignment: FusionMatrix,
    pub gyr_sensitivity: FusionVector,
    pub gyr_offset: FusionVector,
    pub acc_misalignment: FusionMatrix,
    pub acc_sensitivity: FusionVector,
    pub acc_offset: FusionVector,
    pub soft_iron_matrix: FusionMatrix,
    pub hard_iron_offset: FusionVector,
    pub ahrs: FusionAhrs,
    pub offset: FusionGyrOffset,
    pub last_timestamp: f32,
}

pub struct FusionAhrs {
    pub settings: FusionAhrsSettings,
    pub quaternion: FusionQuaternion,
    pub acc: FusionVector,
    pub initialising: bool,
    pub ramped_gain: f32,
    pub ramped_gain_step: f32,
    pub angular_rate_recovery: bool,
    pub half_accelerometer_feedback: FusionVector,
    pub half_magnetometer_feedback: FusionVector,
    pub accelerometer_ignored: bool,
    pub acceleration_recovery_trigger: i32,
    pub acceleration_recovery_timeout: i32,
    pub magnetometer_ignored: bool,
    pub magnetic_recovery_trigger: i32,
    pub magnetic_recovery_timeout: i32,
}

pub struct FusionAhrsSettings {
    pub convention: FusionConvention,
    pub gain: f32,
    pub gyr_range: f32,
    pub acc_rejection: f32,
    pub mag_rejection: f32,
    pub recovery_trigger_period: i32,
}

#[derive(Copy, Clone)]
pub struct Angle {
    pub roll: f32,
    pub pitch: f32,
    pub yaw: f32,
}

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct FusionVector {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Copy, Clone)]
pub struct FusionMatrix {
    pub xx: f32,
    pub xy: f32,
    pub xz: f32,
    pub yx: f32,
    pub yy: f32,
    pub yz: f32,
    pub zx: f32,
    pub zy: f32,
    pub zz: f32,
}

#[derive(Copy, Clone)]
pub struct FusionQuaternion {
    pub w: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct FusionEuler {
    pub angle: Angle,
}

pub struct FusionGyrOffset {
    pub filter_coefficient: f32,
    pub timeout: u32,
    pub timer: u32,
    pub gyroscope_offset: FusionVector,
}

// Timeout in seconds.
const TIMEOUT: u32 = 5;

// Cutoff frequency in Hz.
const CUTOFF_FREQUENCY: f32 = 0.02f32;

// Threshold in degrees per second.
const THRESHOLD: f32 = 3f32;

fn fusion_degrees_to_radians(degrees: f32) -> f32 {
    degrees * (core::f32::consts::PI / 180.0f32)
}

fn fusion_radians_to_degrees(radians: f32) -> f32 {
    radians * (180.0f32 / core::f32::consts::PI)
}

fn asin_safe(value: f32) -> f32 {
    use libm::{asinf};
    if value <= -1.0f32 {
        return core::f32::consts::PI / -2.0f32;
    }
    if value >= 1.0f32 {
        return core::f32::consts::PI / 2.0f32;
    }
    asinf(value)
}

fn fusion_fast_inverse_sqrt(x: f32) -> f32 {
    union Union32 {
        f: f32,
        i: i32,
    }

    let mut union32 = Union32 { f: x };
    unsafe {
        union32.i = 0x5F1F1412 - (union32.i >> 1);
        union32.f * (1.69000231f32 - 0.714158168f32 * x * union32.f * union32.f)
    }
}


#[test]
fn fusion_fast_inverse_sqrt_test() {
    use libm::{fabsf};
    let result = fusion_fast_inverse_sqrt(9.0f32);
    let actual = 1f32 / result;
    let expected = 3f32;
    assert!(fabsf(actual - expected) < 0.01f32);
}
