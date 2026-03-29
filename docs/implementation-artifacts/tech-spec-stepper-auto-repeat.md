---
title: 'Stepper Auto-Repeat on Press-and-Hold'
type: 'feature'
created: '2026-03-29'
status: 'done'
baseline_commit: 'ab83444'
context: []
---

# Stepper Auto-Repeat on Press-and-Hold

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** The settings page steppers require repeated tapping to change values across a range (e.g., note range 21–108, tempo). This is tedious for large adjustments.

**Approach:** Add press-and-hold auto-repeat to the `Stepper` component: fire once on press, then after an initial delay begin repeating at an accelerating rate, stopping on release.

## Boundaries & Constraints

**Always:** Keep the existing single-click behavior unchanged. Auto-repeat must respect the `disabled` signal — stop repeating if the button becomes disabled mid-hold. Must work on both pointer (mouse) and touch devices.

**Ask First:** Acceleration curve parameters (initial delay, repeat interval, minimum interval) if defaults feel wrong during testing.

**Never:** Do not change the Stepper's public prop API — the auto-repeat is internal behavior. Do not use `Interval` from `gloo_timers::callback` with `.forget()` (lifecycle hazard per project conventions).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Single tap | pointerdown + quick pointerup (<300ms) | Fires callback once (on pointerdown) | N/A |
| Hold | pointerdown held for 1s+ | Fires once immediately, then repeats after 400ms delay, accelerating to 50ms min interval | N/A |
| Hold until limit | Hold increment, value hits max | Repeating stops when `disabled` becomes true | N/A |
| Pointer leaves | pointerdown then pointer leaves button | Repeating stops on pointerleave/pointercancel | N/A |
| Touch scroll | Touch down then scroll gesture | Repeating stops on pointercancel | N/A |

</frozen-after-approval>

## Code Map

- `web/src/components/settings_view.rs:80-112` -- `Stepper` component to modify
- `web/src/components/pitch_slider.rs` -- Reference for pointer event patterns and pointer capture

## Tasks & Acceptance

**Execution:**
- [ ] `web/src/components/settings_view.rs` -- Replace `on:click` with `on:pointerdown`/`on:pointerup`/`on:pointerleave`/`on:pointercancel` handlers in `Stepper`. Use `set_interval_with_handle` + `clear_interval` from `web_sys` (or `gloo_timers::callback::Interval` stored in a local `RwSignal<Option<Interval>>`) to drive repeating. Fire callback once immediately on pointerdown, start repeat timer after 400ms initial delay, decrease interval stepwise (400→200→100→50ms) as holding continues. Clear timer on pointerup/pointerleave/pointercancel or when disabled becomes true.

**Acceptance Criteria:**
- Given a stepper button, when tapped quickly, then the callback fires exactly once
- Given a stepper button, when held for 1+ seconds, then the value auto-increments/decrements repeatedly with accelerating speed
- Given a stepper at its limit (disabled), when held, then repeating stops immediately
- Given a stepper being held, when pointer leaves the button area, then repeating stops

## Design Notes

Use a recursive `spawn_local` + `TimeoutFuture` pattern (consistent with project conventions) rather than `Interval`. Each iteration checks a `holding: RwSignal<bool>` flag and the disabled signal before scheduling the next tick. This avoids the `.forget()` lifecycle hazard and gives natural cleanup. The delay decreases each tick: 400ms → 200ms → 100ms → 50ms (clamped).

## Verification

**Commands:**
- `cargo clippy --workspace` -- expected: no warnings
- `trunk build` -- expected: successful WASM build

**Manual checks:**
- In browser, tap stepper: value changes by 1 step
- Hold stepper: value auto-repeats with acceleration
- Hold until limit: repeating stops at boundary
- Move pointer off button while holding: repeating stops
