use crate::components::help_content::HelpSection;

pub static SETTINGS_HELP: &[HelpSection] = &[
    HelpSection {
        title_key: "help-training-range-title",
        body_key: "help-training-range-body",
    },
    HelpSection {
        title_key: "help-intervals-title",
        body_key: "help-intervals-body",
    },
    HelpSection {
        title_key: "help-sound-title",
        body_key: "help-sound-body",
    },
    HelpSection {
        title_key: "help-difficulty-title",
        body_key: "help-difficulty-body",
    },
    HelpSection {
        title_key: "help-data-title",
        body_key: "help-data-body",
    },
];

pub static COMPARISON_HELP: &[HelpSection] = &[
    HelpSection {
        title_key: "help-comparison-goal-title",
        body_key: "help-comparison-goal-body",
    },
    HelpSection {
        title_key: "help-comparison-controls-title",
        body_key: "help-comparison-controls-body",
    },
    HelpSection {
        title_key: "help-comparison-feedback-title",
        body_key: "help-comparison-feedback-body",
    },
    HelpSection {
        title_key: "help-comparison-difficulty-title",
        body_key: "help-comparison-difficulty-body",
    },
    HelpSection {
        title_key: "help-comparison-intervals-title",
        body_key: "help-comparison-intervals-body",
    },
];

pub static PITCH_MATCHING_HELP: &[HelpSection] = &[
    HelpSection {
        title_key: "help-matching-goal-title",
        body_key: "help-matching-goal-body",
    },
    HelpSection {
        title_key: "help-matching-controls-title",
        body_key: "help-matching-controls-body",
    },
    HelpSection {
        title_key: "help-matching-feedback-title",
        body_key: "help-matching-feedback-body",
    },
    HelpSection {
        title_key: "help-matching-intervals-title",
        body_key: "help-matching-intervals-body",
    },
];

pub static INFO_HELP: &[HelpSection] = &[
    HelpSection {
        title_key: "help-info-what-title",
        body_key: "help-info-what-body",
    },
    HelpSection {
        title_key: "help-info-modes-title",
        body_key: "help-info-modes-body",
    },
    HelpSection {
        title_key: "help-info-start-title",
        body_key: "help-info-start-body",
    },
];

pub static INFO_ACKNOWLEDGMENTS: &[HelpSection] = &[HelpSection {
    title_key: "acknowledgments",
    body_key: "acknowledgments-body",
}];
