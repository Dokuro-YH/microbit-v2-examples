#![no_main]
#![no_std]

use microbit_v2_examples::{self as _, music::Music};

use microbit::hal::gpio::Level;
use microbit::hal::prelude::*;
use microbit::hal::pwm::{Channel, CounterMode, Pwm};
use microbit::hal::timer::Timer;
use microbit::Board;

#[cortex_m_rt::entry]
fn main() -> ! {
    let board = Board::take().unwrap();

    let mut pin = board.speaker_pin.into_push_pull_output(Level::High);
    pin.set_low().unwrap();

    let pwm = Pwm::new(board.PWM0);
    pwm.set_output_pin(Channel::C0, pin.degrade())
        .set_counter_mode(CounterMode::UpAndDown)
        .enable();

    let mut music = Music::new(pwm, Timer::new(board.TIMER0));
    let mut timer = Timer::new(board.TIMER1);
    loop {
        defmt::info!("start play music");
        music.play(
            r#"c4 c4 g4 g4 a4 a4 g4 -
               f4 f4 e4 e4 d4 d4 c4 -
               g4 g4 f4 f4 e4 e4 d4 -
               g4 g4 f4 f4 e4 e4 d4 -"#,
        );
        timer.delay_ms(1000_u32);
    }
}
