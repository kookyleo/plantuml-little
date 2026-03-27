//! TIM expression engine — port of Java PlantUML's `tim` package.
//!
//! The TIM (Template/Include/Macro) engine powers PlantUML's preprocessor.
//! This module provides the Java-compatible type structure while delegating
//! actual preprocessing to `crate::preproc` which already implements the
//! full directive pipeline (6000+ lines).
//!
//! # Module structure
//!
//! - [`expression`] — `TValue`, `Token`, `TokenType`, `TokenOperator`,
//!   `Knowledge` trait (port of `net.sourceforge.plantuml.tim.expression`)
//! - [`builtin`] — Built-in `%func()` implementations
//!   (port of `net.sourceforge.plantuml.tim.builtin`)
//!
//! # Key types
//!
//! | Java class | Rust type |
//! |---|---|
//! | `expression.TValue` | [`expression::TValue`] |
//! | `expression.Token` | [`expression::Token`] |
//! | `expression.TokenType` | [`expression::TokenType`] |
//! | `expression.TokenOperator` | [`expression::TokenOperator`] |
//! | `expression.Knowledge` | [`expression::Knowledge`] trait |
//! | `TFunctionType` | [`TFunctionType`] |
//! | `TFunctionSignature` | [`TFunctionSignature`] |
//! | `TVariableScope` | [`TVariableScope`] |
//! | `TMemory` | [`TMemory`] trait |
//! | `TContext` | [`TContext`] |

pub mod builtin;
pub mod expression;

use std::collections::HashMap;

pub use expression::TValue;

// ---------------------------------------------------------------------------
// TFunctionType
// ---------------------------------------------------------------------------

/// The type of a TIM function.
///
/// Mirrors Java `net.sourceforge.plantuml.tim.TFunctionType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TFunctionType {
    /// A procedure (no return value, side-effects only).
    Procedure,
    /// A return function (`!function ... !return`).
    ReturnFunction,
    /// A legacy `!define` macro (single-line).
    LegacyDefine,
    /// A legacy `!definelong` macro (multi-line).
    LegacyDefineLong,
}

impl TFunctionType {
    /// Whether this is a legacy (`!define` / `!definelong`) type.
    pub fn is_legacy(&self) -> bool {
        matches!(self, Self::LegacyDefine | Self::LegacyDefineLong)
    }
}

// ---------------------------------------------------------------------------
// TFunctionSignature
// ---------------------------------------------------------------------------

/// A function signature: name + argument count.
///
/// Mirrors Java `net.sourceforge.plantuml.tim.TFunctionSignature`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TFunctionSignature {
    pub function_name: String,
    pub nb_arg: usize,
}

impl TFunctionSignature {
    pub fn new(function_name: impl Into<String>, nb_arg: usize) -> Self {
        Self {
            function_name: function_name.into(),
            nb_arg,
        }
    }

    /// Check if two signatures refer to the same function name.
    pub fn same_function_name_as(&self, other: &Self) -> bool {
        self.function_name == other.function_name
    }
}

impl std::fmt::Display for TFunctionSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.function_name, self.nb_arg)
    }
}

// ---------------------------------------------------------------------------
// TVariableScope
// ---------------------------------------------------------------------------

/// Variable scope: local to a function or global.
///
/// Mirrors Java `net.sourceforge.plantuml.tim.TVariableScope`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TVariableScope {
    Local,
    Global,
}

impl TVariableScope {
    /// Parse a scope from a string value (case-insensitive).
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "local" => Some(Self::Local),
            "global" => Some(Self::Global),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// TMemory
// ---------------------------------------------------------------------------

/// Variable memory for the TIM engine.
///
/// Mirrors Java `net.sourceforge.plantuml.tim.TMemory` interface.
pub trait TMemory {
    fn get_variable(&self, varname: &str) -> Option<&TValue>;
    fn put_variable(&mut self, varname: &str, value: TValue, scope: TVariableScope);
    fn remove_variable(&mut self, varname: &str);
    fn is_empty(&self) -> bool;
    fn variable_names(&self) -> Vec<String>;
}

/// Global memory implementation.
///
/// Mirrors Java `net.sourceforge.plantuml.tim.TMemoryGlobal`.
#[derive(Debug, Default)]
pub struct TMemoryGlobal {
    variables: HashMap<String, TValue>,
}

impl TMemoryGlobal {
    pub fn new() -> Self {
        Self::default()
    }
}

impl TMemory for TMemoryGlobal {
    fn get_variable(&self, varname: &str) -> Option<&TValue> {
        self.variables.get(varname)
    }

