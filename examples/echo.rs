#![no_std]
#![no_main]

use panic_rtt_target as _;

use core::fmt::Write;
use microbit::hal::prelude::*;
use microbit::hal::uarte::{self, Baudrate, Parity};
use microbit::Board;

use serial_setup::UartePort;

use heapless::String;

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

mod serial_setup {
    use core::fmt;
    use embedded_hal::blocking::serial as bserial;
    use embedded_hal::serial;
    use microbit::hal::uarte::{Error, Instance, Uarte, UarteRx, UarteTx};

    static mut TX_BUF: [u8; 1] = [0; 1];
    static mut RX_BUF: [u8; 1] = [0; 1];

    pub struct UartePort<T: Instance>(UarteTx<T>, UarteRx<T>);

    impl<T: Instance> UartePort<T> {
        pub fn new(serial: Uarte<T>) -> UartePort<T> {
            let (tx, rx) = serial
                .split(unsafe { &mut TX_BUF }, unsafe { &mut RX_BUF })
                .unwrap();
            UartePort(tx, rx)
        }
    }

    impl<T: Instance> fmt::Write for UartePort<T> {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            self.0.write_str(s)
        }
    }

    impl<T: Instance> serial::Write<u8> for UartePort<T> {
        type Error = Error;

        fn write(&mut self, b: u8) -> nb::Result<(), Self::Error> {
            self.0.write(b)
        }

        fn flush(&mut self) -> nb::Result<(), Self::Error> {
            self.0.flush()
        }
    }

    impl<T: Instance> bserial::write::Default<u8> for UartePort<T> {}

    impl<T: Instance> serial::Read<u8> for UartePort<T> {
        type Error = Error;

        fn read(&mut self) -> nb::Result<u8, Self::Error> {
            self.1.read()
        }
    }
}
