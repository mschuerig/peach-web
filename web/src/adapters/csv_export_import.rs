use std::collections::HashSet;

use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, Url};

use domain::records::{
    ContinuousRhythmMatchingRecord, PitchDiscriminationRecord, PitchMatchingRecord,
    RhythmOffsetDetectionRecord, TrainingRecord,
};
use domain::{Interval, MIDINote};

use super::indexeddb_store::IndexedDbStore;

/// Status state machine for the reset confirmation flow.
#[derive(Clone, Copy, PartialEq)]
pub enum ResetStatus {
    Idle,
    Resetting,
    Success,
    Error,
}

/// Status state machine for the import/export flow.
#[derive(Clone, PartialEq)]
pub enum ImportExportStatus {
    Idle,
    Exporting,
    ExportSuccess,
    Importing,
    ImportSuccess(String),
    Error(String),
}

const CSV_HEADER: &str = "trainingType,timestamp,referenceNote,referenceNoteName,targetNote,targetNoteName,interval,tuningSystem,centOffset,isCorrect,tempoBPM,offsetMs,initialCentOffset,userCentError,meanOffsetMs,meanOffsetMsPosition0,meanOffsetMsPosition1,meanOffsetMsPosition2,meanOffsetMsPosition3";
const CSV_HEADER_V1: &str = "trainingType,timestamp,referenceNote,referenceNoteName,targetNote,targetNoteName,interval,tuningSystem,centOffset,isCorrect,initialCentOffset,userCentError";
#[allow(dead_code)] // Used in tests for consistency check against METADATA_LINE
const FORMAT_VERSION: u32 = 3;
const METADATA_PREFIX: &str = "# peach-export-format:";
const METADATA_LINE: &str = "# peach-export-format:3";

/// Result of parsing an import CSV file.
#[derive(Clone, Debug)]
pub struct ParsedImportData {
    pub pitch_discriminations: Vec<PitchDiscriminationRecord>,
    pub pitch_matchings: Vec<PitchMatchingRecord>,
    pub rhythm_offset_detections: Vec<RhythmOffsetDetectionRecord>,
    pub continuous_rhythm_matchings: Vec<ContinuousRhythmMatchingRecord>,
    pub warnings: Vec<String>,
}

/// Result of a merge import operation.
pub struct MergeResult {
    pub discrimination_imported: usize,
    pub discrimination_skipped: usize,
    pub pitch_matching_imported: usize,
    pub pitch_matching_skipped: usize,
    pub rhythm_offset_imported: usize,
    pub rhythm_offset_skipped: usize,
    pub continuous_rhythm_imported: usize,
    pub continuous_rhythm_skipped: usize,
}

/// Export all training data as a CSV file download.
pub async fn export_all_data(store: &IndexedDbStore) -> Result<(), String> {
    let mut all_records = store
        .fetch_all_records()
        .await
        .map_err(|e| format!("Failed to fetch training records: {e:?}"))?;

    all_records.sort_by(|a, b| a.timestamp().cmp(b.timestamp()));

    let mut csv = String::new();
    csv.push_str(METADATA_LINE);
    csv.push('\n');
    csv.push_str(CSV_HEADER);
    csv.push('\n');

    // NOTE: No RFC 4180 CSV escaping applied. Safe because all field values are
    // numeric, boolean, or fixed enum strings that never contain commas or quotes.
    // If user-provided string fields are added in the future, escaping must be added.
    for record in &all_records {
        match record {
            TrainingRecord::PitchDiscrimination(r) => {
                let interval_code = Interval::from_semitones(r.interval)
                    .ok()
                    .map(|i| i.csv_code())
                    .unwrap_or("P1");
                let ref_name = MIDINote::new(r.reference_note).name();
                let target_name = MIDINote::new(r.target_note).name();
                let ts = truncate_timestamp_to_second(&r.timestamp);
                csv.push_str(&format!(
                    "pitchDiscrimination,{},{},{},{},{},{},{},{},{},,,,,,,,,\n",
                    ts,
                    r.reference_note,
                    ref_name,
                    r.target_note,
                    target_name,
                    interval_code,
                    r.tuning_system,
                    r.cent_offset,
                    r.is_correct,
                ));
            }
            TrainingRecord::PitchMatching(r) => {
                let interval_code = Interval::from_semitones(r.interval)
                    .ok()
                    .map(|i| i.csv_code())
                    .unwrap_or("P1");
                let ref_name = MIDINote::new(r.reference_note).name();
                let target_name = MIDINote::new(r.target_note).name();
                let ts = truncate_timestamp_to_second(&r.timestamp);
                // cols 0-7 filled, 8-11 empty (4), 12-13 filled, 14-18 empty (5)
                csv.push_str(&format!(
                    "pitchMatching,{},{},{},{},{},{},{},,,,,{},{},,,,,\n",
                    ts,
                    r.reference_note,
                    ref_name,
                    r.target_note,
                    target_name,
                    interval_code,
                    r.tuning_system,
                    r.initial_cent_offset,
                    r.user_cent_error,
                ));
            }
            TrainingRecord::RhythmOffsetDetection(r) => {
                let ts = truncate_timestamp_to_second(&r.timestamp);
                // cols 0-1 filled, 2-8 empty (7), 9-11 filled, 12-18 empty (7)
                csv.push_str(&format!(
                    "rhythmOffsetDetection,{},,,,,,,,{},{},{},,,,,,,\n",
                    ts, r.is_correct, r.tempo_bpm, r.offset_ms,
                ));
            }
            TrainingRecord::ContinuousRhythmMatching(r) => {
                let ts = truncate_timestamp_to_second(&r.timestamp);
                let p0 = format_optional_f64(r.per_position_mean_ms[0]);
                let p1 = format_optional_f64(r.per_position_mean_ms[1]);
                let p2 = format_optional_f64(r.per_position_mean_ms[2]);
                let p3 = format_optional_f64(r.per_position_mean_ms[3]);
                // cols 0-1 filled, 2-9 empty (8), 10 filled, 11-13 empty (3), 14-18 filled
                csv.push_str(&format!(
                    "continuousRhythmMatching,{},,,,,,,,,{},,,,{},{},{},{},{}\n",
                    ts, r.tempo_bpm, r.mean_offset_ms, p0, p1, p2, p3,
                ));
            }
        }
    }

    // Generate filename with current date
    let now = js_sys::Date::new_0();
    let year = now.get_full_year();
    let month = now.get_month() + 1;
    let day = now.get_date();
    let filename = format!("peach-training-data-{year:04}-{month:02}-{day:02}.csv");

    trigger_download(&csv, &filename)?;

    Ok(())
}

