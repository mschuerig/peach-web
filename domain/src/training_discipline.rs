use crate::records::{
    ContinuousRhythmMatchingRecord, PitchDiscriminationRecord, PitchMatchingRecord,
    RhythmOffsetDetectionRecord, TrainingRecord,
};
use crate::statistics_key::StatisticsKey;
use crate::types::{RhythmDirection, RhythmOffset, TempoBPM, TempoRange};

/// The six independent training disciplines tracked by the app.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TrainingDiscipline {
    UnisonPitchDiscrimination,
    IntervalPitchDiscrimination,
    UnisonPitchMatching,
    IntervalPitchMatching,
    RhythmOffsetDetection,
    ContinuousRhythmMatching,
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
    unit_label: "unit-cents",
    optimal_baseline: 8.0,
    ewma_halflife_secs: EWMA_HALFLIFE_SECS,
    session_gap_secs: SESSION_GAP_SECS,
};

static INTERVAL_PITCH_DISCRIMINATION_CONFIG: TrainingDisciplineConfig = TrainingDisciplineConfig {
    display_name: "training-discipline-hear-discriminate-intervals",
    unit_label: "unit-cents",
    optimal_baseline: 12.0,
    ewma_halflife_secs: EWMA_HALFLIFE_SECS,
    session_gap_secs: SESSION_GAP_SECS,
};

static UNISON_PITCH_MATCHING_CONFIG: TrainingDisciplineConfig = TrainingDisciplineConfig {
    display_name: "training-discipline-tune-match-single",
    unit_label: "unit-cents",
    optimal_baseline: 5.0,
    ewma_halflife_secs: EWMA_HALFLIFE_SECS,
    session_gap_secs: SESSION_GAP_SECS,
};

static INTERVAL_PITCH_MATCHING_CONFIG: TrainingDisciplineConfig = TrainingDisciplineConfig {
    display_name: "training-discipline-tune-match-intervals",
    unit_label: "unit-cents",
    optimal_baseline: 8.0,
    ewma_halflife_secs: EWMA_HALFLIFE_SECS,
    session_gap_secs: SESSION_GAP_SECS,
};

static RHYTHM_OFFSET_DETECTION_CONFIG: TrainingDisciplineConfig = TrainingDisciplineConfig {
    display_name: "training-mode-compare-timing",
    unit_label: "unit-percent-16th",
    optimal_baseline: 5.0,
    ewma_halflife_secs: EWMA_HALFLIFE_SECS,
    session_gap_secs: SESSION_GAP_SECS,
};

static CONTINUOUS_RHYTHM_MATCHING_CONFIG: TrainingDisciplineConfig = TrainingDisciplineConfig {
    display_name: "training-mode-fill-the-gap",
    unit_label: "unit-percent-16th",
    optimal_baseline: 5.0,
    ewma_halflife_secs: EWMA_HALFLIFE_SECS,
    session_gap_secs: SESSION_GAP_SECS,
};

