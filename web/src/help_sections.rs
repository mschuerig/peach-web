use crate::components::help_content::HelpSection;

pub static SETTINGS_HELP: &[HelpSection] = &[
    HelpSection {
        title: "Training Range",
        body: "Set the **lowest** and **highest note** for your training. A wider range is more challenging. If you're just starting out, try a smaller range and expand it as your ear improves.",
    },
    HelpSection {
        title: "Intervals",
        body: "Intervals are the distance between two notes. Choose which intervals you want to practice. Start with a few and add more as you gain confidence.",
    },
    HelpSection {
        title: "Sound",
        body: "Pick the **sound** you want to train with \u{2014} each instrument has a different character.\n\n**Duration** controls how long each note plays.\n\n**Concert Pitch** sets the reference tuning. Most musicians use 440 Hz. Some orchestras tune to 442 Hz.\n\n**Tuning System** determines how intervals are calculated. Equal Temperament divides the octave into 12 equal steps and is standard for most Western music. Just Intonation uses pure frequency ratios and sounds smoother for some intervals.",
    },
    HelpSection {
        title: "Difficulty",
        body: "**Vary Loudness** changes the volume of notes randomly. This makes training harder but more realistic \u{2014} in real music, notes are rarely played at the same volume.",
    },
    HelpSection {
        title: "Data",
        body: "**Export** saves your training data as a file you can keep as a backup or transfer to another device.\n\n**Import** loads training data from a file. You can replace your current data or merge it with existing records.\n\n**Reset** permanently deletes all training data and resets your profile. This cannot be undone.",
    },
];

pub static COMPARISON_HELP: &[HelpSection] = &[
    HelpSection {
        title: "Goal",
        body: "Two notes play one after the other. Your job is to decide: was the **second note higher or lower** than the first? The closer the notes are, the harder it gets \u{2014} and the sharper your ear becomes.",
    },
    HelpSection {
        title: "Controls",
        body: "After both notes have played, the **Higher** and **Lower** buttons become active. Tap the one that matches what you heard. You can also use keyboard shortcuts: **Arrow Up** or **H** for higher, **Arrow Down** or **L** for lower.",
    },
    HelpSection {
        title: "Feedback",
        body: "After each answer you'll see a brief **checkmark** (correct) or **X** (incorrect). Use this to calibrate your listening \u{2014} over time, you'll notice patterns in what you get right.",
    },
    HelpSection {
        title: "Difficulty",
        body: "The difference between the two notes is measured in **cents** (1/100 of a semitone). A smaller number means a harder comparison. The app adapts difficulty to your skill level automatically.",
    },
    HelpSection {
        title: "Intervals",
        body: "In interval mode, the two notes are separated by a specific **musical interval** (like a fifth or an octave) instead of a small pitch difference. You still decide which note is higher \u{2014} but now you're training your sense of musical distance.",
    },
];

pub static PITCH_MATCHING_HELP: &[HelpSection] = &[
    HelpSection {
        title: "Goal",
        body: "You'll hear a **reference note**. Your goal is to match its pitch by sliding to the exact same frequency. The closer you get, the better your ear is becoming.",
    },
    HelpSection {
        title: "Controls",
        body: "**Touch** the slider to hear your note, then **drag** up or down to adjust the pitch. When you think you've matched the reference, **release** the slider to lock in your answer. You can also press **Enter** or **Space** to commit.",
    },
    HelpSection {
        title: "Feedback",
        body: "After each attempt, you'll see how many **cents** off you were. A smaller number means a closer match \u{2014} zero would be perfect. Use the feedback to fine-tune your listening.",
    },
    HelpSection {
        title: "Intervals",
        body: "In interval mode, your target pitch is a specific **musical interval** away from the reference note. Instead of matching the same note, you're matching a note that's a certain distance above or below it.",
    },
];

pub static INFO_HELP: &[HelpSection] = &[
    HelpSection {
        title: "What is Peach?",
        body: "Peach helps you train your ear for music. Practice hearing the difference between notes and learn to match pitches accurately.",
    },
    HelpSection {
        title: "Training Modes",
        body: "**Hear & Compare \u{2014} Single Notes** \u{2014} Listen to two notes and decide which one is higher.\n\n**Hear & Compare \u{2014} Intervals** \u{2014} The same idea, but with musical intervals between notes.\n\n**Tune & Match \u{2014} Single Notes** \u{2014} Hear a note and slide to match its pitch.\n\n**Tune & Match \u{2014} Intervals** \u{2014} Match pitches using musical intervals.",
    },
    HelpSection {
        title: "Getting Started",
        body: "Just pick any training mode on the home screen and start practicing. Peach adapts to your skill level automatically.",
    },
];

pub static INFO_ACKNOWLEDGMENTS: &[HelpSection] = &[
    HelpSection {
        title: "Acknowledgments",
        body: "Sounds provided by GeneralUser GS by S. Christian Collins.",
    },
];
