#![no_main]
#![no_std]

use panic_halt as _;

use microbit::hal::gpio::Level;
use microbit::hal::prelude::*;
use microbit::hal::pwm::{Channel, CounterMode, Pwm, Seq};
use microbit::hal::timer::Timer;
use microbit::Board;

const TONES: [[u32; 14]; 3] = [
    //   C    D     E     F     G     A     B
    //   1.   2.    3.    4.    5.    6.    7.
    [360000, 131, 147,  165,  175,  196,  221,  248,  278,  312,  330,  371,  416,  476],
    //   1    2     3     4     5     6     7 
    [360000, 262, 294,  330,  350,  393,  441,  496,  556,  624,  661,  742,  833,  935],
    //   1'   2'    3'    4'    5'    6'    7'
    [360000, 525, 589,  661,  700,  786,  882,  990,  1112, 1248, 1322, 1484, 1665, 1869],
  ];

pub const XIAO_XING_XING: [usize; 32] = [
    1, 1, 5, 5, 6, 6, 5, 0, 
    4, 4, 3, 3, 2, 2, 1, 0,
    5, 5, 4, 4, 3, 3, 2, 0,
    5, 5, 4, 4, 3, 3, 2, 0,
];

#[cortex_m_rt::entry]
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
        .set_period(1.hz())
        .set_max_duty(100)
        .enable();

    let mut timer = Timer::new(board.TIMER0);
    let volume = 50;

    loop {
        for note in XIAO_XING_XING {
            let pitch = 1;
            let tone = TONES[pitch][note];
            let delay = 500_u32;

            pwm.set_period(tone.hz());
            pwm.set_duty_on_common((pwm.max_duty() as u32 * volume as u32 / 100) as u16);

            timer.delay_ms(delay);
        }
        timer.delay_ms(1000_u32);
    }
}