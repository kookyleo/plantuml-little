// skin::arrow - Arrow configuration for sequence diagrams
// Port of Java PlantUML's skin.Arrow* classes
// Stub - to be filled by agent

/// Arrow head style. Java: `skin.ArrowHead`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArrowHead {
    #[default]
    Normal,
    Async,
    CrossX,
    None,
}

/// Arrow body line style. Java: `skin.ArrowBody`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArrowBody {
    #[default]
    Normal,
    Dotted,
    Hidden,
}

/// Arrow direction. Java: `skin.ArrowDirection`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArrowDirection {
    #[default]
    LeftToRight,
    RightToLeft,
    Self_,
    Both,
}

/// Arrow part (which half to draw for self-messages). Java: `skin.ArrowPart`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArrowPart {
    #[default]
    Full,
    TopPart,
    BottomPart,
}

/// Arrow endpoint dressing. Java: `skin.ArrowDressing`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArrowDressing {
    pub head: ArrowHead,
}

impl ArrowDressing {
    pub fn new(head: ArrowHead) -> Self { Self { head } }
    pub fn none() -> Self { Self { head: ArrowHead::None } }
}

impl Default for ArrowDressing {
    fn default() -> Self { Self { head: ArrowHead::Normal } }
}

/// Complete arrow configuration. Java: `skin.ArrowConfiguration`
#[derive(Debug, Clone)]
pub struct ArrowConfiguration {
    pub body: ArrowBody,
    pub dressing1: ArrowDressing,
    pub dressing2: ArrowDressing,
    pub decoration1: ArrowDecoration,
    pub decoration2: ArrowDecoration,
    pub part: ArrowPart,
    pub color: Option<String>,
    pub is_reversed: bool,
    pub inclination: f64,
}

/// Arrow endpoint decoration. Java: `skin.ArrowDecoration`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArrowDecoration {
    #[default]
    None,
    Circle,
}

impl ArrowConfiguration {
    pub fn with_direction_normal() -> Self {
        Self {
            body: ArrowBody::Normal,
            dressing1: ArrowDressing::none(),
            dressing2: ArrowDressing::default(),
            decoration1: ArrowDecoration::None,
            decoration2: ArrowDecoration::None,
            part: ArrowPart::Full,
            color: None,
            is_reversed: false,
            inclination: 0.0,
        }
    }

    pub fn with_direction_reverse() -> Self {
        Self {
            body: ArrowBody::Normal,
            dressing1: ArrowDressing::default(),
            dressing2: ArrowDressing::none(),
            decoration1: ArrowDecoration::None,
            decoration2: ArrowDecoration::None,
            part: ArrowPart::Full,
            color: None,
            is_reversed: true,
            inclination: 0.0,
        }
    }

    pub fn with_direction_self() -> Self {
        Self {
            body: ArrowBody::Normal,
            dressing1: ArrowDressing::none(),
            dressing2: ArrowDressing::default(),
            decoration1: ArrowDecoration::None,
            decoration2: ArrowDecoration::None,
            part: ArrowPart::Full,
            color: None,
            is_reversed: false,
            inclination: 0.0,
        }
    }

    pub fn is_dotted(&self) -> bool { self.body == ArrowBody::Dotted }
    pub fn is_hidden(&self) -> bool { self.body == ArrowBody::Hidden }

    pub fn with_dotted(mut self) -> Self {
        self.body = ArrowBody::Dotted; self
    }

    pub fn with_head1(mut self, head: ArrowHead) -> Self {
        self.dressing1 = ArrowDressing::new(head); self
    }

    pub fn with_head2(mut self, head: ArrowHead) -> Self {
        self.dressing2 = ArrowDressing::new(head); self
    }

    pub fn with_part(mut self, part: ArrowPart) -> Self {
        self.part = part; self
    }

    pub fn with_decoration1(mut self, d: ArrowDecoration) -> Self {
        self.decoration1 = d; self
    }

    pub fn with_decoration2(mut self, d: ArrowDecoration) -> Self {
        self.decoration2 = d; self
    }

    pub fn with_inclination(mut self, inc: f64) -> Self {
        self.inclination = inc; self
    }

    pub fn reversed(mut self) -> Self {
        std::mem::swap(&mut self.dressing1, &mut self.dressing2);
        std::mem::swap(&mut self.decoration1, &mut self.decoration2);
        self.is_reversed = !self.is_reversed;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_arrow() {
        let a = ArrowConfiguration::with_direction_normal();
        assert_eq!(a.body, ArrowBody::Normal);
        assert_eq!(a.dressing2.head, ArrowHead::Normal);
        assert_eq!(a.dressing1.head, ArrowHead::None);
    }

    #[test]
    fn dotted_arrow() {
        let a = ArrowConfiguration::with_direction_normal().with_dotted();
        assert!(a.is_dotted());
    }

    #[test]
    fn reversed_arrow() {
        let a = ArrowConfiguration::with_direction_normal().reversed();
        assert_eq!(a.dressing1.head, ArrowHead::Normal);
        assert_eq!(a.dressing2.head, ArrowHead::None);
    }
}
