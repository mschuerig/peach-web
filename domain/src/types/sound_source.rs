use serde::{Deserialize, Serialize};

/// Sound source identifier — defaults to "sf2:8:80" if empty.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SoundSourceID {
    pub raw_value: String,
}

impl SoundSourceID {
    const DEFAULT: &str = "sf2:8:80";

    pub fn new(raw_value: String) -> Self {
        if raw_value.is_empty() {
            Self {
                raw_value: Self::DEFAULT.to_string(),
            }
        } else {
            Self { raw_value }
        }
    }
}

impl Default for SoundSourceID {
    fn default() -> Self {
        Self {
            raw_value: Self::DEFAULT.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sound_source_custom() {
        let s = SoundSourceID::new("oscillator:sine".to_string());
        assert_eq!(s.raw_value, "oscillator:sine");
    }

    #[test]
    fn test_sound_source_empty_defaults() {
        let s = SoundSourceID::new(String::new());
        assert_eq!(s.raw_value, "sf2:8:80");
    }

    #[test]
    fn test_sound_source_default_trait() {
        let s = SoundSourceID::default();
        assert_eq!(s.raw_value, "sf2:8:80");
    }
}
