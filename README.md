# Peach

Peach is a pitch discrimination ear training web app. It helps musicians improve their ability to detect fine pitch differences through rapid, reflexive two-note comparisons.

**Repository:** https://github.com/mschuerig/peach-web

**Author:** Michael Schürig

## Project Status

Peach is in **active early development**. The core training loop, adaptive algorithm, and profile system are implemented and functional.

Known rough edges include a profile visualization that needs redesign, no onboarding for new users, and several UX improvements still in progress. See [future-work.md](docs/implementation-artifacts/future-work.md) for the full list of planned improvements.

## Philosophy

**Training, not testing.** Unlike traditional ear training apps that use test-and-score paradigms with gamification, Peach builds a perceptual profile of the user's hearing across their pitch range and adaptively targets weak spots. No scoring, no sessions, no guilt mechanics — every comparison makes you better.

## Features

- **Adaptive difficulty** — narrows intervals after correct answers, widens after incorrect ones, using Kazez convergence formulas
- **Weak-spot targeting** — concentrates training on notes where pitch discrimination is weakest
- **Perceptual profile** — piano keyboard visualization showing detection thresholds and confidence bands
- **Natural/Mechanical balance** — slider to control whether comparisons stay in nearby pitch regions or jump to target weak spots globally
- **Immediate feedback** — visual feedback after each comparison
- **Touch-friendly** — large tap targets, responsive layout for mobile and desktop
- **Localization** — English and German
- **Accessibility** — ARIA labels, adequate color contrast, minimum tap target sizes

## Requirements

- [Rust](https://www.rust-lang.org/tools/install) 1.85+ (edition 2024)
- `wasm32-unknown-unknown` target — `rustup target add wasm32-unknown-unknown`
- [Trunk](https://trunkrs.dev/) — `cargo install trunk`
- A modern web browser with Web Audio API support

## Building

Before the first build, download the SF2 SoundFont sample file:

```bash
./bin/download-sf2.sh
```

This downloads GeneralUser GS (~31 MB) to `.cache/` in the project root. The file is not tracked in git. You only need to run this once.

For development with live reload:

```bash
trunk serve
```

For a release build:

```bash
trunk build --release
```

The output goes to `dist/`.

## Running Tests

```bash
# Domain crate (pure Rust, runs natively)
cargo test -p domain
```

## Author's Note

This project has three purposes

- The obvious: Provide an app for ear training
- The ambitious: For me to gain experience with agentic software development. I'm using [Claude Code](https://code.claude.com/docs/) and the [BMad method](https://docs.bmad-method.org/) for development.
- The optimistic: Learn about Rust for web development.

## License

Source code is licensed under the [MIT License](LICENSE).

Audio samples and other media assets that may be added in the future could be covered by separate licenses. See [NOTICE](NOTICE) for third-party attribution details.
