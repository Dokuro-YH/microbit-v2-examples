use cortex_m::prelude::_embedded_hal_blocking_delay_DelayMs;
use microbit::hal::{pwm, time::U32Ext, timer};

/// A-B, C-G
const PERIODS: &[u32] = &[440000, 493880, 261630, 293660, 329630, 349230, 392000];
/// #A-#B, #C-#G
const PERIODS_SHARP: &[u32] = &[466160, 0, 277180, 311130, 0, 369990, 415300];

pub struct Music<PWM: pwm::Instance, TIM: timer::Instance> {
    bpm: u8,
    buzzer: pwm::Pwm<PWM>,
    timer: timer::Timer<TIM>,
}

impl<PWM: pwm::Instance, TIM: timer::Instance> Music<PWM, TIM> {
    pub fn new(buzzer: pwm::Pwm<PWM>, timer: timer::Timer<TIM>) -> Self {
        Music {
            bpm: 120, // one note with 500ms
            buzzer,
            timer,
        }
    }

    pub fn bpm(&self) -> u8 {
        self.bpm
    }

    /// Set music play note BPM (60 - 240)
    pub fn set_bpm(&mut self, bpm: u8) -> &mut Self {
        self.bpm = bpm.max(60).min(240);
        self
    }

    /// Play notes split with whitespace.
    ///
    /// Note: (#|b)(octave)(:duration)
    ///
    /// Example
    /// ```rust
    /// # use microbit_v2_examples::music::Music;
    /// # let mut music = Music::default();
    /// music.play(r#"#c4 c4 g4 g4 a4 a4 g4 -
    ///               f4 f4 e4 e4 d4 d4 c4 -
    ///               g4 g4 f4 f4 e4 e4 d4 -
    ///               g4 g4 f4 f4 e4 e4 d4 -"#)
    /// ```
    pub fn play(&mut self, tunes: &str) {
        for note in tunes.split_ascii_whitespace() {
            let duration = self.play_note(note);
            self.timer.delay_ms(duration.min(1));
        }
    }

    /// play note and return duration
    pub fn play_note(&mut self, mut note: &str) -> u32 {
        let mut duration = 0;

        // parse the duration
        if let Some((n, tempo_str)) = note.split_once(':') {
            for c in tempo_str.chars() {
                duration *= 10;
                duration += c as u32 & 0xf;
            }

            note = n;
        }

        let note_bytes = note.as_bytes();
        let note_len = note_bytes.len();

        // we'll represent the note as an integer (A=0, G=6)
        // parse the note
        let mut note_index = match note_bytes[0] {
            c @ (b'a'..=b'g' | b'A'..=b'G') => (c & 0xf) as usize - 1,
            c => panic!("invalid note {}, pattern: (#|b)(octave)(:duration)", c),
        };

        let mut sharp = false;
        let mut octave = 0;

        // parse sharp or flat
        if 1 < note_len && (note_bytes[1] == b'#' || note_bytes[1] == b'b') {
            if note_bytes[1] == b'b' {
                if note_index == 0 {
                    note_index = 6;
                } else {
                    note_index -= 1;
                }

                if note_index == 1 {
                    octave -= 1;
                }
            }

            sharp = true;
        }

        // make the octave relative to octave 4
        octave -= 4;

        if note_index < 10 {
            let period;
            if sharp {
                if octave > 0 {
                    period = PERIODS_SHARP[note_index] >> octave;
                } else {
                    period = PERIODS_SHARP[note_index] << -octave;
                }
            } else if octave > 0 {
                period = PERIODS[note_index] >> octave;
            } else {
                period = PERIODS[note_index] << -octave;
            }
            self.buzzer.set_period(period.hz());
            self.buzzer.enable();
        } else {
            self.buzzer.disable();
        }

        let beat_ms = 60000 / self.bpm as u32;
        beat_ms * duration.min(1)
    }
}
