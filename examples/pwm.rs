#![no_main]
#![no_std]

use panic_halt as _;
use rtt_target::rprintln;

use cortex_m_rt::entry;
use microbit::hal::gpio::Level;
use microbit::hal::prelude::*;
use microbit::hal::pwm::{Channel, CounterMode, Prescaler, Pwm, Seq};
use microbit::hal::timer::Timer;
use microbit::Board;

#[entry]
fn main() -> ! {
    rtt_target::rtt_init_print!();

    let board = Board::take().unwrap();

    let mut pin = board.speaker_pin.into_push_pull_output(Level::High);
    pin.set_low().unwrap();
    // let pin = board.display_pins.row1;
    // board.display_pins.col1.set_low().unwrap();

    let pwm = Pwm::new(board.PWM0);
    pwm.set_output_pin(Channel::C0, pin.degrade())
        .set_counter_mode(CounterMode::UpAndDown)
        .set_seq_refresh(Seq::Seq0, 0)
        .set_seq_end_delay(Seq::Seq0, 0)
        .set_prescaler(Prescaler::Div16)
        .set_period(1.hz())
        .set_max_duty(u16::MAX)
        .enable();

    // duty = 音量
    // freq = 音调

    let mut timer = Timer::new(board.TIMER0);

    let mut max_duty = pwm.max_duty();
    let mut duty = pwm.max_duty() / 7;
    pwm.set_duty_on_common(duty);

    loop {
        for i in music::music_1 {
            rprintln!("max_duty {} duty {} {}hz ", max_duty, duty, i);
            pwm.set_period(i.hz());
            timer.delay_ms(3000_u32);

            max_duty = pwm.max_duty();
            duty = max_duty / 2;
            pwm.set_duty_on_common(duty);
            timer.delay_ms(100_u32);
        }

        // pwm.set_duty_on_common(pwm.max_duty());
        rprintln!("Restart!");
        timer.delay_ms(3000_u32);
    }
}

mod music {
    pub const D_DO: u32 = 262;
    pub const D_RE: u32 = 294;
    pub const D_MI: u32 = 330;
    pub const D_FA: u32 = 349;
    pub const D_SO: u32 = 392;
    pub const D_LA: u32 = 440;
    pub const D_SI: u32 = 494;

    pub const M_DO: u32 = 523;
    pub const M_RE: u32 = 587;
    pub const M_MI: u32 = 659;
    pub const M_FA: u32 = 698;
    pub const M_SO: u32 = 784;
    pub const M_LA: u32 = 880;
    pub const M_SI: u32 = 988;

    pub const H_DO: u32 = 1046;
    pub const H_RE: u32 = 1157;
    pub const H_MI: u32 = 1318;
    pub const H_FA: u32 = 1397;
    pub const H_SO: u32 = 1568;
    pub const H_LA: u32 = 1760;
    pub const H_SI: u32 = 1976;

    pub const music_1: [u32; 7] = [M_DO, M_RE, M_MI, M_FA, M_SO, M_LA, M_SI];
}
