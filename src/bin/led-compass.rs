#![no_std]
#![no_main]

use core::f32::consts::PI;

use libm::atan2f;
use lsm303agr::{AccelOutputDataRate, Lsm303agr, MagOutputDataRate};
use microbit::board::Board;
use microbit::display::blocking::Display;
use microbit::hal::twim::Twim;
use microbit::hal::Timer;
use microbit::pac::twim0::frequency::FREQUENCY_A;

use microbit_v2_examples::{
    self as _,
    calibration::calibrated_measurement,
    led::{direction_to_led, Direction},
};

#[cfg(feature = "default")]
use microbit_v2_examples::calibration::Calibration;

#[cfg(feature = "calibration")]
use core::fmt::Write;
#[cfg(feature = "calibration")]
use microbit::{
    hal::uarte::{Baudrate, Parity},
    hal::Uarte,
};
#[cfg(feature = "calibration")]
use microbit_v2_examples::{calibration::calc_calibration, serial_setup::UartePort};

#[cortex_m_rt::entry]
fn main() -> ! {
    // rtt_init_print!();

    let board = Board::take().unwrap();
    #[cfg(feature = "calibration")]
    let mut serial = {
        let serial = Uarte::new(
            board.UARTE0,
            board.uart.into(),
            Parity::EXCLUDED,
            Baudrate::BAUD115200,
        );
        UartePort::new(serial)
    };
    let mut timer = Timer::new(board.TIMER0);
    let mut display = Display::new(board.display_pins);

    let i2c = Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100);

    let mut sensor = {
        let mut sensor = Lsm303agr::new_with_i2c(i2c);
        sensor.init().unwrap();
        sensor.set_accel_odr(AccelOutputDataRate::Hz10).unwrap();
        sensor.set_mag_odr(MagOutputDataRate::Hz10).unwrap();
        sensor.into_mag_continuous().ok().unwrap()
    };

    #[cfg(feature = "calibration")]
    let calibration = {
        let calibration = calc_calibration(&mut sensor, &mut display, &mut timer);
        write!(serial, "Calibration: {:?}\r\n", calibration).unwrap();
        write!(serial, "Calibration done, entering busy loop\r\n").unwrap();
        calibration
    };

    #[cfg(feature = "default")]
    let calibration = Calibration::default();

    loop {
        while !sensor.mag_status().unwrap().xyz_new_data {}
        let mut data = sensor.mag_data().unwrap();
        data = calibrated_measurement(data, &calibration);

        let theta = atan2f(data.y as f32, data.x as f32);

        let dir = if theta < -7. * PI / 8. {
            Direction::West
        } else if theta < -5. * PI / 8. {
            Direction::SouthWest
        } else if theta < -3. * PI / 8. {
            Direction::South
        } else if theta < -PI / 8. {
            Direction::SouthEast
        } else if theta < PI / 8. {
            Direction::East
        } else if theta < 3. * PI / 8. {
            Direction::NorthEast
        } else if theta < 5. * PI / 8. {
            Direction::North
        } else if theta < 7. * PI / 8. {
            Direction::NorthWest
        } else {
            Direction::West
        };

        display.show(&mut timer, direction_to_led(dir), 100);
    }
}
