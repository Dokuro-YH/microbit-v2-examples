#![no_std]
#![no_main]

use microbit::hal::pac::twim0::frequency::FREQUENCY_A;
use microbit::hal::prelude::*;
use microbit::hal::twim;
use microbit::Board;
use panic_rtt_target as _;
use rtt_target::rprintln;

const ACCELEROMETER_ADDR: u8 = 0b0011001;
const MAGNETOMETER_ADDR: u8 = 0b0011110;

const ACCELEROMETER_ID_REG: u8 = 0x0f;
const MAGNETOMETER_ID_REG: u8 = 0x4f;

#[cortex_m_rt::entry]
fn main() -> ! {
    rtt_target::rtt_init_print!();

    let board = Board::take().unwrap();

    let mut i2c = { twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100) };

    let mut acc = [0];
    let mut mag = [0];

    i2c.write_read(ACCELEROMETER_ADDR, &[ACCELEROMETER_ID_REG], &mut acc).unwrap();
    i2c.write_read(MAGNETOMETER_ADDR, &[MAGNETOMETER_ID_REG], &mut mag).unwrap();

    rprintln!("The accelerometer chip's id is: #{:#b}", acc[0]);
    rprintln!("The magnetometer chip's id is: #{:#b}", mag[0]);

    loop {}
}
