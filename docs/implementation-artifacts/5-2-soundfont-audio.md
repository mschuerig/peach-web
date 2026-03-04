# Story 5.2: SoundFont Audio

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a musician,
I want to hear richer instrument sounds during training,
so that training feels more musical and the sound source setting becomes meaningful.

## Acceptance Criteria

1. **Given** the app loads **When** SoundFont loading begins **Then** the SoundFont file is fetched asynchronously via `fetch()` **And** the Start Page is interactive immediately with oscillator fallback **And** no loading indicator is shown — loading is invisible to the user.

2. **Given** the SoundFont has loaded successfully **When** training starts **Then** the SoundFontNotePlayer renders notes using OxiSynth with the selected preset **And** the swap from oscillator to SoundFont is silent — no interruption.

3. **Given** the SoundFont fails to load **When** training starts **Then** the OscillatorNotePlayer continues as the fallback **And** no error message is shown (oscillators are a valid sound source).

4. **Given** SoundFontNotePlayer is active **When** a note is played **Then** the AudioBuffer is rendered from the SoundFont preset via OxiSynth **And** playback uses AudioBufferSourceNode.

5. **Given** SoundFontNotePlayer is active during pitch matching **When** adjustFrequency is called **Then** the pitch is adjusted via `.detune` on the AudioBufferSourceNode (within +/-20 cents range).

6. **Given** the SoundFont has been loaded once **When** the app is visited again **Then** the browser cache serves the SoundFont — load is near-instant.

7. **Given** the sound source setting (FR25, from Epic 2) **When** the user selects a different sound source **Then** the SoundFont preset changes accordingly **And** the change takes effect on the next note played.

## Tasks / Subtasks

