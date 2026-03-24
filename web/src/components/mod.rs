mod audio_gate_overlay;
pub mod help_content;
mod info_view;
pub mod nav_bar;
mod pitch_discrimination_view;
mod pitch_matching_view;
mod pitch_slider;
mod profile_view;
mod progress_card;
mod progress_chart;
mod progress_sparkline;
mod rhythm_placeholder_view;
mod settings_view;
mod start_page;
mod training_stats;

pub use info_view::InfoView;
pub use pitch_discrimination_view::PitchDiscriminationView;
pub use pitch_matching_view::PitchMatchingView;
// VerticalPitchSlider is used internally by PitchMatchingView via crate::components::pitch_slider
pub use profile_view::ProfileView;
pub use rhythm_placeholder_view::{ContinuousRhythmMatchingView, RhythmOffsetDetectionView};
pub use settings_view::SettingsView;
pub use start_page::StartPage;
pub use training_stats::TrainingStats;
