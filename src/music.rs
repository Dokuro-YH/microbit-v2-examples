use core::ops::RangeBounds;

use microbit::hal::{
    gpio::{Output, Pin, PushPull},
    pwm::{self, Channel, CounterMode, Pwm},
    time::U32Ext,
    timer::{self},
};
const DEFAULT_BPM: u32 = 120; // 500ms
/// A-B, C-G
const PERIODS: &[u32] = &[440, 494, 262, 294, 330, 349, 392];
/// A#, -, C#, D#, -, F#, G#
const PERIODS_SHARP: &[u32] = &[466, 0, 277, 311, 0, 370, 415];

pub struct Music<PWM: pwm::Instance, TIMER: timer::Instance> {
    pos: usize,
    notes: &'static [u8],
    bpm: u32,
    volume: u32,
    timer: TIMER,
    buzzer: pwm::Pwm<PWM>,
}

impl<PWM: pwm::Instance, TIMER: timer::Instance> Music<PWM, TIMER> {
    pub fn new(pin: Pin<Output<PushPull>>, pwm: PWM, timer: TIMER) -> Self {
        let buzzer = Pwm::new(pwm);
        buzzer.set_output_pin(Channel::C0, pin);
        buzzer.set_counter_mode(CounterMode::UpAndDown);
        let mut music = Music {
            pos: 0,
            notes: &[],
            volume: 90,
            bpm: DEFAULT_BPM,
            buzzer,
            timer,
        };
        music.initialise();
        music
    }

    fn initialise(&mut self) {
        let timer0 = self.timer.as_timer0();
        // enable compare interrupt
        timer0.intenset.write(|w| w.compare0().set());
        // set frequency to 1Mhz
        timer0.prescaler.write(|w| unsafe { w.bits(4) });
        // set as 32 bits
        timer0.bitmode.write(|w| w.bitmode()._32bit());
        // enable auto clear
        timer0.shorts.write(|w| w.compare0_clear().enabled());
        // reset compare register
        timer0.events_compare[0].reset();
    }

    pub fn volume(&self) -> &u32 {
        &self.volume
    }

    pub fn set_volume(&mut self, volume: u32) -> &mut Self {
        self.volume = volume.clamp(0, 100);
        self
    }

    /// Update the music play state
    pub fn next_tick(&mut self) {
        let timer0 = self.timer.as_timer0();
        let reg = &timer0.events_compare[0];
        let fired = reg.read().bits() != 0;
        if fired {
            let note = self.get_next_note();
            let note_ms = 500; // 500ms
            let cycles = note_ms * 1000;
            timer0.cc[0].write(|w| unsafe { w.bits(cycles) });
            defmt::debug!("play next note: {}", note);
            reg.reset();
        }
    }

    /// Play notes split with whitespace.
    ///
    /// Note: (#|b)(pitch)(octave)(:duration)
    ///
    /// pitch: A-G/a-g
    /// octave: 0-9
    /// duration: 1-10
    ///
    /// Example
    /// ```no_run
    /// music.play(r#"c4 c4 g4 g4 a4 a4 g4 -
    ///               f4 f4 e4 e4 d4 d4 c4 -
    ///               g4 g4 f4 f4 e4 e4 d4 -
    ///               g4 g4 f4 f4 e4 e4 d4 -"#)
    /// ```
    pub fn play(&mut self, notes: &'static [u8], bpm: u32) {
        self.reset_timer();
        self.notes = notes;
        self.bpm = bpm;
        self.pos = 0;
        self.start_timer();
    }

    pub fn stop(&mut self) {
        self.stop_timer();
    }

    fn reset_timer(&self) {
        let timer0 = self.timer.as_timer0();
        // Stop the timer.
        timer0.tasks_stop.write(|w| unsafe { w.bits(1) });
        // Clear the counter value.
        timer0.tasks_clear.write(|w| unsafe { w.bits(1) });
        // Reset the compare register.
        timer0.events_compare[0].reset();
    }

    fn stop_timer(&self) {
        let timer0 = self.timer.as_timer0();
        // Stop the timer.
        timer0.tasks_stop.write(|w| unsafe { w.bits(1) });
    }

    fn start_timer(&self) {
        let timer0 = self.timer.as_timer0();
        // Start timer.
        timer0.tasks_start.write(|w| unsafe { w.bits(1) });
    }

    fn get_next_note(&self) -> Option<&[u8]> {
        let notes_len = self.notes.len();
        let mut start = 0;
        let mut end = 0;
        for pos in self.pos..notes_len {
            let c = self.notes[pos] as char;
            if !c.is_ascii_whitespace() {
                start = pos;
                break;
            }
        }

        if start == 0 {
            return None;
        }

        for pos in start..notes_len {
            let c = self.notes[pos] as char;
            if c.is_ascii_whitespace() {
                end = pos;
                break;
            }
        }
        if end == 0 {
            end = notes_len;
        }
        Some(&self.notes[start..end])
    }

    // /// play note and return the duration(ms)
    // fn play_note(&mut self, mut note: &[u8]) -> u32 {
    //     self.buzzer.disable();
    //     // parse the duration
    //     let duration = 1;
    //     // if let Some((prefix, suffix)) = note.split_once(':') {
    //     //     note = prefix;

    //     //     if let Some(i) = read_int(suffix) {
    //     //         duration = i;
    //     //     }
    //     // }

    //     // parse the bpm*2 symbol is: (
    //     while let Some(s) = read_char(note, b'(', false) {
    //         self.bpm *= 2;
    //         note = s.0;
    //     }

    //     let delay_ms = (60000 / self.bpm) * duration;

    //     // parse the bpm/2 symbol is: )
    //     while let Some(s) = read_char(note, b')', true) {
    //         self.bpm /= 2;
    //         note = s.0;
    //     }

    //     if let Some(period) = self.parse_note(note) {
    //         self.buzzer.set_period(period.hz());
    //         let duty = self.buzzer.max_duty() as u32 * self.volume / 200;
    //         self.buzzer.set_duty_on_common(duty as u16);
    //         self.buzzer.enable();
    //         defmt::info!(
    //             "volume: {}, {}: {}hz {}ms",
    //             self.volume,
    //             note,
    //             period,
    //             delay_ms
    //         );
    //     } else {
    //         defmt::info!("volume: {}, {}: 0hz {}ms", self.volume, note, delay_ms);
    //     }
    //     delay_ms
    // }

    // /// parse the note
    // fn parse_note(&mut self, note: &[u8]) -> Option<u32> {
    //     // parse the octave
    //     let (note, octave) = read_char_if(note, b'0'..=b'9', true)?;

    //     // parse the pitch
    //     let (note, pitch) = read_char_if(note, b'a'..=b'g', true)?;

    //     // parse the sharp or flat
    //     let mut octave = (octave & 0xf) as i32;
    //     let mut period_index = ((pitch & 0x1f) - 1) as usize;
    //     let mut sharp = false;
    //     if let Some((_, c)) = read_sharp_or_flat(note, false) {
    //         if c == b'#' {
    //         } else if c == b'b' {
    //             if period_index == 0 {
    //                 period_index = 6;
    //             } else {
    //                 period_index -= 1;
    //             }

    //             if period_index == 1 {
    //                 octave -= 1;
    //             }
    //         }

    //         sharp = true;
    //     }

    //     // make the octave relative to octave 4
    //     octave -= 4;

    //     let periods = if sharp { PERIODS_SHARP } else { PERIODS };
    //     let period = if octave > 0 {
    //         periods[period_index] << octave
    //     } else {
    //         periods[period_index] >> -octave
    //     };

    //     Some(period)
    // }
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