/// Trigger a browser file download from a string.
fn trigger_download(content: &str, filename: &str) -> Result<(), String> {
    let array = js_sys::Array::of1(&wasm_bindgen::JsValue::from_str(content));
    let bag = BlobPropertyBag::new();
    bag.set_type("text/csv");
    let blob =
        Blob::new_with_str_sequence_and_options(&array, &bag).map_err(|e| format!("{e:?}"))?;
    let url = Url::create_object_url_with_blob(&blob).map_err(|e| format!("{e:?}"))?;

    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;
    let a: HtmlAnchorElement = document
        .create_element("a")
        .map_err(|e| format!("{e:?}"))?
        .unchecked_into();
    a.set_href(&url);
    a.set_download(filename);
    a.click();

    Ok(())
}

/// Format an `Option<f64>` as empty string for `None` or the number for `Some`.
fn format_optional_f64(value: Option<f64>) -> String {
    match value {
        Some(v) => v.to_string(),
        None => String::new(),
    }
}

/// Extract the format version number from the first line of a CSV file.
fn read_format_version(first_line: &str) -> Result<u32, String> {
    if let Some(version_str) = first_line.strip_prefix(METADATA_PREFIX) {
        version_str.parse::<u32>().map_err(|_| {
            format!("The file contains unreadable format metadata on line 1: '{first_line}'.")
        })
    } else {
        Err("This file does not contain format version metadata. It may have been created by an older version of the app. Please re-export your data with the current version.".to_string())
    }
}

/// Parse a CSV file's text content into structured records.
pub fn parse_import_file(content: &str) -> Result<ParsedImportData, String> {
    let content = content.trim();
    if content.is_empty() {
        return Err("File is empty".to_string());
    }

    let mut lines = content.lines();
    // Safe: content is non-empty after trim, so at least one line exists
    let first_line = lines
        .next()
        .expect("non-empty content has at least one line");
    let version = read_format_version(first_line)?;

    let header = lines
        .next()
        .ok_or("File has no header row after version line")?;

    match version {
        1 => {
            if header != CSV_HEADER_V1 {
                return Err(
                    "Invalid file format: header row does not match expected columns".to_string(),
                );
            }
            parse_v1(lines)
        }
        3 => {
            if header != CSV_HEADER {
                return Err(
                    "Invalid file format: header row does not match expected V3 columns"
                        .to_string(),
                );
            }
            parse_v3(lines)
        }
        v => Err(format!(
            "Unsupported export format version {v}. Please update the app to import this file."
        )),
    }
}

/// Parse CSV data rows in the v1 format.
fn parse_v1(lines: std::str::Lines) -> Result<ParsedImportData, String> {
    let mut pitch_discriminations = Vec::new();
    let mut pitch_matchings = Vec::new();
    let mut warnings = Vec::new();
    let mut has_data = false;

    for (line_num, line) in lines.enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        has_data = true;
        let row_num = line_num + 3; // 1-indexed: metadata is line 1, header is line 2

        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() < 12 {
            warnings.push(format!("Row {row_num}: too few columns, skipped"));
            continue;
        }

        let training_type = fields[0];
        match training_type {
            "pitchComparison" => match parse_pitch_discrimination_row(&fields, row_num) {
                Ok(record) => pitch_discriminations.push(record),
                Err(msg) => warnings.push(msg),
            },
            "pitchMatching" => match parse_pitch_matching_row(&fields, row_num) {
                Ok(record) => pitch_matchings.push(record),
                Err(msg) => warnings.push(msg),
            },
            other => {
                warnings.push(format!(
                    "Row {row_num}: unknown trainingType '{other}', skipped"
                ));
            }
        }
    }

    if !has_data {
        return Err("No records found in file".to_string());
    }

    Ok(ParsedImportData {
        pitch_discriminations,
        pitch_matchings,
        rhythm_offset_detections: Vec::new(),
        continuous_rhythm_matchings: Vec::new(),
        warnings,
    })
}

fn parse_pitch_discrimination_row(
    fields: &[&str],
    row_num: usize,
) -> Result<PitchDiscriminationRecord, String> {
    let timestamp = fields[1].to_string();
    let reference_note: u8 = fields[2]
        .parse()
        .map_err(|_| format!("Row {row_num}: invalid referenceNote, skipped"))?;
    let target_note: u8 = fields[4]
        .parse()
        .map_err(|_| format!("Row {row_num}: invalid targetNote, skipped"))?;
    let interval = Interval::from_csv_code(fields[6])
        .map(|i| i.semitones())
        .ok_or_else(|| {
            format!(
                "Row {row_num}: invalid interval code '{}', skipped",
                fields[6]
            )
        })?;
    let tuning_system = fields[7].to_string();
    let cent_offset: f64 = fields[8]
        .parse()
        .map_err(|_| format!("Row {row_num}: invalid centOffset, skipped"))?;
    let is_correct = match fields[9] {
        "true" => true,
        "false" => false,
        _ => {
            return Err(format!("Row {row_num}: invalid isCorrect value, skipped"));
        }
    };

    Ok(PitchDiscriminationRecord {
        reference_note,
        target_note,
        cent_offset,
        is_correct,
        interval,
        tuning_system,
        timestamp,
    })
}