    fn put_variable(&mut self, varname: &str, value: TValue, _scope: TVariableScope) {
        self.variables.insert(varname.to_string(), value);
    }

    fn remove_variable(&mut self, varname: &str) {
        self.variables.remove(varname);
    }

    fn is_empty(&self) -> bool {
        self.variables.is_empty()
    }

    fn variable_names(&self) -> Vec<String> {
        self.variables.keys().cloned().collect()
    }
}

/// Local (function-scoped) memory, backed by a global memory.
///
/// Mirrors Java `net.sourceforge.plantuml.tim.TMemoryLocal`.
#[derive(Debug)]
pub struct TMemoryLocal {
    global: TMemoryGlobal,
    local_vars: HashMap<String, TValue>,
}

impl TMemoryLocal {
    pub fn new(global: TMemoryGlobal) -> Self {
        Self {
            global,
            local_vars: HashMap::new(),
        }
    }

    pub fn with_initial(global: TMemoryGlobal, initial: HashMap<String, TValue>) -> Self {
        Self {
            global,
            local_vars: initial,
        }
    }
}

impl TMemory for TMemoryLocal {
    fn get_variable(&self, varname: &str) -> Option<&TValue> {
        self.local_vars
            .get(varname)
            .or_else(|| self.global.get_variable(varname))
    }

    fn put_variable(&mut self, varname: &str, value: TValue, scope: TVariableScope) {
        match scope {
            TVariableScope::Local => {
                self.local_vars.insert(varname.to_string(), value);
            }
            TVariableScope::Global => {
                self.global.put_variable(varname, value, scope);
            }
        }
    }

    fn remove_variable(&mut self, varname: &str) {
        if self.local_vars.remove(varname).is_none() {
            self.global.remove_variable(varname);
        }
    }

    fn is_empty(&self) -> bool {
        self.local_vars.is_empty() && self.global.is_empty()
    }

    fn variable_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.global.variable_names();
        for k in self.local_vars.keys() {
            if !names.contains(k) {
                names.push(k.clone());
            }
        }
        names
    }
}

// ---------------------------------------------------------------------------
// TContext
// ---------------------------------------------------------------------------

/// TIM evaluation context.
///
/// Mirrors Java `net.sourceforge.plantuml.tim.TContext`.
/// Holds the function registry and delegates to `crate::preproc` for
/// the actual preprocessing pipeline.
pub struct TContext {
    /// Registered built-in functions.
    builtins: HashMap<String, expression::BuiltinFn>,
}

impl TContext {
    /// Create a new context with standard built-in functions.
    pub fn new() -> Self {
        Self {
            builtins: builtin::builtin_map(),
        }
    }

    /// Resolve a variable from memory, with JSON path support.
    pub fn get_variable(&self, memory: &dyn TMemory, name: &str) -> Option<TValue> {
        memory.get_variable(name).cloned()
    }

    /// Look up a built-in function by name.
    pub fn get_builtin(&self, name: &str) -> Option<&expression::BuiltinFn> {
        self.builtins.get(name)
    }

    /// Call a built-in function by name with arguments.
    pub fn call_builtin(&self, name: &str, args: &[TValue]) -> Result<TValue, String> {
        let func = self
            .builtins
            .get(name)
            .ok_or_else(|| format!("Unknown function: {}", name))?;
        func(args)
    }

    /// Get a `Knowledge` implementation backed by this context and memory.
    pub fn as_knowledge<'a>(&'a self, memory: &'a dyn TMemory) -> ContextKnowledge<'a> {
        ContextKnowledge {
            context: self,
            memory,
        }
    }
}

impl Default for TContext {
    fn default() -> Self {
        Self::new()
    }
}

/// A `Knowledge` implementation that combines `TContext` (functions) and
/// `TMemory` (variables).
pub struct ContextKnowledge<'a> {
    context: &'a TContext,
    memory: &'a dyn TMemory,
}

