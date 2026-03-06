use domain::*;

/// Test that Kazez adaptive algorithm converges difficulty downward
/// over a sequence of correct answers.
#[test]
fn test_kazez_convergence_all_correct() {
    let mut profile = PerceptualProfile::new();
    let settings = TrainingSettings::default();
    let interval = DirectedInterval::new(Interval::Prime, Direction::Up);

    let mut last_completed: Option<CompletedPitchComparison> = None;
    let mut magnitudes = Vec::new();

    for _ in 0..20 {
        let comp = next_pitch_comparison(
            &profile,
            &settings,
            last_completed.as_ref(),
            interval,
        );

        let magnitude = comp.target_note().offset.magnitude();
        magnitudes.push(magnitude);

        // Always answer correctly
        let completed = CompletedPitchComparison::new(
            comp,
            comp.is_target_higher(),
            TuningSystem::EqualTemperament,
            "2026-03-03T14:00:00Z".to_string(),
        );

        // Update profile with the comparison result
        profile.update(
            comp.reference_note(),
            magnitude,
            completed.is_correct(),
        );

        last_completed = Some(completed);
    }

    // Difficulty should be strictly decreasing (all correct → kazez_narrow)
    for i in 1..magnitudes.len() {
        assert!(
            magnitudes[i] < magnitudes[i - 1],
            "Magnitude at step {} ({}) should be less than step {} ({})",
            i,
            magnitudes[i],
            i - 1,
            magnitudes[i - 1]
        );
    }

    // Final magnitude should be much less than starting (100 cents)
    assert!(
        magnitudes.last().unwrap() < &20.0,
        "After 20 correct answers, magnitude should be well below 20 cents, got {}",
        magnitudes.last().unwrap()
    );
}

/// Test that Kazez adaptive algorithm increases difficulty
/// after a sequence of incorrect answers, up to the max.
#[test]
fn test_kazez_divergence_all_incorrect() {
    let profile = PerceptualProfile::new();
    let settings = TrainingSettings::default();
    let interval = DirectedInterval::new(Interval::Prime, Direction::Up);

    let mut last_completed: Option<CompletedPitchComparison> = None;
    let mut magnitudes = Vec::new();

    for _ in 0..10 {
        let comp = next_pitch_comparison(
            &profile,
            &settings,
            last_completed.as_ref(),
            interval,
        );

        let magnitude = comp.target_note().offset.magnitude();
        magnitudes.push(magnitude);

        // Always answer incorrectly
        let completed = CompletedPitchComparison::new(
            comp,
            !comp.is_target_higher(),
            TuningSystem::EqualTemperament,
            "2026-03-03T14:00:00Z".to_string(),
        );

        last_completed = Some(completed);
    }

    // All magnitudes should be clamped at 100.0 (max) since starting at 100
    // and kazez_widen(100) > 100 → clamped
    for mag in &magnitudes {
        assert!(
            (*mag - 100.0).abs() < 1e-10,
            "Magnitude should be clamped at 100.0, got {mag}"
        );
    }
}

/// Test that alternating correct/incorrect answers produce oscillating difficulty.
#[test]
fn test_kazez_oscillation() {
    let profile = PerceptualProfile::new();
    let settings = TrainingSettings::default();
    let interval = DirectedInterval::new(Interval::Prime, Direction::Up);

    let mut last_completed: Option<CompletedPitchComparison> = None;

    for i in 0..10 {
        let comp = next_pitch_comparison(
            &profile,
            &settings,
            last_completed.as_ref(),
            interval,
        );

        // Alternate correct/incorrect
        let is_correct_answer = i % 2 == 0;
        let user_answered = if is_correct_answer {
            comp.is_target_higher()
        } else {
            !comp.is_target_higher()
        };

        let completed = CompletedPitchComparison::new(
            comp,
            user_answered,
            TuningSystem::EqualTemperament,
            "2026-03-03T14:00:00Z".to_string(),
        );

        last_completed = Some(completed);
    }

    // After alternating, we should still have a valid comparison
    let final_comp = next_pitch_comparison(
        &profile,
        &settings,
        last_completed.as_ref(),
        interval,
    );
    let mag = final_comp.target_note().offset.magnitude();
    assert!(mag >= settings.min_cent_difference().raw_value);
    assert!(mag <= settings.max_cent_difference().raw_value);
}

/// Test convergence with interval transposition (non-prime interval).
#[test]
fn test_convergence_with_perfect_fifth_interval() {
    let profile = PerceptualProfile::new();
    let settings = TrainingSettings::default();
    let interval = DirectedInterval::new(Interval::PerfectFifth, Direction::Up);

    for _ in 0..20 {
        let comp = next_pitch_comparison(&profile, &settings, None, interval);

        // Verify interval transposition is correct
        let ref_val = comp.reference_note().raw_value() as i16;
        let target_val = comp.target_note().note.raw_value() as i16;
        assert_eq!(
            target_val - ref_val,
            7, // Perfect fifth = 7 semitones up
            "Target should be 7 semitones above reference"
        );

        // Verify reference is within allowed range for upward P5
        assert!(comp.reference_note().raw_value() >= 36);
        assert!(comp.reference_note().raw_value() <= 84 - 7);
    }
}