fn parse_pitch_matching_row(
    fields: &[&str],
    row_num: usize,
) -> Result<PitchMatchingRecord, String> {
    let timestamp = fields[1].to_string();
    let reference_note: u8 = fields[2]
        .parse()
        .map_err(|_| format!("Row {row_num}: invalid referenceNote, skipped"))?;
    let target_note: u8 = fields[4]
        .parse()
        .map_err(|_| format!("Row {row_num}: invalid targetNote, skipped"))?;
    let interval = Interval::from_csv_code(fields[6])
        .map(|i| i.semitones())
        .ok_or_else(|| {
            format!(
                "Row {row_num}: invalid interval code '{}', skipped",
                fields[6]
            )
        })?;
    let tuning_system = fields[7].to_string();
    let initial_cent_offset: f64 = fields[10]
        .parse()
        .map_err(|_| format!("Row {row_num}: invalid initialCentOffset, skipped"))?;
    let user_cent_error: f64 = fields[11]
        .parse()
        .map_err(|_| format!("Row {row_num}: invalid userCentError, skipped"))?;

    Ok(PitchMatchingRecord {
        reference_note,
        target_note,
        initial_cent_offset,
        user_cent_error,
        interval,
        tuning_system,
        timestamp,
    })
}

/// Parse CSV data rows in the V3 format (19 columns, all 4 training types).
fn parse_v3(lines: std::str::Lines) -> Result<ParsedImportData, String> {
    let mut pitch_discriminations = Vec::new();
    let mut pitch_matchings = Vec::new();
    let mut rhythm_offset_detections = Vec::new();
    let mut continuous_rhythm_matchings = Vec::new();
    let mut warnings = Vec::new();
    let mut has_data = false;

    for (line_num, line) in lines.enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        has_data = true;
        let row_num = line_num + 3; // 1-indexed: metadata is line 1, header is line 2

        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() < 19 {
            warnings.push(format!(
                "Row {row_num}: too few columns (expected 19), skipped"
            ));
            continue;
        }

        let training_type = fields[0];
        match training_type {
            "pitchDiscrimination" | "pitchComparison" => {
                match parse_pitch_discrimination_row(&fields, row_num) {
                    Ok(record) => pitch_discriminations.push(record),
                    Err(msg) => warnings.push(msg),
                }
            }
            "pitchMatching" => match parse_pitch_matching_row_v3(&fields, row_num) {
                Ok(record) => pitch_matchings.push(record),
                Err(msg) => warnings.push(msg),
            },
            "rhythmOffsetDetection" => match parse_rhythm_offset_row(&fields, row_num) {
                Ok(record) => rhythm_offset_detections.push(record),
                Err(msg) => warnings.push(msg),
            },
            "continuousRhythmMatching" => match parse_continuous_rhythm_row(&fields, row_num) {
                Ok(record) => continuous_rhythm_matchings.push(record),
                Err(msg) => warnings.push(msg),
            },
            other => {
                warnings.push(format!(
                    "Row {row_num}: unknown trainingType '{other}', skipped"
                ));
            }
        }
    }

    if !has_data {
        return Err("No records found in file".to_string());
    }

    Ok(ParsedImportData {
        pitch_discriminations,
        pitch_matchings,
        rhythm_offset_detections,
        continuous_rhythm_matchings,
        warnings,
    })
}

/// Parse a V3 pitch matching row (initialCentOffset at col 12, userCentError at col 13).
fn parse_pitch_matching_row_v3(
    fields: &[&str],
    row_num: usize,
) -> Result<PitchMatchingRecord, String> {
    let timestamp = fields[1].to_string();
    let reference_note: u8 = fields[2]
        .parse()
        .map_err(|_| format!("Row {row_num}: invalid referenceNote, skipped"))?;
    let target_note: u8 = fields[4]
        .parse()
        .map_err(|_| format!("Row {row_num}: invalid targetNote, skipped"))?;
    let interval = Interval::from_csv_code(fields[6])
        .map(|i| i.semitones())
        .ok_or_else(|| {
            format!(
                "Row {row_num}: invalid interval code '{}', skipped",
                fields[6]
            )
        })?;
    let tuning_system = fields[7].to_string();
    // V3 columns: 12=initialCentOffset, 13=userCentError (shifted from V1 cols 10,11)
    let initial_cent_offset: f64 = fields[12]
        .parse()
        .map_err(|_| format!("Row {row_num}: invalid initialCentOffset, skipped"))?;
    let user_cent_error: f64 = fields[13]
        .parse()
        .map_err(|_| format!("Row {row_num}: invalid userCentError, skipped"))?;

    Ok(PitchMatchingRecord {
        reference_note,
        target_note,
        initial_cent_offset,
        user_cent_error,
        interval,
        tuning_system,
        timestamp,
    })
}

/// Parse a rhythm offset detection row from V3 CSV.
fn parse_rhythm_offset_row(
    fields: &[&str],
    row_num: usize,
) -> Result<RhythmOffsetDetectionRecord, String> {
    let timestamp = fields[1].to_string();
    let is_correct = match fields[9] {
        "true" => true,
        "false" => false,
        _ => {
            return Err(format!("Row {row_num}: invalid isCorrect value, skipped"));
        }
    };
    let tempo_bpm: u16 = fields[10]
        .parse()
        .map_err(|_| format!("Row {row_num}: invalid tempoBPM, skipped"))?;
    let offset_ms: f64 = fields[11]
        .parse()
        .map_err(|_| format!("Row {row_num}: invalid offsetMs, skipped"))?;

    Ok(RhythmOffsetDetectionRecord {
        tempo_bpm,
        offset_ms,
        is_correct,
        timestamp,
    })
}

