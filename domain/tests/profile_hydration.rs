use domain::*;

/// Replay a sequence of discrimination results and verify profile state.
#[test]
fn test_profile_hydration_from_discrimination_sequence() {
    let mut profile = PerceptualProfile::new();

    // Simulate a sequence of discrimination trials via add_point
    let offsets = [50.0, 40.0, 45.0, 35.0, 55.0];
    for (i, &offset) in offsets.iter().enumerate() {
        profile.add_point(
            TrainingDiscipline::UnisonPitchDiscrimination,
            MetricPoint::new(i as f64 * 1000.0, offset),
            true,
        );
    }

    assert_eq!(
        profile.record_count(TrainingDiscipline::UnisonPitchDiscrimination),
        5
    );

    // Manually compute expected mean: (50+40+45+35+55)/5 = 225/5 = 45.0
    let mean = profile.discrimination_mean(0).unwrap();
    assert!((mean.raw_value - 45.0).abs() < 1e-10);
}

/// Verify that replay order matters — Welford's is order-independent for final result.
#[test]
fn test_profile_hydration_order_independence() {
    let offsets = [10.0, 20.0, 30.0, 40.0, 50.0];

    let mut profile_forward = PerceptualProfile::new();
    for (i, &offset) in offsets.iter().enumerate() {
        profile_forward.add_point(
            TrainingDiscipline::UnisonPitchDiscrimination,
            MetricPoint::new(i as f64 * 1000.0, offset),
            true,
        );
    }

    let mut profile_reverse = PerceptualProfile::new();
    for (i, &offset) in offsets.iter().rev().enumerate() {
        profile_reverse.add_point(
            TrainingDiscipline::UnisonPitchDiscrimination,
            MetricPoint::new(i as f64 * 1000.0, offset),
            true,
        );
    }

    let fwd_mean = profile_forward.discrimination_mean(0).unwrap();
    let rev_mean = profile_reverse.discrimination_mean(0).unwrap();

    // Mean should be identical regardless of order
    assert!((fwd_mean.raw_value - rev_mean.raw_value).abs() < 1e-10);
    assert_eq!(
        profile_forward.record_count(TrainingDiscipline::UnisonPitchDiscrimination),
        profile_reverse.record_count(TrainingDiscipline::UnisonPitchDiscrimination),
    );
}

/// Verify that incorrect answers are filtered out of profile.
#[test]
fn test_profile_filters_incorrect_answers() {
    let mut profile = PerceptualProfile::new();

    profile.add_point(
        TrainingDiscipline::UnisonPitchDiscrimination,
        MetricPoint::new(1000.0, 20.0),
        true,
    );
    profile.add_point(
        TrainingDiscipline::UnisonPitchDiscrimination,
        MetricPoint::new(2000.0, 999.0),
        false, // incorrect — should be filtered
    );

    assert_eq!(
        profile.record_count(TrainingDiscipline::UnisonPitchDiscrimination),
        1
    );
    let mean = profile.discrimination_mean(0).unwrap();
    assert!((mean.raw_value - 20.0).abs() < 1e-10);
}

/// Verify pitch matching accumulator hydration.
#[test]
fn test_profile_hydration_matching_accumulators() {
    let mut profile = PerceptualProfile::new();

    // Simulate pitch matching results (absolute errors)
    let errors = [3.0, 5.0, 7.0, 2.0, 4.0];
    for (i, &error) in errors.iter().enumerate() {
        profile.add_point(
            TrainingDiscipline::UnisonPitchMatching,
            MetricPoint::new(i as f64 * 1000.0, error),
            true,
        );
    }

    // Mean: (3+5+7+2+4)/5 = 21/5 = 4.2
    let mean = profile.matching_mean().unwrap();
    assert!((mean.raw_value - 4.2).abs() < 1e-10);
}

/// Verify reset + re-hydration produces same results.
#[test]
fn test_profile_reset_and_rehydrate() {
    let mut profile = PerceptualProfile::new();

    // First hydration
    profile.add_point(
        TrainingDiscipline::UnisonPitchDiscrimination,
        MetricPoint::new(1000.0, 50.0),
        true,
    );
    profile.add_point(
        TrainingDiscipline::UnisonPitchDiscrimination,
        MetricPoint::new(2000.0, 30.0),
        true,
    );

    let mean_before = profile.discrimination_mean(0).unwrap();

    // Reset and re-hydrate with same data
    profile.reset_all();
    profile.add_point(
        TrainingDiscipline::UnisonPitchDiscrimination,
        MetricPoint::new(1000.0, 50.0),
        true,
    );
    profile.add_point(
        TrainingDiscipline::UnisonPitchDiscrimination,
        MetricPoint::new(2000.0, 30.0),
        true,
    );

    let mean_after = profile.discrimination_mean(0).unwrap();
    assert!((mean_before.raw_value - mean_after.raw_value).abs() < 1e-10);
}

/// Stress test: hydrate many records and verify no numerical drift.
#[test]
fn test_profile_hydration_large_dataset() {
    let mut profile = PerceptualProfile::new();

    // 1000 records
    for i in 0..1000 {
        let offset = 50.0 + (i % 20) as f64; // 50..69 repeating
        profile.add_point(
            TrainingDiscipline::UnisonPitchDiscrimination,
            MetricPoint::new(i as f64 * 100.0, offset),
            true,
        );
    }

    assert_eq!(
        profile.record_count(TrainingDiscipline::UnisonPitchDiscrimination),
        1000
    );

    // Expected mean: average of 50..69 repeating = (50+51+...+69)/20 = 59.5
    let mean = profile.discrimination_mean(0).unwrap();
    assert!((mean.raw_value - 59.5).abs() < 0.01);
}

/// Verify rebuild_all produces consistent results.
#[test]
fn test_profile_rebuild_all() {
    use std::collections::HashMap;

    let mut profile = PerceptualProfile::new();
    let mut points = HashMap::new();
    points.insert(
        TrainingDiscipline::UnisonPitchDiscrimination,
        vec![
            MetricPoint::new(1000.0, 20.0),
            MetricPoint::new(2000.0, 40.0),
            MetricPoint::new(3000.0, 30.0),
        ],
    );
    points.insert(
        TrainingDiscipline::UnisonPitchMatching,
        vec![
            MetricPoint::new(1000.0, 5.0),
            MetricPoint::new(2000.0, 10.0),
        ],
    );

    profile.rebuild_all(points);

    assert_eq!(
        profile.record_count(TrainingDiscipline::UnisonPitchDiscrimination),
        3
    );
    assert_eq!(
        profile.record_count(TrainingDiscipline::UnisonPitchMatching),
        2
    );
    assert_eq!(
        profile.record_count(TrainingDiscipline::IntervalPitchDiscrimination),
        0
    );

    let disc_mean = profile.discrimination_mean(0).unwrap();
    assert!((disc_mean.raw_value - 30.0).abs() < 1e-10);

    let match_mean = profile.matching_mean().unwrap();
    assert!((match_mean.raw_value - 7.5).abs() < 1e-10);
}
