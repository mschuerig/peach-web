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