/// Parse a continuous rhythm matching row from V3 CSV.
fn parse_continuous_rhythm_row(
    fields: &[&str],
    row_num: usize,
) -> Result<ContinuousRhythmMatchingRecord, String> {
    let timestamp = fields[1].to_string();
    let tempo_bpm: u16 = fields[10]
        .parse()
        .map_err(|_| format!("Row {row_num}: invalid tempoBPM, skipped"))?;
    let mean_offset_ms: f64 = fields[14]
        .parse()
        .map_err(|_| format!("Row {row_num}: invalid meanOffsetMs, skipped"))?;

    let parse_optional_f64 = |idx: usize, name: &str| -> Result<Option<f64>, String> {
        let val = fields[idx].trim();
        if val.is_empty() {
            Ok(None)
        } else {
            val.parse::<f64>()
                .map(Some)
                .map_err(|_| format!("Row {row_num}: invalid {name}, skipped"))
        }
    };

    let per_position_mean_ms = [
        parse_optional_f64(15, "meanOffsetMsPosition0")?,
        parse_optional_f64(16, "meanOffsetMsPosition1")?,
        parse_optional_f64(17, "meanOffsetMsPosition2")?,
        parse_optional_f64(18, "meanOffsetMsPosition3")?,
    ];

    Ok(ContinuousRhythmMatchingRecord {
        tempo_bpm,
        mean_offset_ms,
        hit_rate: 0.0,
        per_position_mean_ms,
        cycle_count: 0,
        timestamp,
    })
}

/// Import records in replace mode: delete all existing data, then save imported records.
/// Returns total number of records imported.
pub async fn import_replace(
    store: &IndexedDbStore,
    data: &ParsedImportData,
) -> Result<usize, String> {
    store
        .delete_all()
        .await
        .map_err(|e| format!("Failed to delete existing data: {e:?}"))?;

    for record in &data.pitch_discriminations {
        store
            .save_record(&TrainingRecord::PitchDiscrimination(record.clone()))
            .await
            .map_err(|e| format!("Failed to save pitch discrimination record: {e:?}"))?;
    }

    for record in &data.pitch_matchings {
        store
            .save_record(&TrainingRecord::PitchMatching(record.clone()))
            .await
            .map_err(|e| format!("Failed to save pitch matching: {e:?}"))?;
    }

    for record in &data.rhythm_offset_detections {
        store
            .save_record(&TrainingRecord::RhythmOffsetDetection(record.clone()))
            .await
            .map_err(|e| format!("Failed to save rhythm offset detection record: {e:?}"))?;
    }

    for record in &data.continuous_rhythm_matchings {
        store
            .save_record(&TrainingRecord::ContinuousRhythmMatching(record.clone()))
            .await
            .map_err(|e| format!("Failed to save continuous rhythm matching record: {e:?}"))?;
    }

    Ok(data.pitch_discriminations.len()
        + data.pitch_matchings.len()
        + data.rhythm_offset_detections.len()
        + data.continuous_rhythm_matchings.len())
}

/// Import records in merge mode: skip duplicates based on timestamp+type matching.
/// Returns counts of imported and skipped records.
pub async fn import_merge(
    store: &IndexedDbStore,
    data: &ParsedImportData,
) -> Result<MergeResult, String> {
    // Build sets of existing timestamps (truncated to second) per type
    let existing_records = store
        .fetch_all_records()
        .await
        .map_err(|e| format!("Failed to fetch existing records: {e:?}"))?;

    let mut existing_pitch_discrimination_ts: HashSet<String> = HashSet::new();
    let mut existing_pm_ts: HashSet<String> = HashSet::new();
    let mut existing_rod_ts: HashSet<String> = HashSet::new();
    let mut existing_crm_ts: HashSet<String> = HashSet::new();
    for record in &existing_records {
        match record {
            TrainingRecord::PitchDiscrimination(r) => {
                existing_pitch_discrimination_ts.insert(truncate_timestamp_to_second(&r.timestamp));
            }
            TrainingRecord::PitchMatching(r) => {
                existing_pm_ts.insert(truncate_timestamp_to_second(&r.timestamp));
            }
            TrainingRecord::RhythmOffsetDetection(r) => {
                existing_rod_ts.insert(truncate_timestamp_to_second(&r.timestamp));
            }
            TrainingRecord::ContinuousRhythmMatching(r) => {
                existing_crm_ts.insert(truncate_timestamp_to_second(&r.timestamp));
            }
        }
    }

    let mut result = MergeResult {
        discrimination_imported: 0,
        discrimination_skipped: 0,
        pitch_matching_imported: 0,
        pitch_matching_skipped: 0,
        rhythm_offset_imported: 0,
        rhythm_offset_skipped: 0,
        continuous_rhythm_imported: 0,
        continuous_rhythm_skipped: 0,
    };

    for record in &data.pitch_discriminations {
        let ts = truncate_timestamp_to_second(&record.timestamp);
        if existing_pitch_discrimination_ts.contains(&ts) {
            result.discrimination_skipped += 1;
        } else {
            store
                .save_record(&TrainingRecord::PitchDiscrimination(record.clone()))
                .await
                .map_err(|e| format!("Failed to save pitch discrimination record: {e:?}"))?;
            existing_pitch_discrimination_ts.insert(ts);
            result.discrimination_imported += 1;
        }
    }

    for record in &data.pitch_matchings {
        let ts = truncate_timestamp_to_second(&record.timestamp);
        if existing_pm_ts.contains(&ts) {
            result.pitch_matching_skipped += 1;
        } else {
            store
                .save_record(&TrainingRecord::PitchMatching(record.clone()))
                .await
                .map_err(|e| format!("Failed to save pitch matching: {e:?}"))?;
            existing_pm_ts.insert(ts);
            result.pitch_matching_imported += 1;
        }
    }

    for record in &data.rhythm_offset_detections {
        let ts = truncate_timestamp_to_second(&record.timestamp);
        if existing_rod_ts.contains(&ts) {
            result.rhythm_offset_skipped += 1;
        } else {
            store
                .save_record(&TrainingRecord::RhythmOffsetDetection(record.clone()))
                .await
                .map_err(|e| format!("Failed to save rhythm offset detection record: {e:?}"))?;
            existing_rod_ts.insert(ts);
            result.rhythm_offset_imported += 1;
        }
    }

    for record in &data.continuous_rhythm_matchings {
        let ts = truncate_timestamp_to_second(&record.timestamp);
        if existing_crm_ts.contains(&ts) {
            result.continuous_rhythm_skipped += 1;
        } else {
            store
                .save_record(&TrainingRecord::ContinuousRhythmMatching(record.clone()))
                .await
                .map_err(|e| format!("Failed to save continuous rhythm matching record: {e:?}"))?;
            existing_crm_ts.insert(ts);
            result.continuous_rhythm_imported += 1;
        }
    }

    Ok(result)
}

