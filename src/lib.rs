#![no_std]

use defmt_rtt as _;
use panic_probe as _;

pub mod calibration;
pub mod led;
pub mod music;
pub mod serial_setup;
