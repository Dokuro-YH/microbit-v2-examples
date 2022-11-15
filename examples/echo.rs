#![no_std]
#![no_main]

use core::fmt::Write;
use heapless::String;
use microbit::hal::prelude::*;
use microbit::hal::uarte::{self, Baudrate, Parity};
use microbit::Board;
use panic_rtt_target as _;

#[path = "../src/serial_setup.rs"]
mod serial_setup;
use serial_setup::UartePort;

#[cortex_m_rt::entry]
fn main() -> ! {
    rtt_target::rtt_init_print!();

    let board = Board::take().unwrap();
    let serial = uarte::Uarte::new(
        board.UARTE0,
        board.uart.into(),
        Parity::EXCLUDED,
        Baudrate::BAUD115200,
    );
    let mut serial = UartePort::new(serial);
    let mut buffer = String::<32>::new();

    loop {
        loop {
            let byte = nb::block!(serial.read()).unwrap();

            if buffer.push(byte as char).is_err() {
                write!(serial, "error: buffer full\r\n").unwrap();
                break;
            }

            if byte == 13 {
                write!(serial, "{}\r\n", buffer).unwrap();
                break;
            }
        }
        nb::block!(serial.flush()).unwrap();

        buffer.clear();
    }
}
