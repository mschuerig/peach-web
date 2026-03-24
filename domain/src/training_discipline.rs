use crate::records::{PitchDiscriminationRecord, PitchMatchingRecord};

/// The four independent training disciplines tracked by the app.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TrainingDiscipline {
    UnisonPitchDiscrimination,
    IntervalPitchDiscrimination,
    UnisonPitchMatching,
    IntervalPitchMatching,
}

/// Per-discipline configuration constants.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrainingDisciplineConfig {
    pub display_name: &'static str,
    pub unit_label: &'static str,
    pub optimal_baseline: f64,
    pub ewma_halflife_secs: f64,
    pub session_gap_secs: f64,
}

/// Whether a training discipline has accumulated any data yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrainingDisciplineState {
    NoData,
    Active,
}

const EWMA_HALFLIFE_SECS: f64 = 604_800.0; // 7 days
const SESSION_GAP_SECS: f64 = 1_800.0; // 30 minutes

static UNISON_PITCH_DISCRIMINATION_CONFIG: TrainingDisciplineConfig = TrainingDisciplineConfig {
    display_name: "training-discipline-hear-discriminate-single",
    unit_label: "cents",
    optimal_baseline: 8.0,
    ewma_halflife_secs: EWMA_HALFLIFE_SECS,
    session_gap_secs: SESSION_GAP_SECS,
};

static INTERVAL_PITCH_DISCRIMINATION_CONFIG: TrainingDisciplineConfig = TrainingDisciplineConfig {
    display_name: "training-discipline-hear-discriminate-intervals",
    unit_label: "cents",
    optimal_baseline: 12.0,
    ewma_halflife_secs: EWMA_HALFLIFE_SECS,
    session_gap_secs: SESSION_GAP_SECS,
};

static UNISON_PITCH_MATCHING_CONFIG: TrainingDisciplineConfig = TrainingDisciplineConfig {
    display_name: "training-discipline-tune-match-single",
    unit_label: "cents",
    optimal_baseline: 5.0,
    ewma_halflife_secs: EWMA_HALFLIFE_SECS,
    session_gap_secs: SESSION_GAP_SECS,
};

static INTERVAL_PITCH_MATCHING_CONFIG: TrainingDisciplineConfig = TrainingDisciplineConfig {
    display_name: "training-discipline-tune-match-intervals",
    unit_label: "cents",
    optimal_baseline: 8.0,
    ewma_halflife_secs: EWMA_HALFLIFE_SECS,
    session_gap_secs: SESSION_GAP_SECS,
};

impl TrainingDiscipline {
    /// All four training discipline variants.
    pub const ALL: [TrainingDiscipline; 4] = [
        TrainingDiscipline::UnisonPitchDiscrimination,
        TrainingDiscipline::IntervalPitchDiscrimination,
        TrainingDiscipline::UnisonPitchMatching,
        TrainingDiscipline::IntervalPitchMatching,
    ];

