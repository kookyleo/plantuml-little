//! TIM expression value types and operator evaluation.
//!
//! Port of Java PlantUML's `net.sourceforge.plantuml.tim.expression` package.
//! Provides `TValue` (the core value type: integer, string, or JSON),
//! `TokenType`, `TokenOperator`, and `Knowledge` trait.
//!
//! For arithmetic expression parsing, delegates to `crate::preproc::expr`.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// TValue — the central value type for TIM expressions
// ---------------------------------------------------------------------------

/// A value in the TIM expression engine: integer, string, or JSON.
///
/// Mirrors Java `net.sourceforge.plantuml.tim.expression.TValue`.
/// JSON values are stored as `serde_json::Value` for interop convenience.
#[derive(Clone, Debug)]
pub enum TValue {
    Int(i64),
    Str(String),
    Json(serde_json::Value),
}

impl TValue {
    // -- constructors -------------------------------------------------------

    pub fn from_int(v: i64) -> Self {
        TValue::Int(v)
    }

    pub fn from_bool(b: bool) -> Self {
        TValue::Int(if b { 1 } else { 0 })
    }

    pub fn from_string(s: impl Into<String>) -> Self {
        TValue::Str(s.into())
    }

    pub fn from_json(v: serde_json::Value) -> Self {
        TValue::Json(v)
    }

    // -- type queries -------------------------------------------------------

    pub fn is_number(&self) -> bool {
        matches!(self, TValue::Int(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, TValue::Str(_))
    }

    pub fn is_json(&self) -> bool {
        matches!(self, TValue::Json(_))
    }

    // -- conversions --------------------------------------------------------

    /// Integer value (0 for non-integer types).
    pub fn to_int(&self) -> i64 {
        match self {
            TValue::Int(n) => *n,
            TValue::Str(s) => s.parse::<i64>().unwrap_or(0),
            TValue::Json(v) => v.as_i64().unwrap_or_default(),
        }
    }

    /// Boolean coercion: 0 / empty string / null are falsy.
    pub fn to_bool(&self) -> bool {
        match self {
            TValue::Int(n) => *n != 0,
            TValue::Str(s) => !s.is_empty(),
            TValue::Json(v) => !v.is_null(),
        }
    }

    /// Get a reference to the JSON value (if this is a JSON variant).
    pub fn to_json(&self) -> Option<&serde_json::Value> {
        match self {
            TValue::Json(v) => Some(v),
            _ => None,
        }
    }

    /// Convert any variant to a `serde_json::Value`.
    pub fn to_json_value(&self) -> serde_json::Value {
        match self {
            TValue::Int(n) => serde_json::Value::Number((*n).into()),
            TValue::Str(s) => serde_json::Value::String(s.clone()),
            TValue::Json(v) => v.clone(),
        }
    }

    // -- arithmetic ---------------------------------------------------------

    pub fn add(&self, other: &TValue) -> TValue {
        if self.is_number() && other.is_number() {
            TValue::Int(self.to_int() + other.to_int())
        } else {
            TValue::Str(format!("{}{}", self, other))
        }
    }

    pub fn minus(&self, other: &TValue) -> TValue {
        if self.is_number() && other.is_number() {
            TValue::Int(self.to_int() - other.to_int())
        } else {
            TValue::Str(format!("{}{}", self, other))
        }
    }

    pub fn multiply(&self, other: &TValue) -> TValue {
        if self.is_number() && other.is_number() {
            TValue::Int(self.to_int() * other.to_int())
        } else {
            TValue::Str(format!("{}*{}", self, other))
        }
    }

    pub fn divided_by(&self, other: &TValue) -> TValue {
        if self.is_number() && other.is_number() {
            let d = other.to_int();
            if d == 0 {
                TValue::Int(0) // avoid panic; Java would throw ArithmeticException
            } else {
                TValue::Int(self.to_int() / d)
            }
        } else {
            TValue::Str(format!("{}/{}", self, other))
        }
    }

    // -- comparison ---------------------------------------------------------

    pub fn less_than(&self, other: &TValue) -> TValue {
        TValue::from_bool(self.cmp_values(other) == Ordering::Less)
    }

    pub fn less_than_or_equals(&self, other: &TValue) -> TValue {
        TValue::from_bool(self.cmp_values(other) != Ordering::Greater)
    }

    pub fn greater_than(&self, other: &TValue) -> TValue {
        TValue::from_bool(self.cmp_values(other) == Ordering::Greater)
    }

    pub fn greater_than_or_equals(&self, other: &TValue) -> TValue {
        TValue::from_bool(self.cmp_values(other) != Ordering::Less)
    }

    pub fn equals_op(&self, other: &TValue) -> TValue {
        TValue::from_bool(self.cmp_values(other) == Ordering::Equal)
    }

    pub fn not_equals(&self, other: &TValue) -> TValue {
        TValue::from_bool(self.cmp_values(other) != Ordering::Equal)
    }

    // -- logical ------------------------------------------------------------

    pub fn logical_and(&self, other: &TValue) -> TValue {
        TValue::from_bool(self.to_bool() && other.to_bool())
    }

    pub fn logical_or(&self, other: &TValue) -> TValue {
        TValue::from_bool(self.to_bool() || other.to_bool())
    }

    // -- internal -----------------------------------------------------------

    fn cmp_values(&self, other: &TValue) -> Ordering {
        if self.is_number() && other.is_number() {
            self.to_int().cmp(&other.to_int())
        } else {
            self.to_string().cmp(&other.to_string())
        }
    }
}

impl fmt::Display for TValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TValue::Int(n) => write!(f, "{}", n),
            TValue::Str(s) => write!(f, "{}", s),
            TValue::Json(v) => {
                if let Some(s) = v.as_str() {
                    write!(f, "{}", s)
                } else {
                    write!(f, "{}", v)
                }
            }
        }
    }
}

