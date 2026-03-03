use domain::*;

/// Replay a sequence of comparison results and verify profile state.
#[test]
fn test_profile_hydration_from_comparison_sequence() {
    let mut profile = PerceptualProfile::new();

    // Simulate a sequence of comparisons for note 60 (C4)
    let offsets = [50.0, 40.0, 45.0, 35.0, 55.0];
    for &offset in &offsets {
        profile.update(MIDINote::new(60), offset, true);
    }

    let stats = profile.note_stats(MIDINote::new(60));
    assert_eq!(stats.sample_count(), 5);

    // Manually compute expected mean: (50+40+45+35+55)/5 = 225/5 = 45.0
    assert!((stats.mean() - 45.0).abs() < 1e-10);

    // Manually compute expected sample std dev:
    // deviations: 5, -5, 0, -10, 10
    // sum of squares: 25 + 25 + 0 + 100 + 100 = 250
    // variance: 250/4 = 62.5
    // std_dev: sqrt(62.5) ≈ 7.9057
    assert!((stats.std_dev() - (62.5_f64).sqrt()).abs() < 1e-10);
}

/// Verify that replay order matters — Welford's is order-independent for final result
/// but we verify consistency.
#[test]
fn test_profile_hydration_order_independence() {
    let offsets = [10.0, 20.0, 30.0, 40.0, 50.0];

    let mut profile_forward = PerceptualProfile::new();
    for &offset in &offsets {
        profile_forward.update(MIDINote::new(60), offset, true);
    }

    let mut profile_reverse = PerceptualProfile::new();
    for &offset in offsets.iter().rev() {
        profile_reverse.update(MIDINote::new(60), offset, true);
    }

    let fwd = profile_forward.note_stats(MIDINote::new(60));
    let rev = profile_reverse.note_stats(MIDINote::new(60));

    // Mean should be identical regardless of order
    assert!((fwd.mean() - rev.mean()).abs() < 1e-10);
    assert_eq!(fwd.sample_count(), rev.sample_count());
    // Std dev should be the same (Welford's is numerically stable in both orders)
    assert!((fwd.std_dev() - rev.std_dev()).abs() < 1e-10);
}

/// Replay records for multiple notes and verify overall statistics.
#[test]
fn test_profile_hydration_multi_note() {
    let mut profile = PerceptualProfile::new();

    // Note 60: mean = 30
    profile.update(MIDINote::new(60), 20.0, true);
    profile.update(MIDINote::new(60), 40.0, false);

    // Note 72: mean = 60
    profile.update(MIDINote::new(72), 50.0, true);
    profile.update(MIDINote::new(72), 70.0, false);

    // Note 48: mean = 45
    profile.update(MIDINote::new(48), 45.0, true);

    // Overall mean: (30 + 60 + 45) / 3 = 45.0
    assert!((profile.overall_mean().unwrap() - 45.0).abs() < 1e-10);

    // Overall std dev: sample std dev of [30, 60, 45]
    // mean = 45, deviations: -15, 15, 0
    // variance = (225 + 225 + 0) / 2 = 225
    // std dev = 15.0
    assert!((profile.overall_std_dev().unwrap() - 15.0).abs() < 1e-10);
}

/// Verify weak spots after hydration correctly identifies problem areas.
#[test]
fn test_profile_hydration_weak_spots() {
    let mut profile = PerceptualProfile::new();

    // Train some notes with varying difficulty
    profile.update(MIDINote::new(60), 10.0, true); // Easy (low mean)
    profile.update(MIDINote::new(65), 80.0, false); // Hard (high mean)
    profile.update(MIDINote::new(72), 50.0, true); // Medium

    let weak = profile.weak_spots(5);
    assert_eq!(weak.len(), 5);

    // First entries should be untrained notes (125 untrained out of 128)
    assert!(!profile.note_stats(weak[0]).is_trained());
    assert!(!profile.note_stats(weak[1]).is_trained());
}

/// Verify pitch matching accumulator hydration.
#[test]
fn test_profile_hydration_matching_accumulators() {
    let mut profile = PerceptualProfile::new();

    // Simulate pitch matching results
    let errors = [3.0, -5.0, 7.0, -2.0, 4.0];
    for &error in &errors {
        profile.update_matching(MIDINote::new(60), error);
    }

    // Abs values: 3, 5, 7, 2, 4
    // Mean abs: (3+5+7+2+4)/5 = 21/5 = 4.2
    assert!((profile.matching_mean().unwrap() - 4.2).abs() < 1e-10);

    // Sample std dev of abs values:
    // deviations from 4.2: -1.2, 0.8, 2.8, -2.2, -0.2
    // sum sq: 1.44 + 0.64 + 7.84 + 4.84 + 0.04 = 14.8
    // variance: 14.8/4 = 3.7
    // std dev: sqrt(3.7) ≈ 1.9235
    assert!((profile.matching_std_dev().unwrap() - (3.7_f64).sqrt()).abs() < 1e-10);
}

/// Verify reset + re-hydration produces same results.
#[test]
fn test_profile_reset_and_rehydrate() {
    let mut profile = PerceptualProfile::new();

    // First hydration
    profile.update(MIDINote::new(60), 50.0, true);
    profile.update(MIDINote::new(60), 30.0, false);

    let mean_before = profile.note_stats(MIDINote::new(60)).mean();

    // Reset and re-hydrate with same data
    profile.reset();
    profile.update(MIDINote::new(60), 50.0, true);
    profile.update(MIDINote::new(60), 30.0, false);

    let mean_after = profile.note_stats(MIDINote::new(60)).mean();
    assert!((mean_before - mean_after).abs() < 1e-10);
}

/// Stress test: hydrate many records and verify no numerical drift.
#[test]
fn test_profile_hydration_large_dataset() {
    let mut profile = PerceptualProfile::new();

    // 1000 records for a single note
    for i in 0..1000 {
        let offset = 50.0 + (i % 20) as f64; // 50..69 repeating
        profile.update(MIDINote::new(60), offset, i % 2 == 0);
    }

    let stats = profile.note_stats(MIDINote::new(60));
    assert_eq!(stats.sample_count(), 1000);

    // Expected mean: average of 50..69 repeating = (50+51+...+69)/20 = 59.5
    assert!((stats.mean() - 59.5).abs() < 0.01);
    assert!(stats.std_dev() > 0.0);
}
