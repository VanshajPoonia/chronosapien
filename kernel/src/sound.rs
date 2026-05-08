//! PC speaker tones through PIT channel 2.
//!
//! The PC speaker is gated by PIT channel 2. Channel 2 generates a square wave
//! at the requested frequency, and port 0x61 decides whether that wave reaches
//! the speaker.

use crate::io::{inb, outb};
use crate::theme::Era;
use crate::timer;

pub const MIN_FREQUENCY_HZ: u32 = 20;
pub const MAX_FREQUENCY_HZ: u32 = 20_000;

const PIT_INPUT_HZ: u32 = 1_193_182;
const PIT_COMMAND_PORT: u16 = 0x43;
const PIT_CHANNEL_2_PORT: u16 = 0x42;
const PIT_CHANNEL_2_MODE_3: u8 = 0b1011_0110;

const SPEAKER_PORT: u16 = 0x61;
const SPEAKER_ENABLE_BITS: u8 = 0b0000_0011;

const REST_MS: u64 = 35;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BeepError {
    FrequencyOutOfRange,
}

pub fn beep(frequency_hz: u32, duration_ms: u64) -> Result<(), BeepError> {
    if !is_valid_frequency(frequency_hz) {
        return Err(BeepError::FrequencyOutOfRange);
    }

    let divisor = frequency_to_divisor(frequency_hz);
    let [low_byte, high_byte] = divisor.to_le_bytes();

    crate::serial_println!(
        "[CHRONO] sound: beep {}hz {}ms",
        frequency_hz,
        duration_ms
    );

    // SAFETY: Ports 0x43 and 0x42 are the PIT command/data ports used to
    // program channel 2 on PC-compatible hardware. Port 0x61 gates the speaker:
    // bit 0 enables the PIT channel 2 gate, and bit 1 connects the speaker
    // output path. Clearing those two bits silences the tone afterward.
    unsafe {
        outb(PIT_COMMAND_PORT, PIT_CHANNEL_2_MODE_3);
        outb(PIT_CHANNEL_2_PORT, low_byte);
        outb(PIT_CHANNEL_2_PORT, high_byte);

        let speaker_state = inb(SPEAKER_PORT);
        outb(SPEAKER_PORT, speaker_state | SPEAKER_ENABLE_BITS);
    }

    timer::sleep_ms(duration_ms);
    silence();

    Ok(())
}

pub fn play_boot_chime(era: Era) {
    match era {
        Era::Eighties => {
            play_tone(880, 90);
            rest();
            play_tone(660, 90);
            rest();
            play_tone(440, 140);
        }
        Era::Future => {
            play_tone(1760, 180);
        }
        Era::Nineties | Era::TwoThousands => {
            play_tone(523, 80);
            rest();
            play_tone(659, 80);
            rest();
            play_tone(784, 120);
        }
    }
}

pub fn is_valid_frequency(frequency_hz: u32) -> bool {
    (MIN_FREQUENCY_HZ..=MAX_FREQUENCY_HZ).contains(&frequency_hz)
}

fn play_tone(frequency_hz: u32, duration_ms: u64) {
    let _ = beep(frequency_hz, duration_ms);
}

fn rest() {
    silence();
    timer::sleep_ms(REST_MS);
}

fn silence() {
    // SAFETY: Port 0x61 is the PC speaker control port. Preserving unrelated
    // bits keeps other platform flags intact while disconnecting the speaker.
    unsafe {
        let speaker_state = inb(SPEAKER_PORT);
        outb(SPEAKER_PORT, speaker_state & !SPEAKER_ENABLE_BITS);
    }
}

fn frequency_to_divisor(frequency_hz: u32) -> u16 {
    ((PIT_INPUT_HZ + (frequency_hz / 2)) / frequency_hz) as u16
}