/// Read a `web_sys::File` as text using the FileReader API.
///
/// Wraps the callback-based FileReader into a Future.
pub async fn read_file_as_text(file: web_sys::File) -> Result<String, String> {
    let reader =
        web_sys::FileReader::new().map_err(|e| format!("Failed to create FileReader: {e:?}"))?;

    let (sender, receiver) = futures_channel::oneshot::channel::<Result<String, String>>();
    let mut sender = Some(sender);

    let reader_clone = reader.clone();
    let onload = Closure::once(move |_event: web_sys::Event| {
        let result = reader_clone
            .result()
            .map(|val| val.as_string().unwrap_or_default())
            .map_err(|e| format!("Failed to read file: {e:?}"));
        if let Some(s) = sender.take() {
            let _ = s.send(result);
        }
    });

    reader.set_onload(Some(onload.as_ref().unchecked_ref()));
    onload.forget();
    reader
        .read_as_text(&file)
        .map_err(|e| format!("Failed to start reading file: {e:?}"))?;

    receiver
        .await
        .map_err(|_| "FileReader channel closed".to_string())?
}

/// Truncate an ISO 8601 timestamp to second precision.
///
/// Strips fractional seconds: `"2026-03-04T14:30:00.456Z"` -> `"2026-03-04T14:30:00Z"`.
fn truncate_timestamp_to_second(ts: &str) -> String {
    if let Some(dot_pos) = ts.rfind('.')
        && let Some(tz_pos) = ts[dot_pos..].find(['Z', '+', '-'])
    {
        let mut result = ts[..dot_pos].to_string();
        result.push_str(&ts[dot_pos + tz_pos..]);
        return result;
    }
    ts.to_string()
}

