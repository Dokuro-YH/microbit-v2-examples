#![no_std]
#![no_main]

use microbit_v2_examples as _;

use microbit::board::Board;
use microbit::display::blocking::Display;
use microbit::hal::timer::Timer;

const ROULETTE_PIXELS: [(usize, usize); 16] = [
    (0, 0),
    (0, 1),
    (0, 2),
    (0, 3),
    (0, 4),
    (1, 4),
    (2, 4),
    (3, 4),
    (4, 4),
    (4, 3),
    (4, 2),
    (4, 1),
    (4, 0),
    (3, 0),
    (2, 0),
    (1, 0),
];

#[cortex_m_rt::entry]
fn main() -> ! {
    let board = Board::take().unwrap();

    let mut timer = Timer::new(board.TIMER0);
    let mut display = Display::new(board.display_pins);
    let mut led_display = [
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ];

    let duration_ms = 100_u32;
    let mut last_led = (0, 0);
    loop {
        for current_led in ROULETTE_PIXELS.iter() {
            led_display[last_led.0][last_led.1] = 0;
            led_display[current_led.0][current_led.1] = 1;
            display.show(&mut timer, led_display, duration_ms);
            last_led = *current_led;
        }
    }
}