- [ ] Task 1: Add OxiSynth dependency and SoundFont asset (AC: #4)
  - [ ] 1.1 Add `oxisynth = "0.1.0"` to `web/Cargo.toml`
  - [ ] 1.2 Add required web-sys features: `AudioBuffer`, `AudioBufferSourceNode`
  - [ ] 1.3 Source a small SF2 file (piano or similar, ideally < 5 MB) and place in `web/assets/soundfont/`
  - [ ] 1.4 Configure Trunk to copy the SF2 asset to `dist/` (add `[[hooks]]` or `data-` attributes in `index.html`)

- [ ] Task 2: Implement SoundFontNotePlayer adapter (AC: #2, #4, #5)
  - [ ] 2.1 Create `web/src/adapters/audio_soundfont.rs`
  - [ ] 2.2 Implement `SoundFontNotePlayer` struct holding `Rc<RefCell<Synth>>`, `Rc<RefCell<AudioContextManager>>`, and `SoundFontId`
  - [ ] 2.3 Implement `NotePlayer` trait: `play()` renders MIDI note via OxiSynth `read_next()` into left/right f32 buffers, creates `AudioBuffer` via `ctx.create_buffer()`, copies channel data, plays via `AudioBufferSourceNode`
  - [ ] 2.4 Implement `play_for_duration()` — render exactly `duration * sample_rate` note-on samples + release tail, schedule stop via `AudioBufferSourceNode` (finite buffer = natural stop)
  - [ ] 2.5 Implement `SoundFontPlaybackHandle` with `stop()` (calls `source.stop()`) and `adjust_frequency()` (sets `source.detune().set_value(cents)` — convert frequency delta to cents)
  - [ ] 2.6 Use velocity parameter (unlike oscillator which ignores it) — pass to `MidiEvent::NoteOn { vel }`
  - [ ] 2.7 Register module in `web/src/adapters/mod.rs`

- [ ] Task 3: Implement unified NotePlayer enum wrapper (AC: #2, #3)
  - [ ] 3.1 Create `web/src/adapters/note_player.rs` with `UnifiedNotePlayer` enum wrapping `OscillatorNotePlayer` and `SoundFontNotePlayer`
  - [ ] 3.2 Create `UnifiedPlaybackHandle` enum wrapping `OscillatorPlaybackHandle` and `SoundFontPlaybackHandle`
  - [ ] 3.3 Implement `NotePlayer` trait on `UnifiedNotePlayer` delegating to inner variant
  - [ ] 3.4 Implement `PlaybackHandle` trait on `UnifiedPlaybackHandle` delegating to inner variant
  - [ ] 3.5 Add factory function: `create_note_player(sound_source: &SoundSourceID, audio_ctx, synth_option) -> UnifiedNotePlayer`

- [ ] Task 4: Async SoundFont loading at app startup (AC: #1, #6)
  - [ ] 4.1 In `web/src/app.rs`, add `soundfont_synth: RwSignal<Option<Rc<RefCell<Synth>>>>` signal (local, starts as `None`)
  - [ ] 4.2 In `spawn_local` block, `fetch()` the SF2 file from `/assets/soundfont/<name>.sf2`
  - [ ] 4.3 Parse response bytes, load via `SoundFont::load(&mut Cursor::new(bytes))`, create `Synth`, `add_font()`, `select_program()` for default preset
  - [ ] 4.4 On success: set signal to `Some(synth)`. On failure: log warning, leave as `None` (oscillator fallback)
  - [ ] 4.5 Provide the synth signal as Leptos context for views to consume
  - [ ] 4.6 Browser caching handled automatically by `fetch()` — no special cache logic needed

- [ ] Task 5: Update training views to use UnifiedNotePlayer (AC: #2, #3)
  - [ ] 5.1 In `comparison_view.rs`: replace `OscillatorNotePlayer::new()` with `create_note_player()` that checks synth signal and sound source setting
  - [ ] 5.2 In `pitch_matching_view.rs`: same replacement. Change `tunable_handle` type from `Option<OscillatorPlaybackHandle>` to `Option<UnifiedPlaybackHandle>`
  - [ ] 5.3 Both views: read `peach.sound_source` from `LocalStorageSettings` to decide which variant to create
  - [ ] 5.4 If synth signal is `None` (still loading or failed), create `OscillatorNotePlayer` variant regardless of setting

- [ ] Task 6: Update Settings view sound source dropdown (AC: #7)
  - [ ] 6.1 Add SoundFont preset options to the sound source `<select>` element (currently only "Sine Oscillator")
  - [ ] 6.2 Parse `SoundSourceID` format: `"oscillator:sine"` for oscillator, `"sf2:<bank>:<preset>"` for SoundFont presets
  - [ ] 6.3 Sound source change auto-saves to `peach.sound_source` in localStorage (existing pattern)
  - [ ] 6.4 Change takes effect on next note played (next training session start or next comparison)

- [ ] Task 7: Verify and test (AC: all)
  - [ ] 7.1 `cargo test -p domain` — confirm no regressions
  - [ ] 7.2 `trunk build` — confirm WASM compilation with new OxiSynth dependency
  - [ ] 7.3 `cargo clippy` — zero warnings
  - [ ] 7.4 Manual browser test: app loads, start page interactive immediately, training works with oscillator while SF2 loads
  - [ ] 7.5 Manual browser test: after SF2 loads, start training — hear SoundFont audio instead of sine wave
  - [ ] 7.6 Manual browser test: pitch matching — slider adjusts pitch smoothly with SoundFont audio
  - [ ] 7.7 Manual browser test: change sound source in Settings, start training — new sound plays
  - [ ] 7.8 Manual browser test: disable network, reload — SF2 served from browser cache
  - [ ] 7.9 Manual browser test: block SF2 URL — training works with oscillator fallback, no error shown

## Dev Notes

### Architecture — Associated Type Problem and Solution

The `NotePlayer` trait uses an associated type `Handle`:
```rust
pub trait NotePlayer {
    type Handle: PlaybackHandle;
    fn play(&self, ...) -> Result<Self::Handle, AudioError>;
    // ...
}
```

This makes `dyn NotePlayer` impossible. Both training views currently reference concrete types (`OscillatorNotePlayer`, `OscillatorPlaybackHandle`). The solution is a **wrapper enum** pattern:

```rust
pub enum UnifiedNotePlayer {
    Oscillator(OscillatorNotePlayer),
    SoundFont(SoundFontNotePlayer),
}

pub enum UnifiedPlaybackHandle {
    Oscillator(OscillatorPlaybackHandle),
    SoundFont(SoundFontPlaybackHandle),
}
```

Implement `NotePlayer` on `UnifiedNotePlayer` with `type Handle = UnifiedPlaybackHandle`, delegating each method to the inner variant. This avoids changing the domain trait while allowing runtime switching.

### OxiSynth Integration Pattern

**Synth creation:**
```rust
use oxisynth::{Synth, SynthDescriptor, SoundFont, MidiEvent};
use std::io::Cursor;

let desc = SynthDescriptor {
    sample_rate: 44100.0,
    reverb_active: false,  // Save CPU
    chorus_active: false,   // Save CPU
    ..Default::default()
};
let mut synth = Synth::new(desc).unwrap();
let mut cursor = Cursor::new(sf2_bytes);
let font = SoundFont::load(&mut cursor).unwrap();
let sfont_id = synth.add_font(font, true);
synth.select_program(0, sfont_id, bank, preset).unwrap();
```

**Rendering a note to AudioBuffer:**
```rust
// Trigger note
synth.send_event(MidiEvent::NoteOn { channel: 0, key: midi_note, vel: velocity }).unwrap();

// Render samples
let num_samples = (sample_rate * duration_secs) as usize;
let mut left = Vec::with_capacity(num_samples);
let mut right = Vec::with_capacity(num_samples);
for _ in 0..num_samples {
    let (l, r) = synth.read_next();
    left.push(l);
    right.push(r);
}

// Release
synth.send_event(MidiEvent::NoteOff { channel: 0, key: midi_note }).unwrap();
// Render release tail...

// Create Web Audio buffer
let buffer = ctx.create_buffer(2, total_len as u32, sample_rate).unwrap();
buffer.copy_to_channel(&left, 0).unwrap();
buffer.copy_to_channel(&right, 1).unwrap();
```

**Playback with detune:**
```rust
let source = ctx.create_buffer_source().unwrap();
source.set_buffer(Some(&buffer));
source.detune().set_value(detune_cents as f32);
source.connect_with_audio_node(&ctx.destination()).unwrap();
source.start().unwrap();
```

### Frequency-to-MIDI and Detune Conversion

The domain uses `Frequency` (Hz) but OxiSynth needs MIDI note numbers. Conversion:
```rust
fn frequency_to_midi(freq: f64) -> u8 {
    (69.0 + 12.0 * (freq / 440.0).log2()).round() as u8
}

fn frequency_to_cents_from_midi(freq: f64, midi_note: u8) -> f64 {
    let midi_freq = 440.0 * 2.0_f64.powf((midi_note as f64 - 69.0) / 12.0);
    1200.0 * (freq / midi_freq).log2()
}
```

For `play()`: convert frequency to nearest MIDI note, render that note, then apply any fractional cent offset via `.detune`.

For `adjust_frequency()` in pitch matching: compute cent difference between new frequency and the originally rendered MIDI note's frequency, set `.detune` accordingly. The AC specifies +/-20 cents range which is well within AudioBufferSourceNode detune capabilities.

### SoundFont Loading — Async Fetch Pattern

Follow the established IndexedDB async init pattern from `app.rs`:
```rust
let soundfont_synth: RwSignal<Option<Rc<RefCell<Synth>>>> = RwSignal::new_local(None);
provide_context(soundfont_synth);

spawn_local(async move {
    match fetch_sf2_bytes("/assets/soundfont/piano.sf2").await {
        Ok(bytes) => {
            let mut cursor = Cursor::new(bytes);
            match SoundFont::load(&mut cursor) {
                Ok(font) => {
                    let mut synth = Synth::new(desc).unwrap();
                    let sfont_id = synth.add_font(font, true);
                    synth.select_program(0, sfont_id, 8, 80).unwrap();
                    soundfont_synth.set(Some(Rc::new(RefCell::new(synth))));
                }
                Err(e) => log::warn!("SoundFont parse failed: {e:?}"),
            }
        }
        Err(e) => log::warn!("SoundFont fetch failed: {e:?}"),
    }
});
```

The `fetch()` call uses standard `web_sys::Request` + `wasm_bindgen_futures::JsFuture`. Browser automatically caches the response for subsequent visits (AC #6).

### Concrete Type Coupling in PitchMatchingView

**Critical:** `pitch_matching_view.rs` stores the tunable handle with a concrete type:
```rust
let tunable_handle: Rc<RefCell<Option<OscillatorPlaybackHandle>>> = ...;
```

This MUST change to:
```rust
let tunable_handle: Rc<RefCell<Option<UnifiedPlaybackHandle>>> = ...;
```

Similarly, `comparison_view.rs` stores handles typed to `OscillatorPlaybackHandle` — update to `UnifiedPlaybackHandle`.

### Sound Source Setting Wiring

Currently disconnected: `peach.sound_source` exists in localStorage (set by Settings view) but training views never read it. They always create `OscillatorNotePlayer`.

The factory function bridges this gap:
```rust
pub fn create_note_player(
    sound_source: &str,
    audio_ctx: Rc<RefCell<AudioContextManager>>,
    synth: Option<Rc<RefCell<Synth>>>,
) -> UnifiedNotePlayer {
    match (sound_source, synth) {
        (s, Some(synth)) if s.starts_with("sf2:") => {
            // Parse bank:preset from sound_source, select program on synth
            UnifiedNotePlayer::SoundFont(SoundFontNotePlayer::new(audio_ctx, synth))
        }
        _ => UnifiedNotePlayer::Oscillator(OscillatorNotePlayer::new(audio_ctx)),
    }
}
```

### Performance Considerations

- **Audio onset < 50ms (NFR1):** Pre-rendering an AudioBuffer from OxiSynth takes time proportional to note duration. For a 1-second note at 44100 Hz, that's ~44100 iterations of `read_next()`. Benchmark this — if too slow, consider pre-rendering a set of common notes at app startup.
- **Synth is mutable:** `Synth::send_event()` and `read_next()` take `&mut self`. The `Rc<RefCell<Synth>>` pattern handles this, but be careful about borrowing during render loops.
- **AudioBufferSourceNode is one-shot:** A new source node must be created for each `play()` call. This is by Web Audio API design.
- **SF2 file size:** Keep the SoundFont small. A piano-only SF2 can be 2-5 MB. The architecture doc mentions 31 MB but that's excessive for web — find a smaller alternative.

### What Already Exists (DO NOT Rebuild)

| Component | File | Status |
|-----------|------|--------|
| `NotePlayer` trait + `PlaybackHandle` trait | `domain/src/ports.rs` | Complete |
| `AudioError` enum | `domain/src/ports.rs` | Complete with 5 variants |
| `OscillatorNotePlayer` | `web/src/adapters/audio_oscillator.rs` | Complete — reference implementation |
| `AudioContextManager` | `web/src/adapters/audio_context.rs` | Complete — shared between players |
| `SoundSourceID` type (default `"sf2:8:80"`) | `domain/src/types/sound_source.rs` | Complete |
| Sound source dropdown in Settings | `web/src/components/settings_view.rs:93-94` | Has single "Sine Oscillator" option |
| `peach.sound_source` localStorage key | `web/src/adapters/localstorage_settings.rs` | Read via `get_string()` |
| Async init pattern (IndexedDB) | `web/src/app.rs` | Template for SF2 loading |
| `SendWrapper` for non-Send context | `web/src/app.rs` | Pattern to follow for synth context |

### What Must NOT Change

- `domain/src/ports.rs` — the `NotePlayer` and `PlaybackHandle` traits must remain unchanged
- `domain/` crate — zero changes. No browser dependencies.
- `web/src/adapters/audio_oscillator.rs` — keep working as-is for fallback
- `web/src/adapters/audio_context.rs` — keep as-is, shared by both players
- Existing training behavior — oscillator-based training must continue working identically

### Project Structure Notes

**New files:**
- `web/src/adapters/audio_soundfont.rs` — SoundFontNotePlayer implementation
- `web/src/adapters/note_player.rs` — UnifiedNotePlayer/UnifiedPlaybackHandle enum wrapper
- `web/assets/soundfont/<name>.sf2` — SoundFont file asset

**Modified files:**
- `web/Cargo.toml` — add `oxisynth`, new `web-sys` features
- `web/src/adapters/mod.rs` — register new modules
- `web/src/app.rs` — async SoundFont loading + context provider
- `web/src/components/comparison_view.rs` — use UnifiedNotePlayer instead of OscillatorNotePlayer
- `web/src/components/pitch_matching_view.rs` — use UnifiedNotePlayer, update handle types
- `web/src/components/settings_view.rs` — add SoundFont options to sound source dropdown
- `index.html` or `Trunk.toml` — configure SF2 asset copying (if needed)

**No changes to:**
- Any `domain/` crate files
- `web/src/adapters/audio_oscillator.rs` (keep as-is)
- `web/src/adapters/audio_context.rs` (keep as-is)
- `web/src/bridge.rs` (no observer changes)
- `web/src/interval_codes.rs` (unrelated)

### References

- [Source: docs/planning-artifacts/epics.md#Story 5.2 — Acceptance criteria and BDD]
- [Source: docs/planning-artifacts/architecture.md#Audio Architecture — hybrid oscillator+OxiSynth approach]
- [Source: docs/planning-artifacts/architecture.md#Implementation Sequence — step 8, SoundFontNotePlayer]
- [Source: docs/planning-artifacts/architecture.md#Performance Requirements — NFR1 audio onset < 50ms]
- [Source: docs/planning-artifacts/ux-design-specification.md#Loading States — non-blocking SoundFont loading]
- [Source: docs/planning-artifacts/ux-design-specification.md#Error States — silent SoundFont failure]
- [Source: docs/planning-artifacts/ux-design-specification.md#Experience Principles — sound first, pixels second]
- [Source: docs/project-context.md#Technology Stack — OxiSynth, web-sys]
- [Source: docs/project-context.md#Web Audio Edge Cases — SoundFont loading non-blocking]
- [Source: domain/src/ports.rs — NotePlayer trait, PlaybackHandle trait, AudioError]
- [Source: domain/src/types/sound_source.rs — SoundSourceID default "sf2:8:80"]
- [Source: web/src/adapters/audio_oscillator.rs — OscillatorNotePlayer reference pattern]
- [Source: web/src/adapters/audio_context.rs — AudioContextManager shared singleton]
- [Source: web/src/app.rs — async init pattern with IndexedDB, context providers]
- [Source: web/src/components/comparison_view.rs:62 — note player creation point]
- [Source: web/src/components/pitch_matching_view.rs:59 — note player creation point]
- [Source: web/src/components/settings_view.rs:93-94 — sound source dropdown]
- [Source: docs/ios-reference/domain-blueprint.md#9.5 — SoundSourceProvider port]
- [Source: https://crates.io/crates/oxisynth — v0.1.0, pure Rust, WASM compatible]
- [Source: https://docs.rs/oxisynth — Synth, SoundFont, MidiEvent API]

## Previous Story Intelligence

### From Story 5.1 (Interval Training Mode)

- Story 5.1 was entirely web-layer work — zero domain changes. Story 5.2 is also web-layer only.
- `LocalStorageSettings::get_selected_intervals()` is a static method called with `::` not `.` — same applies to `get_string("peach.sound_source")`.
- Code review caught duplicated code between ComparisonView and PitchMatchingView — extracted shared `parse_intervals_param()`. Apply same DRY principle: the `create_note_player()` factory should be shared, not duplicated.
- Code review added screen reader announcements for accessibility — consider if sound source changes need any announcement.
- All 293+ domain tests pass, trunk build and clippy clean as of story 5.1 completion.

### From Story 4.4 (Pitch Matching Persistence)

- "Mirror with targeted changes" pattern: when making parallel changes to ComparisonView and PitchMatchingView, copy the pattern from one to the other with minimal targeted modifications.
- Two-borrow pattern in pitch_matching_view.rs: pre-check state then mutably borrow. Be careful with `Rc<RefCell<Synth>>` borrows during rendering — don't hold the borrow across the render loop.
- Code review caught redundant `.abs()` and ambiguous log messages — be precise.
- Never silently discard Results: `let _ = fallible()` is forbidden.

## Git Intelligence

Recent commits (newest first):
```
6438c72 Apply code review fixes for story 5.1 and mark as done
e595890 Implement story 5.1 Interval Training Mode
0ff658f Add story 5.1 Interval Training Mode and mark as ready-for-dev
6d29983 Apply code review fixes for story 4.4 and mark epic 4 as done
895d216 Implement story 4.4 Pitch Matching Persistence and fix pitch matching bugs
```

Convention: story creation commit -> implementation commit -> code review fixes commit. Commit messages use imperative mood, reference the story number.

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