/// Reload the page to rebuild the PerceptualProfile from stored records.
pub fn reload_page() {
    if let Some(window) = web_sys::window() {
        let _ = window.location().reload();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const V1_METADATA: &str = "# peach-export-format:1";

    /// Build a V3 CSV string with the metadata line, header, and the given data rows.
    fn make_csv(rows: &[&str]) -> String {
        let mut csv = String::new();
        csv.push_str(METADATA_LINE);
        csv.push('\n');
        csv.push_str(CSV_HEADER);
        csv.push('\n');
        for row in rows {
            csv.push_str(row);
            csv.push('\n');
        }
        csv
    }

    /// Build a V1 CSV string with V1 metadata, V1 header, and the given data rows.
    fn make_v1_csv(rows: &[&str]) -> String {
        let mut csv = String::new();
        csv.push_str(V1_METADATA);
        csv.push('\n');
        csv.push_str(CSV_HEADER_V1);
        csv.push('\n');
        for row in rows {
            csv.push_str(row);
            csv.push('\n');
        }
        csv
    }

    // --- Version reader tests ---

    #[test]
    fn test_read_format_version_valid() {
        assert_eq!(read_format_version("# peach-export-format:1"), Ok(1));
    }

    #[test]
    fn test_read_format_version_higher() {
        assert_eq!(read_format_version("# peach-export-format:42"), Ok(42));
    }

    #[test]
    fn test_read_format_version_missing_prefix() {
        let result = read_format_version("trainingType,timestamp,referenceNote");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("does not contain format version metadata")
        );
    }

    #[test]
    fn test_read_format_version_invalid_number() {
        let result = read_format_version("# peach-export-format:abc");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unreadable format metadata"));
    }

    #[test]
    fn test_read_format_version_empty_after_prefix() {
        let result = read_format_version("# peach-export-format:");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unreadable format metadata"));
    }

    // --- Metadata constant consistency ---

    #[test]
    fn test_metadata_line_matches_prefix_and_version() {
        assert_eq!(METADATA_LINE, format!("{METADATA_PREFIX}{FORMAT_VERSION}"));
    }

    // --- V1 import tests ---

    #[test]
    fn test_import_valid_v1_comparison() {
        let csv =
            make_v1_csv(&["pitchComparison,2026-03-04T14:30:00Z,60,C4,64,E4,M3,equal,0,true,,"]);
        let result = parse_import_file(&csv).unwrap();
        assert_eq!(result.pitch_discriminations.len(), 1);
        assert_eq!(result.pitch_matchings.len(), 0);
        assert_eq!(result.rhythm_offset_detections.len(), 0);
        assert_eq!(result.continuous_rhythm_matchings.len(), 0);
        assert!(result.warnings.is_empty());
        let r = &result.pitch_discriminations[0];
        assert_eq!(r.reference_note, 60);
        assert_eq!(r.target_note, 64);
        assert!(r.is_correct);
    }

    #[test]
    fn test_import_valid_v1_pitch_matching() {
        let csv =
            make_v1_csv(&["pitchMatching,2026-03-04T14:30:00Z,60,C4,67,G4,P5,equal,,,25.5,3.2"]);
        let result = parse_import_file(&csv).unwrap();
        assert_eq!(result.pitch_discriminations.len(), 0);
        assert_eq!(result.pitch_matchings.len(), 1);
        assert!(result.warnings.is_empty());
        let r = &result.pitch_matchings[0];
        assert_eq!(r.reference_note, 60);
        assert_eq!(r.target_note, 67);
        assert!((r.initial_cent_offset - 25.5).abs() < f64::EPSILON);
        assert!((r.user_cent_error - 3.2).abs() < f64::EPSILON);
    }

    #[test]
    fn test_import_v1_crlf_line_endings() {
        let csv = format!(
            "{V1_METADATA}\r\n{CSV_HEADER_V1}\r\npitchComparison,2026-03-04T14:30:00Z,60,C4,64,E4,M3,equal,0,true,,\r\n"
        );
        let result = parse_import_file(&csv).unwrap();
        assert_eq!(result.pitch_discriminations.len(), 1);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_import_v1_mixed_valid_and_invalid_rows() {
        let csv = make_v1_csv(&[
            "pitchComparison,2026-03-04T14:30:00Z,60,C4,64,E4,M3,equal,0,true,,",
            "pitchMatching,2026-03-04T14:31:00Z,60,C4,67,G4,P5,equal,,,25.5,3.2",
            "badType,2026-03-04T14:32:00Z,60,C4,64,E4,M3,equal,0,true,,",
        ]);
        let result = parse_import_file(&csv).unwrap();
        assert_eq!(result.pitch_discriminations.len(), 1);
        assert_eq!(result.pitch_matchings.len(), 1);
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].contains("unknown trainingType 'badType'"));
    }

    #[test]
    fn test_import_v1_malformed_comparison_fields() {
        let csv = make_v1_csv(&[
            "pitchComparison,2026-03-04T14:30:00Z,notanumber,C4,64,E4,M3,equal,0,true,,",
        ]);
        let result = parse_import_file(&csv).unwrap();
        assert_eq!(result.pitch_discriminations.len(), 0);
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].contains("invalid referenceNote"));
    }

    #[test]
    fn test_import_v1_too_few_columns_produces_warning() {
        let csv = make_v1_csv(&["pitchComparison,2026-03-04T14:30:00Z,60,C4,64"]);
        let result = parse_import_file(&csv).unwrap();
        assert_eq!(result.pitch_discriminations.len(), 0);
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].contains("too few columns"));
    }

    #[test]
    fn test_import_v1_rejects_old_comparison_type() {
        let csv = make_v1_csv(&["comparison,2026-03-04T14:30:00Z,60,C4,64,E4,M3,equal,0,true,,"]);
        let result = parse_import_file(&csv).unwrap();
        assert_eq!(result.pitch_discriminations.len(), 0);
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].contains("unknown trainingType 'comparison'"));
    }

    // --- General import error tests ---

    #[test]
    fn test_import_missing_version() {
        let csv = format!(
            "{CSV_HEADER}\ncomparison,2026-03-04T14:30:00Z,60,C4,64,E4,M3u,equal,0,true,,\n"
        );
        let result = parse_import_file(&csv);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("does not contain format version metadata")
        );
    }

    #[test]
    fn test_import_unsupported_version() {
        let csv = format!(
            "# peach-export-format:99\n{CSV_HEADER}\ncomparison,2026-03-04T14:30:00Z,60,C4,64,E4,M3u,equal,0,true,,\n"
        );
        let result = parse_import_file(&csv);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Unsupported export format version 99")
        );
    }

    #[test]
    fn test_import_invalid_metadata() {
        let csv = format!(
            "# peach-export-format:xyz\n{CSV_HEADER}\ncomparison,2026-03-04T14:30:00Z,60,C4,64,E4,M3u,equal,0,true,,\n"
        );
        let result = parse_import_file(&csv);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unreadable format metadata"));
    }

    #[test]
    fn test_import_empty_file() {
        let result = parse_import_file("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "File is empty");
    }

    #[test]
    fn test_import_version_only_no_data() {
        let csv = make_csv(&[]);
        let result = parse_import_file(&csv);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No records found"));
    }

    // --- V3 import tests ---

    #[test]
    fn test_import_v3_pitch_discrimination() {
        let csv = make_csv(&[
            "pitchDiscrimination,2026-03-04T14:30:00Z,60,C4,64,E4,M3,equal,0,true,,,,,,,,,",
        ]);
        let result = parse_import_file(&csv).unwrap();
        assert_eq!(result.pitch_discriminations.len(), 1);
        assert!(result.warnings.is_empty());
        let r = &result.pitch_discriminations[0];
        assert_eq!(r.reference_note, 60);
        assert_eq!(r.target_note, 64);
        assert!(r.is_correct);
    }

    #[test]
    fn test_import_v3_pitch_matching() {
        let csv = make_csv(&[
            "pitchMatching,2026-03-04T14:30:00Z,60,C4,67,G4,P5,equal,,,,,25.5,3.2,,,,,",
        ]);
        let result = parse_import_file(&csv).unwrap();
        assert_eq!(result.pitch_matchings.len(), 1);
        assert!(result.warnings.is_empty());
        let r = &result.pitch_matchings[0];
        assert_eq!(r.reference_note, 60);
        assert_eq!(r.target_note, 67);
        assert!((r.initial_cent_offset - 25.5).abs() < f64::EPSILON);
        assert!((r.user_cent_error - 3.2).abs() < f64::EPSILON);
    }

    #[test]
    fn test_import_v3_rhythm_offset_detection() {
        let csv =
            make_csv(&["rhythmOffsetDetection,2026-03-04T14:30:00Z,,,,,,,,true,120,15.5,,,,,,,"]);
        let result = parse_import_file(&csv).unwrap();
        assert_eq!(result.rhythm_offset_detections.len(), 1);
        assert!(result.warnings.is_empty());
        let r = &result.rhythm_offset_detections[0];
        assert_eq!(r.tempo_bpm, 120);
        assert!((r.offset_ms - 15.5).abs() < f64::EPSILON);
        assert!(r.is_correct);
    }

    #[test]
    fn test_import_v3_continuous_rhythm_matching() {
        let csv = make_csv(&[
            "continuousRhythmMatching,2026-03-04T14:30:00Z,,,,,,,,,80,,,,5.2,1.1,2.3,,4.5",
        ]);
        let result = parse_import_file(&csv).unwrap();
        assert_eq!(result.continuous_rhythm_matchings.len(), 1);
        assert!(result.warnings.is_empty());
        let r = &result.continuous_rhythm_matchings[0];
        assert_eq!(r.tempo_bpm, 80);
        assert!((r.mean_offset_ms - 5.2).abs() < f64::EPSILON);
        assert_eq!(r.per_position_mean_ms[0], Some(1.1));
        assert_eq!(r.per_position_mean_ms[1], Some(2.3));
        assert_eq!(r.per_position_mean_ms[2], None);
        assert_eq!(r.per_position_mean_ms[3], Some(4.5));
    }

    #[test]
    fn test_import_v3_unknown_training_type_produces_warning() {
        let csv =
            make_csv(&["unknownType,2026-03-04T14:30:00Z,60,C4,64,E4,M3,equal,0,true,,,,,,,,,"]);
        let result = parse_import_file(&csv).unwrap();
        assert_eq!(result.pitch_discriminations.len(), 0);
        assert_eq!(result.pitch_matchings.len(), 0);
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].contains("unknown trainingType 'unknownType'"));
    }

    #[test]
    fn test_import_v3_too_few_columns_produces_warning() {
        let csv = make_csv(&["pitchDiscrimination,2026-03-04T14:30:00Z,60,C4,64"]);
        let result = parse_import_file(&csv).unwrap();
        assert_eq!(result.pitch_discriminations.len(), 0);
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].contains("too few columns"));
    }

    #[test]
    fn test_import_v3_rejects_old_comparison_type() {
        let csv =
            make_csv(&["comparison,2026-03-04T14:30:00Z,60,C4,64,E4,M3,equal,0,true,,,,,,,,,"]);
        let result = parse_import_file(&csv).unwrap();
        assert_eq!(result.pitch_discriminations.len(), 0);
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].contains("unknown trainingType 'comparison'"));
    }

    #[test]
    fn test_import_v3_accepts_pitch_comparison_as_alias() {
        let csv = make_csv(&[
            "pitchComparison,2026-03-04T14:30:00Z,60,C4,64,E4,M3,equal,0,true,,,,,,,,,",
        ]);
        let result = parse_import_file(&csv).unwrap();
        assert_eq!(result.pitch_discriminations.len(), 1);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_import_v3_all_four_training_types() {
        let csv = make_csv(&[
            "pitchDiscrimination,2026-03-04T14:30:00Z,60,C4,64,E4,M3,equalTemperament,5.0,true,,,,,,,,,",
            "pitchMatching,2026-03-04T14:31:00Z,60,C4,67,G4,P5,equalTemperament,,,,,25.5,3.2,,,,,",
            "rhythmOffsetDetection,2026-03-04T14:32:00Z,,,,,,,,false,90,-12.3,,,,,,,",
            "continuousRhythmMatching,2026-03-04T14:33:00Z,,,,,,,,,100,,,,8.1,2.0,3.0,4.0,5.0",
        ]);
        let result = parse_import_file(&csv).unwrap();
        assert!(result.warnings.is_empty());
        assert_eq!(result.pitch_discriminations.len(), 1);
        assert_eq!(result.pitch_matchings.len(), 1);
        assert_eq!(result.rhythm_offset_detections.len(), 1);
        assert_eq!(result.continuous_rhythm_matchings.len(), 1);

        let rod = &result.rhythm_offset_detections[0];
        assert_eq!(rod.tempo_bpm, 90);
        assert!(!rod.is_correct);
        assert!((rod.offset_ms - (-12.3)).abs() < f64::EPSILON);

        let crm = &result.continuous_rhythm_matchings[0];
        assert_eq!(crm.tempo_bpm, 100);
        assert!((crm.mean_offset_ms - 8.1).abs() < f64::EPSILON);
        assert_eq!(
            crm.per_position_mean_ms,
            [Some(2.0), Some(3.0), Some(4.0), Some(5.0)]
        );
    }

    // --- Round-trip tests ---

    #[test]
    fn test_roundtrip_pitch_discrimination() {
        let original = PitchDiscriminationRecord {
            reference_note: 60,
            target_note: 64,
            cent_offset: 12.5,
            is_correct: true,
            interval: 4,
            tuning_system: "equalTemperament".to_string(),
            timestamp: "2026-03-04T14:30:00Z".to_string(),
        };
        // Build CSV row the same way export does
        let interval_code = Interval::from_semitones(original.interval)
            .ok()
            .map(|i| i.csv_code())
            .unwrap_or("P1");
        let ref_name = MIDINote::new(original.reference_note).name();
        let target_name = MIDINote::new(original.target_note).name();
        let row = format!(
            "pitchDiscrimination,{},{},{},{},{},{},{},{},{},,,,,,,,,",
            original.timestamp,
            original.reference_note,
            ref_name,
            original.target_note,
            target_name,
            interval_code,
            original.tuning_system,
            original.cent_offset,
            original.is_correct,
        );
        let csv = make_csv(&[&row]);
        let result = parse_import_file(&csv).unwrap();
        assert_eq!(result.pitch_discriminations.len(), 1);
        assert_eq!(result.pitch_discriminations[0], original);
    }

    #[test]
    fn test_roundtrip_pitch_matching() {
        let original = PitchMatchingRecord {
            reference_note: 60,
            target_note: 67,
            initial_cent_offset: 25.5,
            user_cent_error: 3.2,
            interval: 7,
            tuning_system: "justIntonation".to_string(),
            timestamp: "2026-03-04T14:31:00Z".to_string(),
        };
        let interval_code = Interval::from_semitones(original.interval)
            .ok()
            .map(|i| i.csv_code())
            .unwrap_or("P1");
        let ref_name = MIDINote::new(original.reference_note).name();
        let target_name = MIDINote::new(original.target_note).name();
        let row = format!(
            "pitchMatching,{},{},{},{},{},{},{},,,,,{},{},,,,,",
            original.timestamp,
            original.reference_note,
            ref_name,
            original.target_note,
            target_name,
            interval_code,
            original.tuning_system,
            original.initial_cent_offset,
            original.user_cent_error,
        );
        let csv = make_csv(&[&row]);
        let result = parse_import_file(&csv).unwrap();
        assert_eq!(result.pitch_matchings.len(), 1);
        assert_eq!(result.pitch_matchings[0], original);
    }

    #[test]
    fn test_roundtrip_rhythm_offset_detection() {
        let original = RhythmOffsetDetectionRecord {
            tempo_bpm: 120,
            offset_ms: -15.5,
            is_correct: false,
            timestamp: "2026-03-04T14:32:00Z".to_string(),
        };
        let row = format!(
            "rhythmOffsetDetection,{},,,,,,,,{},{},{},,,,,,,",
            original.timestamp, original.is_correct, original.tempo_bpm, original.offset_ms,
        );
        let csv = make_csv(&[&row]);
        let result = parse_import_file(&csv).unwrap();
        assert_eq!(result.rhythm_offset_detections.len(), 1);
        assert_eq!(result.rhythm_offset_detections[0], original);
    }

    #[test]
    fn test_roundtrip_continuous_rhythm_matching() {
        let original = ContinuousRhythmMatchingRecord {
            tempo_bpm: 80,
            mean_offset_ms: 5.2,
            hit_rate: 0.0,
            per_position_mean_ms: [Some(1.1), None, Some(3.3), Some(4.4)],
            cycle_count: 0,
            timestamp: "2026-03-04T14:33:00Z".to_string(),
        };
        let p0 = format_optional_f64(original.per_position_mean_ms[0]);
        let p1 = format_optional_f64(original.per_position_mean_ms[1]);
        let p2 = format_optional_f64(original.per_position_mean_ms[2]);
        let p3 = format_optional_f64(original.per_position_mean_ms[3]);
        let row = format!(
            "continuousRhythmMatching,{},,,,,,,,,{},,,,{},{},{},{},{}",
            original.timestamp, original.tempo_bpm, original.mean_offset_ms, p0, p1, p2, p3,
        );
        let csv = make_csv(&[&row]);
        let result = parse_import_file(&csv).unwrap();
        assert_eq!(result.continuous_rhythm_matchings.len(), 1);
        assert_eq!(result.continuous_rhythm_matchings[0], original);
    }

    // --- Cross-platform import test (iOS V3 fixture) ---

    #[test]
    fn test_import_ios_v3_fixture() {
        // Simulate an iOS-exported V3 file with all 4 training types
        let csv = format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n",
            "# peach-export-format:3",
            CSV_HEADER,
            "pitchDiscrimination,2026-01-15T10:00:00Z,69,A4,72,C5,m3,equalTemperament,8.5,true,,,,,,,,,",
            "pitchMatching,2026-01-15T10:01:00Z,60,C4,60,C4,P1,justIntonation,,,,,10.0,2.5,,,,,",
            "rhythmOffsetDetection,2026-01-15T10:02:00Z,,,,,,,,true,100,5.0,,,,,,,",
            "continuousRhythmMatching,2026-01-15T10:03:00Z,,,,,,,,,120,,,,3.5,1.0,2.0,3.0,4.0",
        );
        let result = parse_import_file(&csv).unwrap();
        assert!(result.warnings.is_empty());
        assert_eq!(result.pitch_discriminations.len(), 1);
        assert_eq!(result.pitch_matchings.len(), 1);
        assert_eq!(result.rhythm_offset_detections.len(), 1);
        assert_eq!(result.continuous_rhythm_matchings.len(), 1);

        // Verify pitch discrimination fields
        let pd = &result.pitch_discriminations[0];
        assert_eq!(pd.reference_note, 69);
        assert_eq!(pd.target_note, 72);
        assert_eq!(pd.interval, 3); // minor third
        assert_eq!(pd.tuning_system, "equalTemperament");
        assert!((pd.cent_offset - 8.5).abs() < f64::EPSILON);
        assert!(pd.is_correct);

        // Verify pitch matching fields
        let pm = &result.pitch_matchings[0];
        assert_eq!(pm.tuning_system, "justIntonation");
        assert!((pm.initial_cent_offset - 10.0).abs() < f64::EPSILON);

        // Verify rhythm offset detection fields
        let rod = &result.rhythm_offset_detections[0];
        assert_eq!(rod.tempo_bpm, 100);
        assert!((rod.offset_ms - 5.0).abs() < f64::EPSILON);

        // Verify continuous rhythm matching fields
        let crm = &result.continuous_rhythm_matchings[0];
        assert_eq!(crm.tempo_bpm, 120);
        assert!((crm.mean_offset_ms - 3.5).abs() < f64::EPSILON);
        assert_eq!(
            crm.per_position_mean_ms,
            [Some(1.0), Some(2.0), Some(3.0), Some(4.0)]
        );
    }

    // --- V3 header column count consistency ---

    #[test]
    fn test_v3_header_has_19_columns() {
        assert_eq!(CSV_HEADER.split(',').count(), 19);
    }

    #[test]
    fn test_v1_header_has_12_columns() {
        assert_eq!(CSV_HEADER_V1.split(',').count(), 12);
    }

    // --- format_optional_f64 tests ---

    #[test]
    fn test_format_optional_f64_some() {
        assert_eq!(format_optional_f64(Some(3.14)), "3.14");
    }

    #[test]
    fn test_format_optional_f64_none() {
        assert_eq!(format_optional_f64(None), "");
    }
}
