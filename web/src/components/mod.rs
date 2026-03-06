mod pitch_comparison_view;
mod info_view;
mod page_nav;
mod pitch_matching_view;
mod pitch_slider;
mod profile_preview;
mod profile_view;
mod profile_visualization;
mod settings_view;
mod start_page;

pub use pitch_comparison_view::PitchComparisonView;
pub use info_view::InfoView;
pub use pitch_matching_view::PitchMatchingView;
// VerticalPitchSlider is used internally by PitchMatchingView via crate::components::pitch_slider
pub use profile_preview::ProfilePreview;
pub use profile_view::ProfileView;
pub use settings_view::SettingsView;
pub use start_page::StartPage;