impl PartialEq for TValue {
    fn eq(&self, other: &Self) -> bool {
        self.cmp_values(other) == Ordering::Equal
    }
}

impl Eq for TValue {}

// ---------------------------------------------------------------------------
// TokenType — expression token classification
// ---------------------------------------------------------------------------

/// Token types used during expression tokenisation.
///
/// Mirrors Java `net.sourceforge.plantuml.tim.expression.TokenType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    QuotedString,
    JsonData,
    Operator,
    OpenParenMath,
    Comma,
    CloseParenMath,
    Number,
    PlainText,
    Spaces,
    FunctionName,
    OpenParenFunc,
    CloseParenFunc,
    Affectation,
}

/// Special minus sign used internally to distinguish subtraction from negation.
/// Mirrors Java `TokenType.COMMERCIAL_MINUS_SIGN` (U+2052).
pub const COMMERCIAL_MINUS_SIGN: char = '\u{2052}';

// ---------------------------------------------------------------------------
// TokenOperator — binary operators with precedence
// ---------------------------------------------------------------------------

/// Binary operators for TIM expressions, ordered by precedence.
///
/// Precedence values follow C operator precedence (higher = tighter binding).
/// Mirrors Java `net.sourceforge.plantuml.tim.expression.TokenOperator`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenOperator {
    Multiplication,
    Division,
    Addition,
    Subtraction,
    LessThan,
    GreaterThan,
    LessThanOrEquals,
    GreaterThanOrEquals,
    Equals,
    NotEquals,
    LogicalAnd,
    LogicalOr,
}

impl TokenOperator {
    /// Operator precedence (higher = binds tighter).
    pub fn precedence(&self) -> i32 {
        match self {
            Self::Multiplication | Self::Division => 97,
            Self::Addition | Self::Subtraction => 96,
            Self::LessThan | Self::GreaterThan
            | Self::LessThanOrEquals | Self::GreaterThanOrEquals => 94,
            Self::Equals | Self::NotEquals => 93,
            Self::LogicalAnd => 89,
            Self::LogicalOr => 88,
        }
    }

