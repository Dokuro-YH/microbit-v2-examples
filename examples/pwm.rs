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

// const DO0: u32 = 262;
// const RE0: u32 = 294;
// const MI0: u32 = 330;
// const FA0: u32 = 349;
// const SOL0: u32 = 392;
// const LA0: u32 = 440;
// const SI0: u32 = 494;
// const DO: u32 = 523;
// const RE: u32 = 587;
// const MI: u32 = 659;
// const FA: u32 = 698;
// const SOL: u32 = 784;
// const LA: u32 = 880;
// const SI: u32 = 988;
// const DO2: u32 = 1047;
// const RE2: u32 = 1177;
// const STOP: u32 = 36000;

// const music: [(u32, u32); 8] = [
//     (DO, 8),
//     (RE, 8),
//     (MI, 8),
//     (FA, 8),
//     (SOL, 8),
//     (LA, 8),
//     (SI, 8),
//     (DO2, 8),
// ];

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
        .set_prescaler(Prescaler::Div1)
        .set_period(20.hz())
        .set_max_duty(10)
        .enable();

    let mut timer = Timer::new(board.TIMER0);

    pwm.set_duty_on_common(5);
    loop {
        // for (note, th) in music {
        //     pwm.set_duty_on_common(note * 100);
        //     timer.delay_ms(th * 100_u32);
        // }
        for i in 0..1000 {
            rprintln!("{}hz", i);
            pwm.set_period(i.hz());
            pwm.set_duty_on_common(5);
            timer.delay_ms(100_u32);
        }

        // pwm.set_duty_on_common(pwm.max_duty());
        rprintln!("Restart!");
        timer.delay_ms(3000_u32);
    }
}