    /// Returns the static configuration for this training discipline.
    pub fn config(&self) -> &'static TrainingDisciplineConfig {
        match self {
            TrainingDiscipline::UnisonPitchDiscrimination => &UNISON_PITCH_DISCRIMINATION_CONFIG,
            TrainingDiscipline::IntervalPitchDiscrimination => {
                &INTERVAL_PITCH_DISCRIMINATION_CONFIG
            }
            TrainingDiscipline::UnisonPitchMatching => &UNISON_PITCH_MATCHING_CONFIG,
            TrainingDiscipline::IntervalPitchMatching => &INTERVAL_PITCH_MATCHING_CONFIG,
        }
    }

    /// Extracts the metric (absolute cent offset) from a discrimination record
    /// if this is a discrimination discipline and the record's interval matches.
    ///
    /// Returns `None` for matching disciplines (use `extract_matching_metric` instead).
    /// Unison disciplines match interval == 0; interval disciplines match interval != 0.
    pub fn extract_comparison_metric(&self, record: &PitchDiscriminationRecord) -> Option<f64> {
        match self {
            TrainingDiscipline::UnisonPitchDiscrimination
            | TrainingDiscipline::IntervalPitchDiscrimination => {
                if self.matches_interval(record.interval) {
                    Some(record.cent_offset.abs())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Extracts the metric (absolute user cent error) from a pitch matching record
    /// if this is a matching discipline and the record's interval matches.
    ///
    /// Returns `None` for discrimination disciplines (use `extract_comparison_metric` instead).
    /// Unison disciplines match interval == 0; interval disciplines match interval != 0.
    pub fn extract_matching_metric(&self, record: &PitchMatchingRecord) -> Option<f64> {
        match self {
            TrainingDiscipline::UnisonPitchMatching | TrainingDiscipline::IntervalPitchMatching => {
                if self.matches_interval(record.interval) {
                    Some(record.user_cent_error.abs())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn matches_interval(&self, interval: u8) -> bool {
        match self {
            TrainingDiscipline::UnisonPitchDiscrimination
            | TrainingDiscipline::UnisonPitchMatching => interval == 0,
            TrainingDiscipline::IntervalPitchDiscrimination
            | TrainingDiscipline::IntervalPitchMatching => interval != 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Config tests (AC: 1, 2, 3, 4, 8, 9) ---

    #[test]
    fn test_unison_discrimination_config() {
        let cfg = TrainingDiscipline::UnisonPitchDiscrimination.config();
        assert_eq!(
            cfg.display_name,
            "training-discipline-hear-discriminate-single"
        );
        assert_eq!(cfg.unit_label, "cents");
        assert_eq!(cfg.optimal_baseline, 8.0);
        assert_eq!(cfg.ewma_halflife_secs, 604_800.0);
        assert_eq!(cfg.session_gap_secs, 1_800.0);
    }

    #[test]
    fn test_interval_discrimination_config() {
        let cfg = TrainingDiscipline::IntervalPitchDiscrimination.config();
        assert_eq!(
            cfg.display_name,
            "training-discipline-hear-discriminate-intervals"
        );
        assert_eq!(cfg.unit_label, "cents");
        assert_eq!(cfg.optimal_baseline, 12.0);
        assert_eq!(cfg.ewma_halflife_secs, 604_800.0);
        assert_eq!(cfg.session_gap_secs, 1_800.0);
    }

    #[test]
    fn test_unison_matching_config() {
        let cfg = TrainingDiscipline::UnisonPitchMatching.config();
        assert_eq!(cfg.display_name, "training-discipline-tune-match-single");
        assert_eq!(cfg.unit_label, "cents");
        assert_eq!(cfg.optimal_baseline, 5.0);
        assert_eq!(cfg.ewma_halflife_secs, 604_800.0);
        assert_eq!(cfg.session_gap_secs, 1_800.0);
    }

    #[test]
    fn test_interval_matching_config() {
        let cfg = TrainingDiscipline::IntervalPitchMatching.config();
        assert_eq!(cfg.display_name, "training-discipline-tune-match-intervals");
        assert_eq!(cfg.unit_label, "cents");
        assert_eq!(cfg.optimal_baseline, 8.0);
        assert_eq!(cfg.ewma_halflife_secs, 604_800.0);
        assert_eq!(cfg.session_gap_secs, 1_800.0);
    }

    #[test]
    fn test_all_contains_four_variants() {
        assert_eq!(TrainingDiscipline::ALL.len(), 4);
        assert!(TrainingDiscipline::ALL.contains(&TrainingDiscipline::UnisonPitchDiscrimination));
        assert!(TrainingDiscipline::ALL.contains(&TrainingDiscipline::IntervalPitchDiscrimination));
        assert!(TrainingDiscipline::ALL.contains(&TrainingDiscipline::UnisonPitchMatching));
        assert!(TrainingDiscipline::ALL.contains(&TrainingDiscipline::IntervalPitchMatching));
    }

    // --- Metric extraction tests (AC: 5, 6) ---

    fn comparison_record(interval: u8, cent_offset: f64) -> PitchDiscriminationRecord {
        PitchDiscriminationRecord {
            reference_note: 60,
            target_note: 60 + interval,
            cent_offset,
            is_correct: true,
            interval,
            tuning_system: "equalTemperament".to_string(),
            timestamp: "2026-03-06T12:00:00Z".to_string(),
        }
    }

    fn matching_record(interval: u8, user_cent_error: f64) -> PitchMatchingRecord {
        PitchMatchingRecord {
            reference_note: 60,
            target_note: 60 + interval,
            initial_cent_offset: 20.0,
            user_cent_error,
            interval,
            tuning_system: "equalTemperament".to_string(),
            timestamp: "2026-03-06T12:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_extract_comparison_metric_unison_matches_zero_interval() {
        let record = comparison_record(0, 15.0);
        assert_eq!(
            TrainingDiscipline::UnisonPitchDiscrimination.extract_comparison_metric(&record),
            Some(15.0)
        );
        assert_eq!(
            TrainingDiscipline::IntervalPitchDiscrimination.extract_comparison_metric(&record),
            None
        );
    }

    #[test]
    fn test_extract_comparison_metric_interval_matches_nonzero() {
        let record = comparison_record(4, 10.0);
        assert_eq!(
            TrainingDiscipline::UnisonPitchDiscrimination.extract_comparison_metric(&record),
            None
        );
        assert_eq!(
            TrainingDiscipline::IntervalPitchDiscrimination.extract_comparison_metric(&record),
            Some(10.0)
        );
    }

    #[test]
    fn test_extract_comparison_metric_negative_offset_returns_positive() {
        let record = comparison_record(0, -25.0);
        assert_eq!(
            TrainingDiscipline::UnisonPitchDiscrimination.extract_comparison_metric(&record),
            Some(25.0)
        );
    }

    #[test]
    fn test_extract_matching_metric_unison_matches_zero_interval() {
        let record = matching_record(0, 3.5);
        assert_eq!(
            TrainingDiscipline::UnisonPitchMatching.extract_matching_metric(&record),
            Some(3.5)
        );
        assert_eq!(
            TrainingDiscipline::IntervalPitchMatching.extract_matching_metric(&record),
            None
        );
    }

    #[test]
    fn test_extract_matching_metric_interval_matches_nonzero() {
        let record = matching_record(7, 5.0);
        assert_eq!(
            TrainingDiscipline::UnisonPitchMatching.extract_matching_metric(&record),
            None
        );
        assert_eq!(
            TrainingDiscipline::IntervalPitchMatching.extract_matching_metric(&record),
            Some(5.0)
        );
    }

    #[test]
    fn test_extract_matching_metric_negative_error_returns_positive() {
        let record = matching_record(0, -8.0);
        assert_eq!(
            TrainingDiscipline::UnisonPitchMatching.extract_matching_metric(&record),
            Some(8.0)
        );
    }

    #[test]
    fn test_boundary_interval_zero_is_unison() {
        let comp = comparison_record(0, 10.0);
        assert!(
            TrainingDiscipline::UnisonPitchDiscrimination
                .extract_comparison_metric(&comp)
                .is_some()
        );
        assert!(
            TrainingDiscipline::IntervalPitchDiscrimination
                .extract_comparison_metric(&comp)
                .is_none()
        );

        let matching = matching_record(0, 5.0);
        assert!(
            TrainingDiscipline::UnisonPitchMatching
                .extract_matching_metric(&matching)
                .is_some()
        );
        assert!(
            TrainingDiscipline::IntervalPitchMatching
                .extract_matching_metric(&matching)
                .is_none()
        );
    }

    #[test]
    fn test_boundary_interval_one_is_interval() {
        let comp = comparison_record(1, 10.0);
        assert!(
            TrainingDiscipline::UnisonPitchDiscrimination
                .extract_comparison_metric(&comp)
                .is_none()
        );
        assert!(
            TrainingDiscipline::IntervalPitchDiscrimination
                .extract_comparison_metric(&comp)
                .is_some()
        );

        let matching = matching_record(1, 5.0);
        assert!(
            TrainingDiscipline::UnisonPitchMatching
                .extract_matching_metric(&matching)
                .is_none()
        );
        assert!(
            TrainingDiscipline::IntervalPitchMatching
                .extract_matching_metric(&matching)
                .is_some()
        );
    }

    #[test]
    fn test_matching_modes_return_none_for_comparison_records() {
        let comp = comparison_record(0, 10.0);
        assert!(
            TrainingDiscipline::UnisonPitchMatching
                .extract_comparison_metric(&comp)
                .is_none()
        );
        assert!(
            TrainingDiscipline::IntervalPitchMatching
                .extract_comparison_metric(&comp)
                .is_none()
        );

        let comp_interval = comparison_record(4, 10.0);
        assert!(
            TrainingDiscipline::UnisonPitchMatching
                .extract_comparison_metric(&comp_interval)
                .is_none()
        );
        assert!(
            TrainingDiscipline::IntervalPitchMatching
                .extract_comparison_metric(&comp_interval)
                .is_none()
        );
    }

    #[test]
    fn test_comparison_modes_return_none_for_matching_records() {
        let matching = matching_record(0, 5.0);
        assert!(
            TrainingDiscipline::UnisonPitchDiscrimination
                .extract_matching_metric(&matching)
                .is_none()
        );
        assert!(
            TrainingDiscipline::IntervalPitchDiscrimination
                .extract_matching_metric(&matching)
                .is_none()
        );

        let matching_interval = matching_record(7, 5.0);
        assert!(
            TrainingDiscipline::UnisonPitchDiscrimination
                .extract_matching_metric(&matching_interval)
                .is_none()
        );
        assert!(
            TrainingDiscipline::IntervalPitchDiscrimination
                .extract_matching_metric(&matching_interval)
                .is_none()
        );
    }
}
