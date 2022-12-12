use core::ops::RangeBounds;

use embedded_hal::blocking::delay::DelayMs;
use microbit::hal::{pwm, time::U32Ext, timer};

/// A-B, C-G
const PERIODS: &[u32] = &[440, 494, 262, 294, 330, 349, 392];
/// A#, -, C#, D#, -, F#, G#
const PERIODS_SHARP: &[u32] = &[466, 0, 277, 311, 0, 370, 415];

pub struct Music<PWM: pwm::Instance, TIM: timer::Instance> {
    bpm: u32,
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

    pub fn bpm(&self) -> u32 {
        self.bpm
    }

    /// Set music play note BPM
    pub fn set_bpm(&mut self, bpm: u32) -> &mut Self {
        self.bpm = bpm;
        self
    }

    /// Play notes split with whitespace.
    ///
    /// Note: (pitch)(octave)(#|b)(:duration)
    ///
    /// pitch: A-G/a-g
    /// octave: 0-9
    /// duration: 1-10
    ///
    /// Example
    /// ```rust
    /// # use microbit_v2_examples::music::Music;
    /// # let mut music = Music::default();
    /// music.play(r#"c4 c4 g4 g4 a4 a4 g4 -
    ///               f4 f4 e4 e4 d4 d4 c4 -
    ///               g4 g4 f4 f4 e4 e4 d4 -
    ///               g4 g4 f4 f4 e4 e4 d4 -"#)
    /// ```
    pub fn play(&mut self, tunes: &str) {
        for note in tunes.split_ascii_whitespace() {
            let duration = self.play_note(note);
            self.timer.delay_ms(duration)
        }
    }

    /// play note and return the duration(ms)
    pub fn play_note(&mut self, mut note: &str) -> u32 {
        self.buzzer.disable();
        // parse the duration
        let mut duration = 1;
        if let Some((prefix, suffix)) = note.split_once(':') {
            note = prefix;

            if let Some(i) = read_int(suffix) {
                duration = i;
            }
        }

        let mut note_str = note.as_bytes();
        // parse the bpm*2 symbol is: (
        while let Some(s) = read_char(note_str, b'(', false) {
            self.bpm *= 2;
            note_str = s.0;
        }
        // parse the bpm/2 symbol is: )
        while let Some(s) = read_char(note_str, b')', true) {
            self.bpm /= 2;
            note_str = s.0;
        }

        let note_len = note_str.len();
        if note_len >= 2 {
            // parse the octave
            let mut octave = 4;
            if let Some(s) = read_char_if(note_str, b'0'..=b'9', true) {
                note_str = s.0;
                octave = (s.1 & 0xf) as i32;
            }

            // parse the note
            let mut period_index = 3;
            if let Some(s) = read_char_if(note_str, b'a'..=b'g', true) {
                note_str = s.0;
                period_index = (s.1 & 0xf) as usize - 1;
            }

            // parse the sharp or flat
            let mut sharp = false;
            if let Some((_, c)) = read_sharp_or_flat(note_str, false) {
                if c == b'#' {
                } else if c == b'b' {
                    if period_index == 0 {
                        period_index = 6;
                    } else {
                        period_index -= 1;
                    }

                    if period_index == 1 {
                        octave -= 1;
                    }
                }

                sharp = true;
            }

            // make the octave relative to octave 4
            octave -= 4;

            let periods = if sharp { PERIODS_SHARP } else { PERIODS };
            let period = if octave > 0 {
                periods[period_index] >> octave
            } else {
                periods[period_index] << -octave
            };
            defmt::info!("{}: {}", note, period);
            self.buzzer.set_period(period.hz());
            self.buzzer.set_duty_on_common(self.buzzer.max_duty() / 2);
            self.buzzer.enable();
        }
        let delay_ms = (60000 / self.bpm) * duration;
        defmt::info!("delay {}ms", delay_ms);
        delay_ms
    }
}

/// parse sharp or flat
fn read_sharp_or_flat(src: &[u8], revease: bool) -> Option<(&[u8], u8)> {
    read_char(src, b'#', revease).or_else(|| read_char(src, b'b', revease))
}

/// parse one char
fn read_char_if<R: RangeBounds<u8>>(src: &[u8], range: R, revease: bool) -> Option<(&[u8], u8)> {
    let src_len = src.len();
    if src_len == 0 {
        return None;
    };

    let (dest, idx) = if revease {
        (&src[..src_len - 1], src_len - 1)
    } else {
        (&src[1..], 0)
    };

    let s = src[idx];
    if range.contains(&s) {
        Some((dest, s))
    } else {
        None
    }
}

/// parse one char
fn read_char(src: &[u8], c: u8, revease: bool) -> Option<(&[u8], u8)> {
    let src_len = src.len();
    if src_len == 0 {
        return None;
    };

    if revease && src[src_len - 1] == c {
        Some((&src[0..(src_len - 1)], c))
    } else if src[0] == c {
        Some((&src[1..], c))
    } else {
        None
    }
}

/// parse int
fn read_int(str: &str) -> Option<u32> {
    let str = str.as_bytes();
    let str_len = str.len();
    let mut pos = 0;
    let mut n = 0;
    while pos < str_len {
        pos += 1;
        match str[pos] {
            c @ b'0'..=b'9' => {
                n *= 10;
                n += (c & 0xf) as u32;
            }
            _ => break,
        }
    }

    if n > 0 {
        Some(n)
    } else {
        None
    }
}
