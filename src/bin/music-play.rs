#![no_main]
#![no_std]

use microbit::{hal::gpio::Level, pac, Board};

use microbit_v2_examples::{self as _, monotonic::MonoTimer, music::Music};

#[rtic::app(device = microbit::pac, dispatchers = [RTC0, RTC1, RTC2])]
mod app {
    use microbit::hal::gpiote::Gpiote;

    use super::*;

    #[monotonic(binds = TIMER0, default = true)]
    type Tnoic = MonoTimer<microbit::pac::TIMER0>;

    #[shared]
    struct Shared {
        music: Music<pac::PWM0, pac::TIMER1>,
    }

    #[local]
    struct Local {
        gpiote: Gpiote,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        let board = Board::new(cx.device, cx.core);
        let btn_a_pin = board.buttons.button_a.degrade();
        let btn_b_pin = board.buttons.button_b.degrade();
        let speaker_pin = board
            .speaker_pin
            .into_push_pull_output(Level::High)
            .degrade();
        let music = Music::new(speaker_pin, board.PWM0, board.TIMER1);
        let mono = MonoTimer::new(board.TIMER0);
        let gpiote = Gpiote::new(board.GPIOTE);

        gpiote
            .channel0()
            .input_pin(&btn_a_pin)
            .hi_to_lo()
            .enable_interrupt();
        gpiote
            .channel1()
            .input_pin(&btn_b_pin)
            .hi_to_lo()
            .enable_interrupt();

        (Shared { music }, Local { gpiote }, init::Monotonics(mono))
    }

    #[task(binds = TIMER1, shared = [music])]
    fn play(mut cx: play::Context) {
        cx.shared.music.lock(|music| music.next_tick());
    }

    #[task(binds = GPIOTE, local = [gpiote], shared = [music])]
    fn gpiote(mut cx: gpiote::Context) {
        let gpiote = cx.local.gpiote;
        let apressed = gpiote.channel0().is_event_triggered();
        let bpressed = gpiote.channel1().is_event_triggered();

        defmt::info!(
            "Button pressed {:?}",
            match (apressed, bpressed) {
                (false, false) => "",
                (true, false) => "A",
                (false, true) => "B",
                (true, true) => "A + B",
            }
        );

        cx.shared.music.lock(|music| {
            if apressed {
                music.set_volume(music.volume() - 10);
            } else if bpressed {
                music.set_volume(music.volume() + 10);
            }
        });

        gpiote.reset_events();
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            cortex_m::asm::wfi();
            // let _ = play::spawn_after(1.secs());
        }
    }
}
