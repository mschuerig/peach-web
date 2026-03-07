// This crate is a C-style FFI boundary called from JavaScript AudioWorklet.
// All unsafe functions are called exclusively by the JS processor with valid pointers.
#![allow(clippy::missing_safety_doc)]

use oxisynth::{MidiEvent, SoundFont, Synth, SynthDescriptor};
use std::io::Cursor;

/// Allocate `size` bytes in WASM linear memory and return a pointer.
/// The AudioWorklet JS uses this to copy SoundFont data into WASM memory.
#[unsafe(no_mangle)]
pub extern "C" fn alloc(size: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

/// Deallocate memory previously allocated by `alloc`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dealloc(ptr: *mut u8, size: usize) {
    unsafe {
        drop(Vec::from_raw_parts(ptr, 0, size));
    }
}

/// Create a new Synth instance at the given sample rate.
/// Returns a pointer to the Synth, or null on failure.
#[unsafe(no_mangle)]
pub extern "C" fn synth_new(sample_rate: f32) -> *mut Synth {
    let desc = SynthDescriptor {
        sample_rate,
        gain: 1.0,
        reverb_active: false,
        chorus_active: false,
        ..Default::default()
    };
    match Synth::new(desc) {
        Ok(synth) => Box::into_raw(Box::new(synth)),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Load a SoundFont from raw bytes into the Synth.
/// Returns the SoundFont ID as i32, or -1 on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn synth_load_soundfont(
    synth: *mut Synth,
    data: *const u8,
    len: usize,
) -> i32 {
    let synth = match unsafe { synth.as_mut() } {
        Some(s) => s,
        None => return -1,
    };

    let bytes = unsafe { std::slice::from_raw_parts(data, len) }.to_vec();
    let mut cursor = Cursor::new(bytes);

    match SoundFont::load(&mut cursor) {
        Ok(sf) => {
            let _ = synth.add_font(sf, true);
            0
        }
        Err(_) => -1,
    }
}

/// Select a program (bank + preset) on channel 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn synth_select_program(synth: *mut Synth, bank: u32, preset: u8) {
    if let Some(synth) = unsafe { synth.as_mut() } {
        let _ = synth.send_event(MidiEvent::ProgramChange {
            channel: 0,
            program_id: preset,
        });
        let _ = synth.select_bank(0, bank);
    }
}

/// Send a NoteOn event on channel 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn synth_note_on(synth: *mut Synth, key: u8, vel: u8) {
    if let Some(synth) = unsafe { synth.as_mut() } {
        let _ = synth.send_event(MidiEvent::NoteOn {
            channel: 0,
            key,
            vel,
        });
    }
}

/// Send a NoteOff event on channel 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn synth_note_off(synth: *mut Synth, key: u8) {
    if let Some(synth) = unsafe { synth.as_mut() } {
        let _ = synth.send_event(MidiEvent::NoteOff { channel: 0, key });
    }
}

/// Send a PitchBend event on channel 0.
/// `bend_value`: 0-16383, center = 8192.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn synth_pitch_bend(synth: *mut Synth, bend_value: u16) {
    if let Some(synth) = unsafe { synth.as_mut() } {
        let _ = synth.send_event(MidiEvent::PitchBend {
            channel: 0,
            value: bend_value,
        });
    }
}

/// Render `len` audio samples into the provided left/right buffers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn synth_render(
    synth: *mut Synth,
    left: *mut f32,
    right: *mut f32,
    len: usize,
) {
    if let Some(synth) = unsafe { synth.as_mut() } {
        let left_buf = unsafe { std::slice::from_raw_parts_mut(left, len) };
        let right_buf = unsafe { std::slice::from_raw_parts_mut(right, len) };
        synth.write_f32(len, left_buf, 0, 1, right_buf, 0, 1);
    }
}

/// Send AllNotesOff on channel 0 to stop all voices.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn synth_all_notes_off(synth: *mut Synth) {
    if let Some(synth) = unsafe { synth.as_mut() } {
        let _ = synth.send_event(MidiEvent::AllNotesOff { channel: 0 });
    }
}