    /// Display string for this operator.
    pub fn display(&self) -> &'static str {
        match self {
            Self::Multiplication => "*",
            Self::Division => "/",
            Self::Addition => "+",
            Self::Subtraction => "\u{2052}", // COMMERCIAL_MINUS_SIGN
            Self::LessThan => "<",
            Self::GreaterThan => ">",
            Self::LessThanOrEquals => "<=",
            Self::GreaterThanOrEquals => ">=",
            Self::Equals => "==",
            Self::NotEquals => "!=",
            Self::LogicalAnd => "&&",
            Self::LogicalOr => "||",
        }
    }

    /// Apply this operator to two values.
    pub fn operate(&self, v1: &TValue, v2: &TValue) -> TValue {
        match self {
            Self::Multiplication => v1.multiply(v2),
            Self::Division => v1.divided_by(v2),
            Self::Addition => v1.add(v2),
            Self::Subtraction => v1.minus(v2),
            Self::LessThan => v1.less_than(v2),
            Self::GreaterThan => v1.greater_than(v2),
            Self::LessThanOrEquals => v1.less_than_or_equals(v2),
            Self::GreaterThanOrEquals => v1.greater_than_or_equals(v2),
            Self::Equals => v1.equals_op(v2),
            Self::NotEquals => v1.not_equals(v2),
            Self::LogicalAnd => v1.logical_and(v2),
            Self::LogicalOr => v1.logical_or(v2),
        }
    }

    /// Look up a token operator from one or two characters.
    /// Returns `None` if the character(s) don't form a known operator.
    pub fn from_chars(ch: char, ch2: char) -> Option<Self> {
        match ch {
            '*' => Some(Self::Multiplication),
            '/' => Some(Self::Division),
            '+' => Some(Self::Addition),
            c if c == COMMERCIAL_MINUS_SIGN => Some(Self::Subtraction),
            '<' => {
                if ch2 == '=' {
                    Some(Self::LessThanOrEquals)
                } else {
                    Some(Self::LessThan)
                }
            }
            '>' => {
                if ch2 == '=' {
                    Some(Self::GreaterThanOrEquals)
                } else {
                    Some(Self::GreaterThan)
                }
            }
            '=' if ch2 == '=' => Some(Self::Equals),
            '!' if ch2 == '=' => Some(Self::NotEquals),
            '&' if ch2 == '&' => Some(Self::LogicalAnd),
            '|' if ch2 == '|' => Some(Self::LogicalOr),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Token — a single token in an expression
// ---------------------------------------------------------------------------

/// A single token produced by expression tokenisation.
///
/// Mirrors Java `net.sourceforge.plantuml.tim.expression.Token`.
#[derive(Debug, Clone)]
pub struct Token {
    pub surface: String,
    pub token_type: TokenType,
    pub json: Option<serde_json::Value>,
}

impl Token {
    pub fn new(surface: impl Into<String>, token_type: TokenType) -> Self {
        Self {
            surface: surface.into(),
            token_type,
            json: None,
        }
    }

    pub fn with_json(surface: impl Into<String>, token_type: TokenType, json: serde_json::Value) -> Self {
        Self {
            surface: surface.into(),
            token_type,
            json: Some(json),
        }
    }

    /// Convert this token to a `TValue`.
    pub fn to_tvalue(&self) -> Option<TValue> {
        match self.token_type {
            TokenType::Number => {
                let n = self.surface.parse::<i64>().ok()?;
                Some(TValue::from_int(n))
            }
            TokenType::QuotedString => Some(TValue::from_string(&self.surface)),
            TokenType::JsonData => {
                let json = self.json.clone()?;
                Some(TValue::from_json(json))
            }
            _ => None,
        }
    }

    /// Get the operator for an OPERATOR token.
    pub fn get_operator(&self) -> Option<TokenOperator> {
        if self.token_type != TokenType::Operator {
            return None;
        }
        let mut chars = self.surface.chars();
        let ch1 = chars.next().unwrap_or('\0');
        let ch2 = chars.next().unwrap_or('\0');
        TokenOperator::from_chars(ch1, ch2)
    }

    /// Promote a PLAIN_TEXT token to FUNCTION_NAME.
    pub fn mute_to_function(mut self) -> Self {
        debug_assert_eq!(self.token_type, TokenType::PlainText);
        self.token_type = TokenType::FunctionName;
        self
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}{{{}}}", self.token_type, self.surface)
    }
}

// ---------------------------------------------------------------------------
// Knowledge — interface for variable/function resolution during evaluation
// ---------------------------------------------------------------------------

/// Trait for resolving variables and functions during expression evaluation.
///
/// Mirrors Java `net.sourceforge.plantuml.tim.expression.Knowledge`.
pub trait Knowledge {
    /// Look up a variable by name. Returns `None` if undefined.
    fn get_variable(&self, name: &str) -> Option<TValue>;

    /// Look up a function by signature. Returns `None` if not found.
    fn get_function(&self, name: &str, nb_arg: usize) -> Option<BuiltinFn>;
}

/// A builtin function pointer: takes argument values, returns a result.
pub type BuiltinFn = fn(&[TValue]) -> Result<TValue, String>;

/// A no-op knowledge implementation (knows nothing).
#[derive(Debug, Default)]
pub struct EmptyKnowledge;

impl Knowledge for EmptyKnowledge {
    fn get_variable(&self, _name: &str) -> Option<TValue> {
        None
    }
    fn get_function(&self, _name: &str, _nb_arg: usize) -> Option<BuiltinFn> {
        None
    }
}

/// A simple knowledge implementation backed by a variable map and function registry.
pub struct SimpleKnowledge {
    pub variables: HashMap<String, TValue>,
    pub functions: HashMap<String, BuiltinFn>,
}

impl SimpleKnowledge {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }
}

