#![no_main]
#![no_std]

use microbit::hal::gpio::Level;
use microbit::hal::prelude::*;
use microbit::hal::timer::Timer;
use microbit::Board;

use microbit_v2_examples::{self as _, music::Music};

#[cortex_m_rt::entry]
fn main() -> ! {
    let board = Board::take().unwrap();

    let pin = board.speaker_pin.into_push_pull_output(Level::High);
    let mut music = Music::new(pin.degrade(), board.PWM0, board.TIMER0);
    let mut timer = Timer::new(board.TIMER1);
    defmt::info!("start loop play music");
    loop {
        music.play(
            r#"c4 c4 g4 g4 a4 a4 g4 -
               f4 f4 e4 e4 d4 d4 c4 -
               g4 g4 f4 f4 e4 e4 d4 -
               g4 g4 f4 f4 e4 e4 d4 -"#,
        );
        timer.delay_ms(1000_u32);
    }
}
