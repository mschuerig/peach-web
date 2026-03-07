use std::collections::HashSet;

use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, Url};

use domain::records::{PitchComparisonRecord, PitchMatchingRecord};
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

const CSV_HEADER: &str = "trainingType,timestamp,referenceNote,referenceNoteName,targetNote,targetNoteName,interval,tuningSystem,centOffset,isCorrect,initialCentOffset,userCentError";

/// Result of parsing an import CSV file.
#[derive(Clone)]
pub struct ParsedImportData {
    pub pitch_comparisons: Vec<PitchComparisonRecord>,
    pub pitch_matchings: Vec<PitchMatchingRecord>,
    pub warnings: Vec<String>,
}

/// Result of a merge import operation.
pub struct MergeResult {
    pub comparison_imported: usize,
    pub comparison_skipped: usize,
    pub pitch_matching_imported: usize,
    pub pitch_matching_skipped: usize,
}

/// Export all training data as a CSV file download.
pub async fn export_all_data(store: &IndexedDbStore) -> Result<(), String> {
    let pitch_comparisons = store
        .fetch_all_pitch_comparisons()
        .await
        .map_err(|e| format!("Failed to fetch pitch comparisons: {e:?}"))?;
    let pitch_matchings = store
        .fetch_all_pitch_matchings()
        .await
        .map_err(|e| format!("Failed to fetch pitch matchings: {e:?}"))?;

    let mut csv = String::new();
    csv.push_str(CSV_HEADER);
    csv.push('\n');

    // Collect all records with timestamps for chronological sorting
    enum Record<'a> {
        Comparison(&'a PitchComparisonRecord),
        PitchMatching(&'a PitchMatchingRecord),
    }

    let mut all_records: Vec<Record> =
        Vec::with_capacity(pitch_comparisons.len() + pitch_matchings.len());
    for r in &pitch_comparisons {
        all_records.push(Record::Comparison(r));
    }
    for r in &pitch_matchings {
        all_records.push(Record::PitchMatching(r));
    }

    all_records.sort_by(|a, b| {
        let ts_a = match a {
            Record::Comparison(r) => &r.timestamp,
            Record::PitchMatching(r) => &r.timestamp,
        };
        let ts_b = match b {
            Record::Comparison(r) => &r.timestamp,
            Record::PitchMatching(r) => &r.timestamp,
        };
        ts_a.cmp(ts_b)
    });

    // NOTE: No RFC 4180 CSV escaping applied. Safe because all field values are
    // numeric, boolean, or fixed enum strings that never contain commas or quotes.
    // If user-provided string fields are added in the future, escaping must be added.
    for record in &all_records {
        match record {
            Record::Comparison(r) => {
                let interval_code = Interval::from_semitones(r.interval)
                    .ok()
                    .map(|i| i.csv_code())
                    .unwrap_or("P1");
                let ref_name = MIDINote::new(r.reference_note).name();
                let target_name = MIDINote::new(r.target_note).name();
                let ts = truncate_timestamp_to_second(&r.timestamp);
                csv.push_str(&format!(
                    "comparison,{},{},{},{},{},{},{},{},{},,\n",
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
            Record::PitchMatching(r) => {
                let interval_code = Interval::from_semitones(r.interval)
                    .ok()
                    .map(|i| i.csv_code())
                    .unwrap_or("P1");
                let ref_name = MIDINote::new(r.reference_note).name();
                let target_name = MIDINote::new(r.target_note).name();
                let ts = truncate_timestamp_to_second(&r.timestamp);
                csv.push_str(&format!(
                    "pitchMatching,{},{},{},{},{},{},{},,,{},{}\n",
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

/// Parse a CSV file's text content into structured records.
pub fn parse_import_file(content: &str) -> Result<ParsedImportData, String> {
    let content = content.trim();
    if content.is_empty() {
        return Err("File is empty".to_string());
    }

    let mut lines = content.lines();
    let header = lines.next().ok_or("File is empty")?;

    if header != CSV_HEADER {
        return Err("Invalid file format: header row does not match expected columns".to_string());
    }

    let mut pitch_comparisons = Vec::new();
    let mut pitch_matchings = Vec::new();
    let mut warnings = Vec::new();
    let mut has_data = false;

    for (line_num, line) in lines.enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        has_data = true;
        let row_num = line_num + 2; // 1-indexed, header is line 1

        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() < 12 {
            warnings.push(format!("Row {row_num}: too few columns, skipped"));
            continue;
        }

        let training_type = fields[0];
        match training_type {
            "comparison" => match parse_comparison_row(&fields, row_num) {
                Ok(record) => pitch_comparisons.push(record),
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
        pitch_comparisons,
        pitch_matchings,
        warnings,
    })
}

fn parse_comparison_row(fields: &[&str], row_num: usize) -> Result<PitchComparisonRecord, String> {
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

    Ok(PitchComparisonRecord {
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

    for record in &data.pitch_comparisons {
        store
            .save_pitch_comparison(record)
            .await
            .map_err(|e| format!("Failed to save comparison: {e:?}"))?;
    }

    for record in &data.pitch_matchings {
        store
            .save_pitch_matching(record)
            .await
            .map_err(|e| format!("Failed to save pitch matching: {e:?}"))?;
    }

    Ok(data.pitch_comparisons.len() + data.pitch_matchings.len())
}

/// Import records in merge mode: skip duplicates based on timestamp+type comparison.
/// Returns counts of imported and skipped records.
pub async fn import_merge(
    store: &IndexedDbStore,
    data: &ParsedImportData,
) -> Result<MergeResult, String> {
    // Build sets of existing timestamps (truncated to second) per type
    let existing_pitch_comparisons = store
        .fetch_all_pitch_comparisons()
        .await
        .map_err(|e| format!("Failed to fetch pitch comparisons: {e:?}"))?;
    let mut existing_pitch_comparison_ts: HashSet<String> = existing_pitch_comparisons
        .iter()
        .map(|r| truncate_timestamp_to_second(&r.timestamp))
        .collect();

    let existing_pitch_matchings = store
        .fetch_all_pitch_matchings()
        .await
        .map_err(|e| format!("Failed to fetch pitch matchings: {e:?}"))?;
    let mut existing_pm_ts: HashSet<String> = existing_pitch_matchings
        .iter()
        .map(|r| truncate_timestamp_to_second(&r.timestamp))
        .collect();

    let mut result = MergeResult {
        comparison_imported: 0,
        comparison_skipped: 0,
        pitch_matching_imported: 0,
        pitch_matching_skipped: 0,
    };

    for record in &data.pitch_comparisons {
        let ts = truncate_timestamp_to_second(&record.timestamp);
        if existing_pitch_comparison_ts.contains(&ts) {
            result.comparison_skipped += 1;
        } else {
            store
                .save_pitch_comparison(record)
                .await
                .map_err(|e| format!("Failed to save comparison: {e:?}"))?;
            existing_pitch_comparison_ts.insert(ts);
            result.comparison_imported += 1;
        }
    }

    for record in &data.pitch_matchings {
        let ts = truncate_timestamp_to_second(&record.timestamp);
        if existing_pm_ts.contains(&ts) {
            result.pitch_matching_skipped += 1;
        } else {
            store
                .save_pitch_matching(record)
                .await
                .map_err(|e| format!("Failed to save pitch matching: {e:?}"))?;
            existing_pm_ts.insert(ts);
            result.pitch_matching_imported += 1;
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
