#![no_std]
#![no_main]

use microbit_v2_examples as _;

use core::fmt::Write;
use core::str;
use heapless::Vec;
use lsm303agr::{AccelOutputDataRate, Lsm303agr, MagOutputDataRate};
use microbit::hal::pac::twim0::frequency::FREQUENCY_A;
use microbit::hal::prelude::*;
use microbit::hal::twim::Twim;
use microbit::hal::uarte::{Baudrate, Parity, Uarte};
use microbit::Board;

use microbit_v2_examples::serial_setup::UartePort;

#[cortex_m_rt::entry]
fn main() -> ! {
    let board = Board::take().unwrap();

    // init uarte port
    let mut serial = {
        let serial = Uarte::new(
            board.UARTE0,
            board.uart.into(),
            Parity::EXCLUDED,
            Baudrate::BAUD115200,
        );
        UartePort::new(serial)
    };

    // init i2c sensor
    let mut sensor = {
        let i2c = Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100);
        Lsm303agr::new_with_i2c(i2c)
    };
    sensor.init().unwrap();
    sensor.set_accel_odr(AccelOutputDataRate::Hz50).unwrap();
    sensor.set_mag_odr(MagOutputDataRate::Hz50).unwrap();

    let mut sensor = sensor.into_mag_continuous().ok().unwrap();

    let mut buffer = Vec::<u8, 32>::new();
    loop {
        write!(serial, "input: ").unwrap();
        nb::block!(serial.flush()).unwrap();
        loop {
            let byte = nb::block!(serial.read()).unwrap();
            write!(serial, "{}", byte as char).unwrap();
            nb::block!(serial.flush()).unwrap();
            if byte == 13 {
                break;
            }
            if buffer.push(byte).is_err() {
                write!(serial, "error: buffer full\r\n").unwrap();
                break;
            }
        }

        match str::from_utf8(&buffer) {
            Ok("accelerometer") => {
                while !sensor.accel_status().unwrap().xyz_new_data {}
                let data = sensor.accel_data().unwrap();
                // RTT instead of normal print
                // rprintln!("Acceleration: x {} y {} z {}\r\n", data.x, data.y, data.z);
                write!(
                    serial,
                    "Acceleration: x {} y {} z {}\r\n",
                    data.x, data.y, data.z
                )
                .unwrap();
            }
            Ok("magnetometer") => {
                while !sensor.mag_status().unwrap().xyz_new_data {}
                let data = sensor.mag_data().unwrap();
                // RTT instead of normal print
                // rprintln!("Magnetometer: x {} y {} z {}\r\n", data.x, data.y, data.z);
                write!(
                    serial,
                    "Magnetometer: x {} y {} z {}\r\n",
                    data.x, data.y, data.z
                )
                .unwrap();
            }
            Ok(command) => {
                write!(serial, "error: '{}' command not detected\r\n", command).unwrap();
            }
            Err(_) => {
                write!(serial, "error: can't convert to utf8 str").unwrap();
            }
        };

        nb::block!(serial.flush()).unwrap();
        buffer.clear();
    }
}