impl Default for SimpleKnowledge {
    fn default() -> Self {
        Self::new()
    }
}

impl Knowledge for SimpleKnowledge {
    fn get_variable(&self, name: &str) -> Option<TValue> {
        self.variables.get(name).cloned()
    }

    fn get_function(&self, name: &str, _nb_arg: usize) -> Option<BuiltinFn> {
        self.functions.get(name).copied()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tvalue_int_arithmetic() {
        let a = TValue::from_int(10);
        let b = TValue::from_int(3);
        assert_eq!(a.add(&b).to_int(), 13);
        assert_eq!(a.minus(&b).to_int(), 7);
        assert_eq!(a.multiply(&b).to_int(), 30);
        assert_eq!(a.divided_by(&b).to_int(), 3);
    }

    #[test]
    fn tvalue_string_concat() {
        let a = TValue::from_string("hello");
        let b = TValue::from_string(" world");
        assert_eq!(a.add(&b).to_string(), "hello world");
    }

    #[test]
    fn tvalue_comparisons() {
        let a = TValue::from_int(5);
        let b = TValue::from_int(10);

        assert!(a.less_than(&b).to_bool());
        assert!(!a.greater_than(&b).to_bool());
        assert!(a.less_than_or_equals(&b).to_bool());
        assert!(!b.less_than(&a).to_bool());

        let c = TValue::from_int(5);
        assert!(a.equals_op(&c).to_bool());
        assert!(!a.not_equals(&c).to_bool());
        assert!(a.not_equals(&b).to_bool());
    }

    #[test]
    fn tvalue_string_comparisons() {
        let a = TValue::from_string("abc");
        let b = TValue::from_string("def");
        assert!(a.less_than(&b).to_bool());
        assert!(!a.greater_than(&b).to_bool());

        let c = TValue::from_string("abc");
        assert!(a.equals_op(&c).to_bool());
    }

    #[test]
    fn tvalue_logical_ops() {
        let t = TValue::from_int(1);
        let f = TValue::from_int(0);

        assert!(t.logical_and(&t).to_bool());
        assert!(!t.logical_and(&f).to_bool());
        assert!(t.logical_or(&f).to_bool());
        assert!(!f.logical_or(&f).to_bool());
    }

    #[test]
    fn tvalue_bool_coercion() {
        assert!(TValue::from_int(1).to_bool());
        assert!(TValue::from_int(-1).to_bool());
        assert!(!TValue::from_int(0).to_bool());
        assert!(TValue::from_string("x").to_bool());
        assert!(!TValue::from_string("").to_bool());
    }

    #[test]
    fn tvalue_display() {
        assert_eq!(TValue::from_int(42).to_string(), "42");
        assert_eq!(TValue::from_string("hello").to_string(), "hello");
        assert_eq!(
            TValue::from_json(serde_json::json!("test")).to_string(),
            "test"
        );
        assert_eq!(
            TValue::from_json(serde_json::json!({"a": 1})).to_string(),
            "{\"a\":1}"
        );
    }

    #[test]
    fn tvalue_from_bool() {
        assert_eq!(TValue::from_bool(true).to_int(), 1);
        assert_eq!(TValue::from_bool(false).to_int(), 0);
    }

    #[test]
    fn tvalue_division_by_zero() {
        let a = TValue::from_int(10);
        let b = TValue::from_int(0);
        // Should not panic
        assert_eq!(a.divided_by(&b).to_int(), 0);
    }

    #[test]
    fn tvalue_mixed_add() {
        // int + string => string concatenation
        let a = TValue::from_int(42);
        let b = TValue::from_string(" items");
        assert_eq!(a.add(&b).to_string(), "42 items");
    }

    #[test]
    fn token_operator_precedence() {
        assert!(TokenOperator::Multiplication.precedence() > TokenOperator::Addition.precedence());
        assert!(TokenOperator::Addition.precedence() > TokenOperator::LessThan.precedence());
        assert!(TokenOperator::Equals.precedence() > TokenOperator::LogicalAnd.precedence());
        assert!(TokenOperator::LogicalAnd.precedence() > TokenOperator::LogicalOr.precedence());
    }

    #[test]
    fn token_operator_from_chars() {
        assert_eq!(
            TokenOperator::from_chars('*', '\0'),
            Some(TokenOperator::Multiplication)
        );
        assert_eq!(
            TokenOperator::from_chars('<', '='),
            Some(TokenOperator::LessThanOrEquals)
        );
        assert_eq!(
            TokenOperator::from_chars('<', ' '),
            Some(TokenOperator::LessThan)
        );
        assert_eq!(
            TokenOperator::from_chars('=', '='),
            Some(TokenOperator::Equals)
        );
        assert_eq!(
            TokenOperator::from_chars('=', ' '),
            None
        );
        assert_eq!(
            TokenOperator::from_chars('&', '&'),
            Some(TokenOperator::LogicalAnd)
        );
        assert_eq!(
            TokenOperator::from_chars('&', ' '),
            None
        );
    }

    #[test]
    fn token_operator_operate() {
        let a = TValue::from_int(6);
        let b = TValue::from_int(3);
        assert_eq!(TokenOperator::Multiplication.operate(&a, &b).to_int(), 18);
        assert_eq!(TokenOperator::Division.operate(&a, &b).to_int(), 2);
        assert_eq!(TokenOperator::Addition.operate(&a, &b).to_int(), 9);
        assert_eq!(TokenOperator::Subtraction.operate(&a, &b).to_int(), 3);
    }

    #[test]
    fn token_to_tvalue() {
        let t = Token::new("42", TokenType::Number);
        assert_eq!(t.to_tvalue().unwrap().to_int(), 42);

        let t = Token::new("hello", TokenType::QuotedString);
        assert_eq!(t.to_tvalue().unwrap().to_string(), "hello");
    }

    #[test]
    fn token_get_operator() {
        let t = Token::new("+", TokenType::Operator);
        assert_eq!(t.get_operator(), Some(TokenOperator::Addition));

        let t = Token::new("<=", TokenType::Operator);
        assert_eq!(t.get_operator(), Some(TokenOperator::LessThanOrEquals));

        let t = Token::new("42", TokenType::Number);
        assert_eq!(t.get_operator(), None);
    }

    #[test]
    fn empty_knowledge() {
        let k = EmptyKnowledge;
        assert!(k.get_variable("x").is_none());
        assert!(k.get_function("%strlen", 1).is_none());
    }

    #[test]
    fn simple_knowledge() {
        let mut k = SimpleKnowledge::new();
        k.variables
            .insert("$x".to_string(), TValue::from_int(42));
        assert_eq!(k.get_variable("$x").unwrap().to_int(), 42);
        assert!(k.get_variable("$y").is_none());
    }

    #[test]
    fn tvalue_json_conversion() {
        let v = TValue::from_int(42);
        assert_eq!(v.to_json_value(), serde_json::json!(42));

        let v = TValue::from_string("hello");
        assert_eq!(v.to_json_value(), serde_json::json!("hello"));

        let json = serde_json::json!({"key": "val"});
        let v = TValue::from_json(json.clone());
        assert_eq!(v.to_json_value(), json);
    }
}
