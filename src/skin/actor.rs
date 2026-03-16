// skin::actor - Actor rendering styles
// Port of Java PlantUML's skin.ActorStyle + Actor*
// Stub - to be filled by agent

/// Actor visual style. Java: `skin.ActorStyle`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActorStyle {
    #[default]
    Stickman,
    Awesome,
    Hollow,
}

impl ActorStyle {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "stickman" | "stick" => Some(Self::Stickman),
            "awesome" => Some(Self::Awesome),
            "hollow" => Some(Self::Hollow),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_style() {
        assert_eq!(ActorStyle::from_str("awesome"), Some(ActorStyle::Awesome));
        assert_eq!(ActorStyle::from_str("hollow"), Some(ActorStyle::Hollow));
        assert_eq!(ActorStyle::from_str("stick"), Some(ActorStyle::Stickman));
        assert!(ActorStyle::from_str("unknown").is_none());
    }
}
