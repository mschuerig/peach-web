use domain::types::{Cents, DetunedMIDINote, Frequency, MIDINote};
use domain::TuningSystem;

/// Verify that equal temperament frequency conversions are accurate to within 0.1 cent
/// of mathematically correct values (NFR2 precision requirement).
#[test]
fn test_equal_temperament_precision_all_midi_notes() {
    let reference = Frequency::CONCERT_440;

    for midi in 0..=127u8 {
        let note = DetunedMIDINote::from(MIDINote::new(midi));
        let computed = TuningSystem::EqualTemperament.frequency(note, reference);

        // Exact mathematical formula
        let expected = 440.0 * 2.0_f64.powf((midi as f64 - 69.0) / 12.0);

        // Convert frequency difference to cents: 1200 * log2(computed/expected)
        let cent_error = 1200.0 * (computed.raw_value() / expected).log2();

        assert!(
            cent_error.abs() < 0.1,
            "MIDI {midi}: computed {}, expected {}, cent error {cent_error}",
            computed.raw_value(),
            expected
        );
    }
}

/// Verify A4 (MIDI 69) returns exactly 440.0 Hz.
#[test]
fn test_a4_exact_frequency() {
    let note = DetunedMIDINote::from(MIDINote::new(69));
    let freq = TuningSystem::EqualTemperament.frequency(note, Frequency::CONCERT_440);
    assert_eq!(freq.raw_value(), 440.0);
}

/// Verify detuned note frequency calculation precision.
#[test]
fn test_detuned_note_precision() {
    let reference = Frequency::CONCERT_440;

    // A4 + 50 cents should be halfway between A4 and A#4 on a log scale
    let detuned = DetunedMIDINote {
        note: MIDINote::new(69),
        offset: Cents::new(50.0),
    };
    let freq = TuningSystem::EqualTemperament.frequency(detuned, reference);

    // Expected: 440 * 2^(50/1200) ≈ 452.893
    let expected = 440.0 * 2.0_f64.powf(50.0 / 1200.0);
    let cent_error = 1200.0 * (freq.raw_value() / expected).log2();

    assert!(
        cent_error.abs() < 0.1,
        "Detuned A4+50c: computed {}, expected {expected}, cent error {cent_error}",
        freq.raw_value()
    );
}

/// Verify known frequency values for common notes.
#[test]
fn test_known_frequencies() {
    let reference = Frequency::CONCERT_440;
    let et = TuningSystem::EqualTemperament;

    // Middle C (C4, MIDI 60) ≈ 261.6256 Hz
    let c4 = et.frequency_for_note(MIDINote::new(60), reference);
    assert!((c4.raw_value() - 261.6256).abs() < 0.001);

    // A3 (MIDI 57) = 220.0 Hz
    let a3 = et.frequency_for_note(MIDINote::new(57), reference);
    assert!((a3.raw_value() - 220.0).abs() < 1e-10);

    // A5 (MIDI 81) = 880.0 Hz
    let a5 = et.frequency_for_note(MIDINote::new(81), reference);
    assert!((a5.raw_value() - 880.0).abs() < 1e-10);
}
