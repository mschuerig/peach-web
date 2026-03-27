---
title: 'Rhythm Training: SoundFont Percussion Click'
type: 'feature'
created: '2026-03-27'
status: 'done'
baseline_commit: 'c82b76f'
context: []
---

# Rhythm Training: SoundFont Percussion Click

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Rhythm training uses a synthesized 5ms noise burst for clicks, which sounds artificial. A SoundFont percussion sample (high woodblock) would be more musically natural and consistent with the pitch training's use of SoundFont samples.

**Approach:** Replace the `AudioBuffer` noise-burst click with OxiSynth percussion playback via the existing WorkletBridge. Use GM percussion bank 128, first available program, MIDI note 76. Add channel support to the synth worklet so percussion (channel 9) doesn't interfere with pitch training (channel 0).

## Boundaries & Constraints

**Always:** Use MIDI channel 9 for percussion (GM standard). Keep pitch training on channel 0 unaffected. Select percussion program once when the scheduler is created, not on every click.

**Ask First:** Changes to the synth worklet WASM API (adding channel parameter vs. dedicated percussion functions).

**Never:** Change the scheduler's timing architecture (lookahead interval pattern). Break pitch training sound playback. Add user-facing settings for rhythm sound selection (future scope).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Normal click | Scheduler fires step at scheduled time | Woodblock sample plays via worklet | Log warning, continue (silent click) |
| Accent click (beat 1) | First step in pattern | Higher velocity noteOn (vel=127 vs vel=80) | Same as normal |
| Tap feedback click | User taps during continuous matching | Immediate woodblock via noteOn | Log warning, continue |
| Worklet not ready | Rhythm training starts before SF2 loaded | Graceful fallback — log error, no crash | Log error, set audio error signal |
| Rapid tempo (200+ BPM) | 16th notes at ~75ms spacing | Each click distinct, no overlap issues | noteOff sent before next noteOn |

</frozen-after-approval>

## Code Map

- `synth-worklet/src/lib.rs` -- WASM FFI: add channel parameter to note_on/note_off/select_program
- `web/assets/soundfont/synth-processor.js` -- JS worklet: remove percussion bank filter, pass channel in messages
- `web/src/adapters/audio_soundfont.rs` -- WorkletBridge: add channel-aware send methods for percussion
- `web/src/adapters/rhythm_scheduler.rs` -- Replace AudioBuffer click with WorkletBridge noteOn/noteOff
- `web/src/components/rhythm_offset_detection_view.rs` -- Adapt to new scheduler API (WorkletBridge instead of click_buffer)
- `web/src/components/continuous_rhythm_matching_view.rs` -- Same adaptation + tap feedback via WorkletBridge

## Tasks & Acceptance

**Execution:**
- [ ] `synth-worklet/src/lib.rs` -- Add channel parameter to `synth_note_on`, `synth_note_off`, `synth_select_program`; fix bank-before-program-change order in select_program
- [ ] `web/assets/soundfont/synth-processor.js` -- Remove `bank < 120` filter from `parseSF2Presets`; pass `msg.channel ?? 0` to WASM functions
- [ ] `web/src/adapters/audio_soundfont.rs` -- Add `send_note_on_ch`, `send_note_off_ch`, `send_select_program_ch` to WorkletBridge with channel field
- [ ] `web/src/adapters/rhythm_scheduler.rs` -- Replace `AudioBuffer`/`AudioBufferSourceNode` with `WorkletBridge` percussion noteOn/noteOff; use `setTimeout` for sub-interval timing; select bank 128 program on channel 9 at construction
- [ ] `web/src/components/continuous_rhythm_matching_view.rs` -- Pass WorkletBridge to scheduler; use WorkletBridge for tap feedback click
- [ ] `web/src/components/rhythm_offset_detection_view.rs` -- Pass WorkletBridge to scheduler and offset click

**Acceptance Criteria:**
- Given rhythm training starts, when the metronome plays, then a woodblock percussion sound is heard (not a noise burst)
- Given pitch training is active, when switching to rhythm training, then pitch training sounds remain on channel 0 unaffected
- Given the synth worklet is loaded, when `selectProgram` is sent with bank 128, then percussion presets are found and selected
- Given a tap during continuous matching, when the tap handler fires, then an immediate woodblock click is heard

## Design Notes

**Timing approach:** The scheduler's lookahead loop currently pre-schedules `AudioBufferSourceNode.start(when)` for sample-accurate timing. With WorkletBridge noteOn, messages play immediately on receipt. To preserve timing within the lookahead window, each scheduled click uses `gloo_timers::callback::Timeout` with delay = `(next_step_time - current_time) * 1000` ms. This gives JS-timer precision (~1-4ms) which is adequate for rhythm training.

**noteOff strategy:** Send noteOff 50ms after each noteOn via a forgotten Timeout. Percussion samples are short, but explicit noteOff prevents voice accumulation in OxiSynth.

**Channel architecture:** Channel 0 = pitch training (existing). Channel 9 = percussion/rhythm (new, GM standard). No other channels used.

**Bank/program select order fix:** Current `synth_select_program` sends ProgramChange before `select_bank` — this is wrong. Fix: call `select_bank` first, then `ProgramChange`.

## Verification

**Commands:**
- `cargo clippy --workspace` -- expected: no warnings
- `cargo test -p domain` -- expected: all tests pass (domain unaffected)
- `cargo fmt --check --all` -- expected: no formatting issues
- `trunk build` -- expected: WASM build succeeds

**Manual checks:**
- Start rhythm offset detection training — hear woodblock clicks instead of noise burst
- Start continuous rhythm matching — hear woodblock clicks for metronome and tap feedback
- Switch between pitch and rhythm training — both sound correct
- Verify accent on beat 1 is audibly louder
