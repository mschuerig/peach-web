# Future Work

Deferred findings from code reviews and implementation that need attention in future stories.

## Fill the Gap: tap latency and audio doubling

**Source:** Story 18.2 E2E testing
**Date:** 2026-03-25

The "Fill the Gap" training screen has noticeable tap latency issues:

- **Desktop (Mac):** Unusable with trackpad or keyboard. Partly caused by Bluetooth audio latency, but the overall round-trip (pointerdown → audioContext.currentTime read → click playback) adds further delay.
- **Mobile (iPhone):** Sort-of usable. Latency is better but still noticeable. At slow tempos, tapping every 16th note produces a jittery doubling of tones — the user's tap click overlaps with the scheduler's next beat.

**Root causes to investigate:**
1. Main-thread event processing delay between pointerdown and audioContext.currentTime read
2. Click buffer creation on every tap (currently `create_click_buffer` is called per-tap in the handler; should reuse the shared buffer exclusively)
3. Possible overlap between user tap click and scheduler click when tap timing is close to a non-gap beat
4. Bluetooth audio output adds 40-200ms latency that cannot be compensated without explicit latency measurement
5. Whether an AudioWorklet-based approach (moving tap detection closer to the audio thread) could reduce perceived latency

**Impact:** The training mode is functional but the experience is degraded, especially on desktop. Rhythm training inherently requires low-latency audio feedback.

**Potential improvements:**
- Pre-compute and reuse click buffer (avoid per-tap allocation)
- Investigate Web Audio `outputLatency` / `baseLatency` properties for compensation
- Consider visual-only feedback mode (no tap click) as a fallback
- Research AudioWorklet-based tap timestamping