impl expression::Knowledge for ContextKnowledge<'_> {
    fn get_variable(&self, name: &str) -> Option<TValue> {
        self.memory.get_variable(name).cloned()
    }

    fn get_function(&self, name: &str, _nb_arg: usize) -> Option<expression::BuiltinFn> {
        self.context.builtins.get(name).copied()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn function_type_is_legacy() {
        assert!(!TFunctionType::Procedure.is_legacy());
        assert!(!TFunctionType::ReturnFunction.is_legacy());
        assert!(TFunctionType::LegacyDefine.is_legacy());
        assert!(TFunctionType::LegacyDefineLong.is_legacy());
    }

    #[test]
    fn function_signature_equality() {
        let a = TFunctionSignature::new("%strlen", 1);
        let b = TFunctionSignature::new("%strlen", 1);
        let c = TFunctionSignature::new("%strlen", 2);
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert!(a.same_function_name_as(&c));
    }

    #[test]
    fn function_signature_display() {
        let sig = TFunctionSignature::new("%strlen", 1);
        assert_eq!(sig.to_string(), "%strlen/1");
    }

    #[test]
    fn variable_scope_parse() {
        assert_eq!(TVariableScope::parse("local"), Some(TVariableScope::Local));
        assert_eq!(
            TVariableScope::parse("GLOBAL"),
            Some(TVariableScope::Global)
        );
        assert_eq!(TVariableScope::parse("Local"), Some(TVariableScope::Local));
        assert_eq!(TVariableScope::parse("unknown"), None);
    }

    #[test]
    fn memory_global_basic() {
        let mut mem = TMemoryGlobal::new();
        assert!(mem.is_empty());
        assert!(mem.get_variable("$x").is_none());

        mem.put_variable("$x", TValue::from_int(42), TVariableScope::Global);
        assert!(!mem.is_empty());
        assert_eq!(mem.get_variable("$x").unwrap().to_int(), 42);

        mem.remove_variable("$x");
        assert!(mem.is_empty());
    }

    #[test]
    fn memory_local_scoping() {
        let mut global = TMemoryGlobal::new();
        global.put_variable("$g", TValue::from_int(1), TVariableScope::Global);

        let mut local = TMemoryLocal::new(global);
        // Can see global
        assert_eq!(local.get_variable("$g").unwrap().to_int(), 1);

        // Local variable shadows nothing initially
        local.put_variable("$l", TValue::from_int(2), TVariableScope::Local);
        assert_eq!(local.get_variable("$l").unwrap().to_int(), 2);

        // Local overrides global for same name
        local.put_variable("$g", TValue::from_int(99), TVariableScope::Local);
        assert_eq!(local.get_variable("$g").unwrap().to_int(), 99);

        // Remove local => falls back to global
        local.remove_variable("$g");
        assert_eq!(local.get_variable("$g").unwrap().to_int(), 1);
    }

    #[test]
    fn memory_variable_names() {
        let mut global = TMemoryGlobal::new();
        global.put_variable("$a", TValue::from_int(1), TVariableScope::Global);
        global.put_variable("$b", TValue::from_int(2), TVariableScope::Global);

        let mut local = TMemoryLocal::new(global);
        local.put_variable("$c", TValue::from_int(3), TVariableScope::Local);
        local.put_variable("$a", TValue::from_int(10), TVariableScope::Local);

        let mut names = local.variable_names();
        names.sort();
        assert_eq!(names, vec!["$a", "$b", "$c"]);
    }

    #[test]
    fn context_new_has_builtins() {
        let ctx = TContext::new();
        assert!(ctx.get_builtin("%strlen").is_some());
        assert!(ctx.get_builtin("%upper").is_some());
        assert!(ctx.get_builtin("%nonexistent").is_none());
    }

    #[test]
    fn context_call_builtin() {
        let ctx = TContext::new();

        let result = ctx
            .call_builtin("%strlen", &[TValue::from_string("hello")])
            .unwrap();
        assert_eq!(result.to_int(), 5);

        let result = ctx
            .call_builtin("%upper", &[TValue::from_string("hello")])
            .unwrap();
        assert_eq!(result.to_string(), "HELLO");

        assert!(ctx.call_builtin("%unknown_func", &[]).is_err());
    }

    #[test]
    fn context_as_knowledge() {
        let ctx = TContext::new();
        let mut mem = TMemoryGlobal::new();
        mem.put_variable("$x", TValue::from_int(42), TVariableScope::Global);

        let knowledge = ctx.as_knowledge(&mem);

        use expression::Knowledge;
        assert_eq!(knowledge.get_variable("$x").unwrap().to_int(), 42);
        assert!(knowledge.get_variable("$y").is_none());
        assert!(knowledge.get_function("%strlen", 1).is_some());
    }

    #[test]
    fn context_default() {
        let ctx = TContext::default();
        assert!(ctx.get_builtin("%strlen").is_some());
    }

    #[test]
    fn memory_put_global_via_local() {
        let global = TMemoryGlobal::new();
        let mut local = TMemoryLocal::new(global);

        // Put a variable with Global scope through local memory
        local.put_variable("$g", TValue::from_int(7), TVariableScope::Global);

        // Should be accessible
        assert_eq!(local.get_variable("$g").unwrap().to_int(), 7);
    }
}
