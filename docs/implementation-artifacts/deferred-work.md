# Deferred Work

## Pre-existing: Training views don't stop session when navigating to Settings/Profile

**Source:** Review of tech-spec-true-back-navigation, 2026-03-30

Clicking the Settings or Profile icon from a training view navigates away without calling the session stop logic (`on_nav_away`). The training session continues running in the background (audio playing, training loop active). Only `on_cleanup` (component unmount) eventually stops it. This is pre-existing behavior — not introduced by the back-navigation change.
