#![no_main]
#![no_std]

use microbit::{hal::gpio::Level, pac, Board};

use microbit_v2_examples::{
    self as _,
    monotonic::{ExtU32, MonoTimer},
    music::Music,
};

#[rtic::app(device = microbit::pac, dispatchers = [RTC0, RTC1, RTC2])]
mod app {
    use super::*;

    #[monotonic(binds = TIMER0, default = true)]
    type Tnoic = MonoTimer<microbit::pac::TIMER0>;

    #[shared]
    struct Shared {
        music: Music<pac::PWM0, pac::TIMER1>,
        tones: &'static str,
    }

    #[local]
    struct Local {}

    const XIAO_XING_XING: &str = r#"
    c4 c4 g4 g4 a4 a4 g4 -
    f4 f4 e4 e4 d4 d4 c4 -
    g4 g4 f4 f4 e4 e4 d4 -
    g4 g4 f4 f4 e4 e4 d4 -
    "#;

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        let board = Board::new(cx.device, cx.core);
        let mono = MonoTimer::new(board.TIMER0);

        let music = Music::new(
            board
                .speaker_pin
                .into_push_pull_output(Level::High)
                .degrade(),
            board.PWM0,
            board.TIMER1,
        );
        let tones = XIAO_XING_XING;

        (Shared { music, tones }, Local {}, init::Monotonics(mono))
    }

    #[task(shared = [music, &tones])]
    fn play_music(mut cx: play_music::Context) {
        cx.shared.music.lock(|music| music.play(&cx.shared.tones));
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            let _ = play_music::spawn_after(1.secs());
        }
    }
}