impl TrainingDiscipline {
    /// All six training discipline variants.
    pub const ALL: [TrainingDiscipline; 6] = [
        TrainingDiscipline::UnisonPitchDiscrimination,
        TrainingDiscipline::IntervalPitchDiscrimination,
        TrainingDiscipline::UnisonPitchMatching,
        TrainingDiscipline::IntervalPitchMatching,
        TrainingDiscipline::RhythmOffsetDetection,
        TrainingDiscipline::ContinuousRhythmMatching,
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
            TrainingDiscipline::RhythmOffsetDetection => &RHYTHM_OFFSET_DETECTION_CONFIG,
            TrainingDiscipline::ContinuousRhythmMatching => &CONTINUOUS_RHYTHM_MATCHING_CONFIG,
        }
    }

    /// Extracts the metric (absolute cent offset) from a discrimination record
    /// if this is a discrimination discipline and the record's interval matches.
    ///
    /// Returns `None` for matching disciplines (use `extract_matching_metric` instead).
    /// Unison disciplines match interval == 0; interval disciplines match interval != 0.
    pub fn extract_discrimination_metric(&self, record: &PitchDiscriminationRecord) -> Option<f64> {
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
    /// Returns `None` for discrimination disciplines (use `extract_discrimination_metric` instead).
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

    /// Extracts the metric (percentage of sixteenth note) from a rhythm offset detection record
    /// if this is the `RhythmOffsetDetection` discipline.
    ///
    /// Returns `None` for all other disciplines.
    pub fn extract_rhythm_offset_metric(
        &self,
        record: &RhythmOffsetDetectionRecord,
    ) -> Option<f64> {
        match self {
            TrainingDiscipline::RhythmOffsetDetection => {
                let tempo = TempoBPM::try_new(record.tempo_bpm).ok()?;
                if !record.offset_ms.is_finite() {
                    return None;
                }
                let offset = RhythmOffset::new(record.offset_ms);
                Some(offset.percentage_of_sixteenth(tempo))
            }
            _ => None,
        }
    }

    /// Returns the `StatisticsKey` for a rhythm offset detection record,
    /// combining the discipline with the tempo range and direction extracted from the record.
    ///
    /// Returns `None` if this is not the `RhythmOffsetDetection` discipline or the tempo is invalid.
    pub fn rhythm_offset_statistics_key(
        &self,
        record: &RhythmOffsetDetectionRecord,
    ) -> Option<StatisticsKey> {
        match self {
            TrainingDiscipline::RhythmOffsetDetection => {
                let tempo = TempoBPM::try_new(record.tempo_bpm).ok()?;
                if !record.offset_ms.is_finite() {
                    return None;
                }
                let direction = RhythmDirection::from_offset_ms(record.offset_ms);
                let tempo_range = TempoRange::from_bpm(tempo);
                Some(StatisticsKey::Rhythm(*self, tempo_range, direction))
            }
            _ => None,
        }
    }

    /// Extracts the metric (percentage of sixteenth note) from a continuous rhythm matching record
    /// if this is the `ContinuousRhythmMatching` discipline.
    ///
    /// Returns `None` for all other disciplines.
    pub fn extract_continuous_rhythm_metric(
        &self,
        record: &ContinuousRhythmMatchingRecord,
    ) -> Option<f64> {
        match self {
            TrainingDiscipline::ContinuousRhythmMatching => {
                let tempo = TempoBPM::try_new(record.tempo_bpm).ok()?;
                if !record.mean_offset_ms.is_finite() {
                    return None;
                }
                let offset = RhythmOffset::new(record.mean_offset_ms);
                Some(offset.percentage_of_sixteenth(tempo))
            }
            _ => None,
        }
    }

    /// Returns the `StatisticsKey` for a continuous rhythm matching record,
    /// combining the discipline with the tempo range and direction extracted from the record.
    ///
    /// Returns `None` if this is not the `ContinuousRhythmMatching` discipline or the tempo is invalid.
    pub fn continuous_rhythm_statistics_key(
        &self,
        record: &ContinuousRhythmMatchingRecord,
    ) -> Option<StatisticsKey> {
        match self {
            TrainingDiscipline::ContinuousRhythmMatching => {
                let tempo = TempoBPM::try_new(record.tempo_bpm).ok()?;
                if !record.mean_offset_ms.is_finite() {
                    return None;
                }
                let direction = RhythmDirection::from_offset_ms(record.mean_offset_ms);
                let tempo_range = TempoRange::from_bpm(tempo);
                Some(StatisticsKey::Rhythm(*self, tempo_range, direction))
            }
            _ => None,
        }
    }

    /// Extracts the metric value and corresponding `StatisticsKey` from any training record.
    ///
    /// Returns `None` if this discipline does not match the record type/content.
    pub fn extract_metric_and_key(&self, record: &TrainingRecord) -> Option<(f64, StatisticsKey)> {
        match record {
            TrainingRecord::PitchDiscrimination(r) => self
                .extract_discrimination_metric(r)
                .map(|m| (m, StatisticsKey::Pitch(*self))),
            TrainingRecord::PitchMatching(r) => self
                .extract_matching_metric(r)
                .map(|m| (m, StatisticsKey::Pitch(*self))),
            TrainingRecord::RhythmOffsetDetection(r) => {
                let metric = self.extract_rhythm_offset_metric(r)?;
                let key = self.rhythm_offset_statistics_key(r)?;
                Some((metric, key))
            }
            TrainingRecord::ContinuousRhythmMatching(r) => {
                let metric = self.extract_continuous_rhythm_metric(r)?;
                let key = self.continuous_rhythm_statistics_key(r)?;
                Some((metric, key))
            }
        }
    }

    /// URL-safe slug for routing.
    pub fn slug(&self) -> &'static str {
        match self {
            TrainingDiscipline::UnisonPitchDiscrimination => "unison-pitch-discrimination",
            TrainingDiscipline::IntervalPitchDiscrimination => "interval-pitch-discrimination",
            TrainingDiscipline::UnisonPitchMatching => "unison-pitch-matching",
            TrainingDiscipline::IntervalPitchMatching => "interval-pitch-matching",
            TrainingDiscipline::RhythmOffsetDetection => "rhythm-offset-detection",
            TrainingDiscipline::ContinuousRhythmMatching => "continuous-rhythm-matching",
        }
    }

    /// Parse a discipline from its slug. Returns `None` for unknown slugs.
    pub fn from_slug(slug: &str) -> Option<Self> {
        TrainingDiscipline::ALL
            .iter()
            .find(|d| d.slug() == slug)
            .copied()
    }

    /// Whether this is a rhythm discipline (expands to multiple statistics keys).
    pub fn is_rhythm(&self) -> bool {
        matches!(
            self,
            TrainingDiscipline::RhythmOffsetDetection
                | TrainingDiscipline::ContinuousRhythmMatching
        )
    }

    /// Returns all statistics keys for this discipline.
    /// Pitch disciplines: 1 key. Rhythm disciplines: 18 keys (6 tempo ranges × 3 directions).
    pub fn statistics_keys(&self) -> Vec<StatisticsKey> {
        if self.is_rhythm() {
            let mut keys = Vec::with_capacity(18);
            for tempo_range in TempoRange::ALL {
                for direction in RhythmDirection::ALL {
                    keys.push(StatisticsKey::Rhythm(*self, tempo_range, direction));
                }
            }
            keys
        } else {
            vec![StatisticsKey::Pitch(*self)]
        }
    }

    fn matches_interval(&self, interval: u8) -> bool {
        match self {
            TrainingDiscipline::UnisonPitchDiscrimination
            | TrainingDiscipline::UnisonPitchMatching => interval == 0,
            TrainingDiscipline::IntervalPitchDiscrimination
            | TrainingDiscipline::IntervalPitchMatching => interval != 0,
            // Rhythm disciplines don't use pitch intervals
            TrainingDiscipline::RhythmOffsetDetection
            | TrainingDiscipline::ContinuousRhythmMatching => false,
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
        assert_eq!(cfg.unit_label, "unit-cents");
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
        assert_eq!(cfg.unit_label, "unit-cents");
        assert_eq!(cfg.optimal_baseline, 12.0);
        assert_eq!(cfg.ewma_halflife_secs, 604_800.0);
        assert_eq!(cfg.session_gap_secs, 1_800.0);
    }

    #[test]
    fn test_unison_matching_config() {
        let cfg = TrainingDiscipline::UnisonPitchMatching.config();
        assert_eq!(cfg.display_name, "training-discipline-tune-match-single");
        assert_eq!(cfg.unit_label, "unit-cents");
        assert_eq!(cfg.optimal_baseline, 5.0);
        assert_eq!(cfg.ewma_halflife_secs, 604_800.0);
        assert_eq!(cfg.session_gap_secs, 1_800.0);
    }

    #[test]
    fn test_interval_matching_config() {
        let cfg = TrainingDiscipline::IntervalPitchMatching.config();
        assert_eq!(cfg.display_name, "training-discipline-tune-match-intervals");
        assert_eq!(cfg.unit_label, "unit-cents");
        assert_eq!(cfg.optimal_baseline, 8.0);
        assert_eq!(cfg.ewma_halflife_secs, 604_800.0);
        assert_eq!(cfg.session_gap_secs, 1_800.0);
    }

    #[test]
    fn test_all_contains_six_variants() {
        assert_eq!(TrainingDiscipline::ALL.len(), 6);
        assert!(TrainingDiscipline::ALL.contains(&TrainingDiscipline::UnisonPitchDiscrimination));
        assert!(TrainingDiscipline::ALL.contains(&TrainingDiscipline::IntervalPitchDiscrimination));
        assert!(TrainingDiscipline::ALL.contains(&TrainingDiscipline::UnisonPitchMatching));
        assert!(TrainingDiscipline::ALL.contains(&TrainingDiscipline::IntervalPitchMatching));
        assert!(TrainingDiscipline::ALL.contains(&TrainingDiscipline::RhythmOffsetDetection));
        assert!(TrainingDiscipline::ALL.contains(&TrainingDiscipline::ContinuousRhythmMatching));
    }

    // --- Metric extraction tests (AC: 5, 6) ---

    fn discrimination_record(interval: u8, cent_offset: f64) -> PitchDiscriminationRecord {
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
    fn test_extract_discrimination_metric_unison_matches_zero_interval() {
        let record = discrimination_record(0, 15.0);
        assert_eq!(
            TrainingDiscipline::UnisonPitchDiscrimination.extract_discrimination_metric(&record),
            Some(15.0)
        );
        assert_eq!(
            TrainingDiscipline::IntervalPitchDiscrimination.extract_discrimination_metric(&record),
            None
        );
    }

    #[test]
    fn test_extract_discrimination_metric_interval_matches_nonzero() {
        let record = discrimination_record(4, 10.0);
        assert_eq!(
            TrainingDiscipline::UnisonPitchDiscrimination.extract_discrimination_metric(&record),
            None
        );
        assert_eq!(
            TrainingDiscipline::IntervalPitchDiscrimination.extract_discrimination_metric(&record),
            Some(10.0)
        );
    }

    #[test]
    fn test_extract_discrimination_metric_negative_offset_returns_positive() {
        let record = discrimination_record(0, -25.0);
        assert_eq!(
            TrainingDiscipline::UnisonPitchDiscrimination.extract_discrimination_metric(&record),
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
        let comp = discrimination_record(0, 10.0);
        assert!(
            TrainingDiscipline::UnisonPitchDiscrimination
                .extract_discrimination_metric(&comp)
                .is_some()
        );
        assert!(
            TrainingDiscipline::IntervalPitchDiscrimination
                .extract_discrimination_metric(&comp)
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
        let comp = discrimination_record(1, 10.0);
        assert!(
            TrainingDiscipline::UnisonPitchDiscrimination
                .extract_discrimination_metric(&comp)
                .is_none()
        );
        assert!(
            TrainingDiscipline::IntervalPitchDiscrimination
                .extract_discrimination_metric(&comp)
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
        let comp = discrimination_record(0, 10.0);
        assert!(
            TrainingDiscipline::UnisonPitchMatching
                .extract_discrimination_metric(&comp)
                .is_none()
        );
        assert!(
            TrainingDiscipline::IntervalPitchMatching
                .extract_discrimination_metric(&comp)
                .is_none()
        );

        let comp_interval = discrimination_record(4, 10.0);
        assert!(
            TrainingDiscipline::UnisonPitchMatching
                .extract_discrimination_metric(&comp_interval)
                .is_none()
        );
        assert!(
            TrainingDiscipline::IntervalPitchMatching
                .extract_discrimination_metric(&comp_interval)
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

    // --- Rhythm discipline config tests ---

    #[test]
    fn test_rhythm_offset_detection_config() {
        let cfg = TrainingDiscipline::RhythmOffsetDetection.config();
        assert_eq!(cfg.display_name, "training-mode-compare-timing");
        assert_eq!(cfg.unit_label, "unit-percent-16th");
        assert_eq!(cfg.optimal_baseline, 5.0);
        assert_eq!(cfg.ewma_halflife_secs, 604_800.0);
        assert_eq!(cfg.session_gap_secs, 1_800.0);
    }

    #[test]
    fn test_continuous_rhythm_matching_config() {
        let cfg = TrainingDiscipline::ContinuousRhythmMatching.config();
        assert_eq!(cfg.display_name, "training-mode-fill-the-gap");
        assert_eq!(cfg.unit_label, "unit-percent-16th");
        assert_eq!(cfg.optimal_baseline, 5.0);
        assert_eq!(cfg.ewma_halflife_secs, 604_800.0);
        assert_eq!(cfg.session_gap_secs, 1_800.0);
    }

    // --- Rhythm metric extraction tests (AC6) ---

    #[test]
    fn test_rhythm_disciplines_return_none_for_discrimination_metric() {
        let record = discrimination_record(0, 10.0);
        assert!(
            TrainingDiscipline::RhythmOffsetDetection
                .extract_discrimination_metric(&record)
                .is_none()
        );
        assert!(
            TrainingDiscipline::ContinuousRhythmMatching
                .extract_discrimination_metric(&record)
                .is_none()
        );
    }

    #[test]
    fn test_rhythm_disciplines_return_none_for_matching_metric() {
        let record = matching_record(0, 5.0);
        assert!(
            TrainingDiscipline::RhythmOffsetDetection
                .extract_matching_metric(&record)
                .is_none()
        );
        assert!(
            TrainingDiscipline::ContinuousRhythmMatching
                .extract_matching_metric(&record)
                .is_none()
        );
    }

    // --- Slug tests (AC7) ---

    #[test]
    fn test_rhythm_offset_detection_slug() {
        assert_eq!(
            TrainingDiscipline::RhythmOffsetDetection.slug(),
            "rhythm-offset-detection"
        );
    }

    #[test]
    fn test_continuous_rhythm_matching_slug() {
        assert_eq!(
            TrainingDiscipline::ContinuousRhythmMatching.slug(),
            "continuous-rhythm-matching"
        );
    }

    #[test]
    fn test_all_slugs_unique() {
        let slugs: Vec<&str> = TrainingDiscipline::ALL.iter().map(|d| d.slug()).collect();
        let mut deduped = slugs.clone();
        deduped.sort();
        deduped.dedup();
        assert_eq!(slugs.len(), deduped.len());
    }

    #[test]
    fn test_from_slug_roundtrip() {
        for discipline in TrainingDiscipline::ALL {
            let slug = discipline.slug();
            assert_eq!(TrainingDiscipline::from_slug(slug), Some(discipline));
        }
    }

    #[test]
    fn test_from_slug_unknown_returns_none() {
        assert_eq!(TrainingDiscipline::from_slug("nonexistent"), None);
    }

    // --- statistics_keys tests ---

    #[test]
    fn test_pitch_discipline_has_one_key() {
        let keys = TrainingDiscipline::UnisonPitchDiscrimination.statistics_keys();
        assert_eq!(keys.len(), 1);
        assert_eq!(
            keys[0],
            StatisticsKey::Pitch(TrainingDiscipline::UnisonPitchDiscrimination)
        );
    }

    #[test]
    fn test_rhythm_discipline_has_eighteen_keys() {
        let keys = TrainingDiscipline::RhythmOffsetDetection.statistics_keys();
        assert_eq!(keys.len(), 18);
        // Verify all combinations present
        for tempo in TempoRange::ALL {
            for dir in RhythmDirection::ALL {
                assert!(keys.contains(&StatisticsKey::Rhythm(
                    TrainingDiscipline::RhythmOffsetDetection,
                    tempo,
                    dir,
                )));
            }
        }
    }

    #[test]
    fn test_all_pitch_disciplines_have_one_key() {
        for discipline in [
            TrainingDiscipline::UnisonPitchDiscrimination,
            TrainingDiscipline::IntervalPitchDiscrimination,
            TrainingDiscipline::UnisonPitchMatching,
            TrainingDiscipline::IntervalPitchMatching,
        ] {
            assert_eq!(discipline.statistics_keys().len(), 1, "{discipline:?}");
        }
    }

    #[test]
    fn test_all_rhythm_disciplines_have_eighteen_keys() {
        for discipline in [
            TrainingDiscipline::RhythmOffsetDetection,
            TrainingDiscipline::ContinuousRhythmMatching,
        ] {
            assert_eq!(discipline.statistics_keys().len(), 18, "{discipline:?}");
        }
    }

    #[test]
    fn test_is_rhythm() {
        assert!(!TrainingDiscipline::UnisonPitchDiscrimination.is_rhythm());
        assert!(!TrainingDiscipline::IntervalPitchDiscrimination.is_rhythm());
        assert!(!TrainingDiscipline::UnisonPitchMatching.is_rhythm());
        assert!(!TrainingDiscipline::IntervalPitchMatching.is_rhythm());
        assert!(TrainingDiscipline::RhythmOffsetDetection.is_rhythm());
        assert!(TrainingDiscipline::ContinuousRhythmMatching.is_rhythm());
    }

    // --- matches_interval for rhythm (AC4 of Task 4) ---

    #[test]
    fn test_rhythm_disciplines_matches_interval_always_false() {
        for interval in [0, 1, 4, 7, 12] {
            assert!(
                !TrainingDiscipline::RhythmOffsetDetection.matches_interval(interval),
                "RhythmOffsetDetection should not match interval {interval}"
            );
            assert!(
                !TrainingDiscipline::ContinuousRhythmMatching.matches_interval(interval),
                "ContinuousRhythmMatching should not match interval {interval}"
            );
        }
    }

    // --- Rhythm offset metric extraction tests ---

    fn rhythm_record(tempo_bpm: u16, offset_ms: f64) -> RhythmOffsetDetectionRecord {
        RhythmOffsetDetectionRecord {
            tempo_bpm,
            offset_ms,
            is_correct: true,
            timestamp: "2026-03-24T12:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_extract_rhythm_offset_metric_correct_discipline() {
        // 9.375ms at 80 BPM = 5.0%
        let record = rhythm_record(80, 9.375);
        let metric =
            TrainingDiscipline::RhythmOffsetDetection.extract_rhythm_offset_metric(&record);
        assert!((metric.unwrap() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_extract_rhythm_offset_metric_negative_offset() {
        let record = rhythm_record(80, -9.375);
        let metric =
            TrainingDiscipline::RhythmOffsetDetection.extract_rhythm_offset_metric(&record);
        assert!((metric.unwrap() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_extract_rhythm_offset_metric_wrong_discipline() {
        let record = rhythm_record(80, 9.375);
        assert!(
            TrainingDiscipline::UnisonPitchDiscrimination
                .extract_rhythm_offset_metric(&record)
                .is_none()
        );
        assert!(
            TrainingDiscipline::ContinuousRhythmMatching
                .extract_rhythm_offset_metric(&record)
                .is_none()
        );
    }

    #[test]
    fn test_rhythm_offset_statistics_key() {
        // 80 BPM → Moderate, negative offset → Early
        let record = rhythm_record(80, -5.0);
        let key = TrainingDiscipline::RhythmOffsetDetection.rhythm_offset_statistics_key(&record);
        assert_eq!(
            key,
            Some(StatisticsKey::Rhythm(
                TrainingDiscipline::RhythmOffsetDetection,
                TempoRange::Moderate,
                RhythmDirection::Early,
            ))
        );
    }

    #[test]
    fn test_rhythm_offset_statistics_key_slow_late() {
        // 60 BPM → Slow, positive offset → Late
        let record = rhythm_record(60, 10.0);
        let key = TrainingDiscipline::RhythmOffsetDetection.rhythm_offset_statistics_key(&record);
        assert_eq!(
            key,
            Some(StatisticsKey::Rhythm(
                TrainingDiscipline::RhythmOffsetDetection,
                TempoRange::Slow,
                RhythmDirection::Late,
            ))
        );
    }

    #[test]
    fn test_rhythm_offset_statistics_key_wrong_discipline() {
        let record = rhythm_record(80, 5.0);
        assert!(
            TrainingDiscipline::UnisonPitchDiscrimination
                .rhythm_offset_statistics_key(&record)
                .is_none()
        );
    }
}
