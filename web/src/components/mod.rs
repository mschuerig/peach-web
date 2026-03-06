mod pitch_comparison_view;
mod info_view;
mod page_nav;
mod pitch_matching_view;
mod pitch_slider;
mod progress_card;
mod progress_chart;
mod progress_sparkline;
mod profile_view;
mod settings_view;
mod start_page;
mod training_stats;

pub use pitch_comparison_view::PitchComparisonView;
pub use info_view::InfoView;
pub use pitch_matching_view::PitchMatchingView;
// VerticalPitchSlider is used internally by PitchMatchingView via crate::components::pitch_slider
pub use profile_view::ProfileView;
pub use settings_view::SettingsView;
pub use start_page::StartPage;
pub use training_stats::TrainingStats;
