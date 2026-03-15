use serde::{Deserialize, Serialize};

/// Sound source identifier — defaults to "sf2:0:0" if empty.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SoundSourceID {
    raw_value: String,
}

impl SoundSourceID {
    const DEFAULT: &str = "sf2:0:0";

    /// Access the raw sound source identifier string.
    pub fn raw_value(&self) -> &str {
        &self.raw_value
    }

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
        assert_eq!(s.raw_value, "sf2:0:0");
    }

    #[test]
    fn test_sound_source_default_trait() {
        let s = SoundSourceID::default();
        assert_eq!(s.raw_value, "sf2:0:0");
    }

    #[test]
    fn test_sound_source_serde_roundtrip() {
        let s = SoundSourceID::new("custom:source".to_string());
        let json = serde_json::to_string(&s).unwrap();
        let parsed: SoundSourceID = serde_json::from_str(&json).unwrap();
        assert_eq!(s, parsed);
    }
}
