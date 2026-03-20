// Port of Java tim-package skeleton tests to Rust.
// Source: generated-tests/.../tim/
//
// Mapping notes:
//   Java TFunctionType           → tim::TFunctionType
//   Java TFunctionSignature      → tim::TFunctionSignature
//   Java TVariableScope          → tim::TVariableScope
//   Java TMemory (interface)     → tim::TMemory trait
//   Java TMemoryGlobal           → tim::TMemoryGlobal
//   Java TMemoryLocal            → tim::TMemoryLocal
//   Java TContext                → tim::TContext
//   Java TValue                  → tim::TValue  (re-export of expression::TValue)
//   Java TokenType               → tim::expression::TokenType
//   Java TokenOperator           → tim::expression::TokenOperator
//   Java Token                   → tim::expression::Token
//   Java Knowledge (interface)   → tim::expression::Knowledge trait
//   Java built-in functions      → tim::builtin::{standard_builtins, builtin_map}

// ═══════════════════════════════════════════════════════════════════════════
// TFunctionTypeSkeletonTest.java
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tfunction_type_tests {
	use plantuml_little::tim::TFunctionType;

	#[test]
	fn values_4_variants() {
		// Java: assertEquals(4, TFunctionType.values().length)
		let variants = [
			TFunctionType::Procedure,
			TFunctionType::ReturnFunction,
			TFunctionType::LegacyDefine,
			TFunctionType::LegacyDefineLong,
		];
		assert_eq!(variants.len(), 4);
		// All distinct
		for (i, a) in variants.iter().enumerate() {
			for (j, b) in variants.iter().enumerate() {
				if i == j {
					assert_eq!(a, b);
				} else {
					assert_ne!(a, b);
				}
			}
		}
	}

	#[test]
	fn value_of_all_names() {
		// Java: assertSame(TFunctionType.PROCEDURE, TFunctionType.valueOf("PROCEDURE")) etc.
		assert_eq!(format!("{:?}", TFunctionType::Procedure), "Procedure");
		assert_eq!(format!("{:?}", TFunctionType::ReturnFunction), "ReturnFunction");
		assert_eq!(format!("{:?}", TFunctionType::LegacyDefine), "LegacyDefine");
		assert_eq!(format!("{:?}", TFunctionType::LegacyDefineLong), "LegacyDefineLong");
		// Cross-inequality: all variants are distinct
		assert_ne!(TFunctionType::Procedure, TFunctionType::ReturnFunction);
		assert_ne!(TFunctionType::LegacyDefine, TFunctionType::LegacyDefineLong);
		assert_ne!(TFunctionType::Procedure, TFunctionType::LegacyDefine);
	}

	#[test]
	fn is_legacy_procedure_false() {
		// Java: assertFalse(PROCEDURE.isLegacy())
		assert!(TFunctionType::Procedure.is_legacy() == false);
	}

	#[test]
	fn is_legacy_return_function_false() {
		// Java: assertFalse(RETURN_FUNCTION.isLegacy())
		assert!(TFunctionType::ReturnFunction.is_legacy() == false);
	}

	#[test]
	fn is_legacy_legacy_define_true() {
		// Java: assertTrue(LEGACY_DEFINE.isLegacy())
		assert!(TFunctionType::LegacyDefine.is_legacy());
	}

	#[test]
	fn is_legacy_legacy_define_long_true() {
		// Java: assertTrue(LEGACY_DEFINE_LONG.isLegacy())
		assert!(TFunctionType::LegacyDefineLong.is_legacy());
	}

	#[test]
	fn is_legacy_only_two_of_four_return_true() {
		// Java: exactly LEGACY_DEFINE and LEGACY_DEFINE_LONG are legacy
		let all = [
			TFunctionType::Procedure,
			TFunctionType::ReturnFunction,
			TFunctionType::LegacyDefine,
			TFunctionType::LegacyDefineLong,
		];
		let legacy_count = all.iter().filter(|t| t.is_legacy()).count();
		let non_legacy_count = all.iter().filter(|t| !t.is_legacy()).count();
		assert_eq!(legacy_count, 2);
		assert_eq!(non_legacy_count, 2);
	}

	#[test]
	fn clone_preserves_is_legacy() {
		// Clone-derived: cloning should preserve legacy status
		let orig = TFunctionType::LegacyDefine;
		let cloned = orig;
		assert_eq!(orig.is_legacy(), cloned.is_legacy());
		assert_eq!(orig, cloned);
	}
}

// ═══════════════════════════════════════════════════════════════════════════
// TFunctionSignatureSkeletonTest.java
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tfunction_signature_tests {
	use plantuml_little::tim::TFunctionSignature;

	#[test]
	fn new_sets_name_and_arg_count() {
		// Java: new TFunctionSignature("foo", 2); getFunctionName()=="foo"; getNbArg()==2
		let sig = TFunctionSignature::new("foo", 2);
		assert_eq!(sig.function_name, "foo");
		assert_eq!(sig.nb_arg, 2);
	}

	#[test]
	fn new_zero_args() {
		// Java: new TFunctionSignature("bar", 0); getNbArg()==0
		let sig = TFunctionSignature::new("bar", 0);
		assert_eq!(sig.function_name, "bar");
		assert_eq!(sig.nb_arg, 0);
	}

	#[test]
	fn display_format_name_slash_nb_arg() {
		// Java: toString() → "name/nbArg"
		let sig = TFunctionSignature::new("%strlen", 1);
		assert_eq!(sig.to_string(), "%strlen/1");
	}

	#[test]
	fn display_format_zero_args() {
		let sig = TFunctionSignature::new("%true", 0);
		assert_eq!(sig.to_string(), "%true/0");
	}

	#[test]
	fn display_format_multi_args() {
		let sig = TFunctionSignature::new("%substr", 3);
		assert_eq!(sig.to_string(), "%substr/3");
	}

	#[test]
	fn equality_same_name_and_count() {
		// Java: assertEquals(sig1, sig2) when both have same name and nb_arg
		let a = TFunctionSignature::new("%strlen", 1);
		let b = TFunctionSignature::new("%strlen", 1);
		assert_eq!(a, b);
	}

	#[test]
	fn inequality_different_arg_count() {
		// Java: assertNotEquals(sig1, sig2) when nb_arg differs
		let a = TFunctionSignature::new("%strlen", 1);
		let c = TFunctionSignature::new("%strlen", 2);
		assert_ne!(a, c);
	}

	#[test]
	fn inequality_different_name() {
		// Java: assertNotEquals(sig1, sig2) when name differs
		let a = TFunctionSignature::new("%strlen", 1);
		let b = TFunctionSignature::new("%upper", 1);
		assert_ne!(a, b);
	}

	#[test]
	fn same_function_name_as_ignores_arg_count() {
		// Java: sig1.sameFunctionNameAs(sig2) when names match regardless of nb_arg
		let a = TFunctionSignature::new("%strlen", 1);
		let b = TFunctionSignature::new("%strlen", 2);
		assert!(a.same_function_name_as(&b));
	}

	#[test]
	fn same_function_name_as_false_different_name() {
		// Java: sig1.sameFunctionNameAs(sig2) == false when names differ
		let a = TFunctionSignature::new("%strlen", 1);
		let b = TFunctionSignature::new("%upper", 1);
		assert!(a.same_function_name_as(&b) == false);
	}

	#[test]
	fn same_function_name_as_reflexive() {
		// Java: sig.sameFunctionNameAs(sig) == true
		let a = TFunctionSignature::new("%modulo", 2);
		assert!(a.same_function_name_as(&a));
	}

	#[test]
	fn hash_equal_signatures_have_equal_hash() {
		use std::collections::HashMap;
		// Java: HashSet<TFunctionSignature> — equal sigs dedup correctly
		let a = TFunctionSignature::new("%strlen", 1);
		let b = TFunctionSignature::new("%strlen", 1);
		let mut map = HashMap::new();
		map.insert(a, "first");
		map.insert(b, "second");
		assert_eq!(map.len(), 1);
	}

	#[test]
	fn clone_produces_equal_signature() {
		let sig = TFunctionSignature::new("%substr", 3);
		let cloned = sig.clone();
		assert_eq!(sig, cloned);
		assert_eq!(sig.function_name, cloned.function_name);
		assert_eq!(sig.nb_arg, cloned.nb_arg);
	}
}

// ═══════════════════════════════════════════════════════════════════════════
// TVariableScopeSkeletonTest.java
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tvariable_scope_tests {
	use plantuml_little::tim::TVariableScope;

	#[test]
	fn values_2_variants() {
		// Java: assertEquals(2, TVariableScope.values().length)
		let variants = [TVariableScope::Local, TVariableScope::Global];
		assert_eq!(variants.len(), 2);
		assert_ne!(variants[0], variants[1]);
	}

	#[test]
	fn value_of_all_names() {
		// Java: assertSame(TVariableScope.LOCAL, TVariableScope.valueOf("LOCAL")) etc.
		assert_eq!(format!("{:?}", TVariableScope::Local), "Local");
		assert_eq!(format!("{:?}", TVariableScope::Global), "Global");
		assert_ne!(TVariableScope::Local, TVariableScope::Global);
	}

	#[test]
	fn parse_local_lowercase() {
		// Java: TVariableScope.fromString("local") == LOCAL
		assert_eq!(TVariableScope::parse("local"), Some(TVariableScope::Local));
	}

	#[test]
	fn parse_global_lowercase() {
		// Java: TVariableScope.fromString("global") == GLOBAL
		assert_eq!(TVariableScope::parse("global"), Some(TVariableScope::Global));
	}

	#[test]
	fn parse_local_uppercase() {
		// Java: case-insensitive matching
		assert_eq!(TVariableScope::parse("LOCAL"), Some(TVariableScope::Local));
	}

	#[test]
	fn parse_global_uppercase() {
		assert_eq!(TVariableScope::parse("GLOBAL"), Some(TVariableScope::Global));
	}

	#[test]
	fn parse_local_mixed_case() {
		assert_eq!(TVariableScope::parse("Local"), Some(TVariableScope::Local));
	}

	#[test]
	fn parse_global_mixed_case() {
		assert_eq!(TVariableScope::parse("Global"), Some(TVariableScope::Global));
	}

	#[test]
	fn parse_unknown_returns_none() {
		// Java: fromString("unknown") → null
		assert_eq!(TVariableScope::parse("unknown"), None);
	}

	#[test]
	fn parse_empty_string_returns_none() {
		assert_eq!(TVariableScope::parse(""), None);
	}

	#[test]
	fn clone_and_copy_work() {
		let s = TVariableScope::Local;
		let s2 = s; // Copy
		assert_eq!(s, s2);
		let s3 = TVariableScope::Global;
		assert_ne!(s, s3);
	}
}

// ═══════════════════════════════════════════════════════════════════════════
// TMemoryGlobalSkeletonTest.java
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tmemory_global_tests {
	use plantuml_little::tim::{TMemory, TMemoryGlobal, TValue, TVariableScope};

	#[test]
	fn new_is_empty() {
		// Java: new TMemoryGlobal(); assertTrue(isEmpty())
		let mem = TMemoryGlobal::new();
		assert!(mem.is_empty());
	}

	#[test]
	fn get_variable_unknown_is_none() {
		// Java: assertNull(mem.getVariable("$x"))
		let mem = TMemoryGlobal::new();
		assert!(mem.get_variable("$x").is_none());
	}

	#[test]
	fn put_and_get_integer_variable() {
		// Java: putVariable("$x", TValue.fromInt(42)); assertEquals(42, getVariable("$x").toInt())
		let mut mem = TMemoryGlobal::new();
		mem.put_variable("$x", TValue::from_int(42), TVariableScope::Global);
		let val = mem.get_variable("$x").unwrap();
		assert_eq!(val.to_int(), 42);
	}

	#[test]
	fn put_and_get_string_variable() {
		let mut mem = TMemoryGlobal::new();
		mem.put_variable("$name", TValue::from_string("hello"), TVariableScope::Global);
		let val = mem.get_variable("$name").unwrap();
		assert_eq!(val.to_string(), "hello");
	}

	#[test]
	fn put_overwrites_existing_value() {
		// Java: put "x"=1, then "x"=99; getVariable("x") == 99
		let mut mem = TMemoryGlobal::new();
		mem.put_variable("$x", TValue::from_int(1), TVariableScope::Global);
		mem.put_variable("$x", TValue::from_int(99), TVariableScope::Global);
		assert_eq!(mem.get_variable("$x").unwrap().to_int(), 99);
	}

	#[test]
	fn is_empty_false_after_put() {
		// Java: assertFalse(isEmpty()) after putVariable
		let mut mem = TMemoryGlobal::new();
		mem.put_variable("$a", TValue::from_int(1), TVariableScope::Global);
		assert!(!mem.is_empty());
	}

	#[test]
	fn remove_existing_variable() {
		// Java: removeVariable("$x"); assertNull(getVariable("$x"))
		let mut mem = TMemoryGlobal::new();
		mem.put_variable("$x", TValue::from_int(42), TVariableScope::Global);
		mem.remove_variable("$x");
		assert!(mem.get_variable("$x").is_none());
	}

	#[test]
	fn remove_variable_makes_empty() {
		let mut mem = TMemoryGlobal::new();
		mem.put_variable("$x", TValue::from_int(1), TVariableScope::Global);
		mem.remove_variable("$x");
		assert!(mem.is_empty());
	}

	#[test]
	fn remove_nonexistent_is_noop() {
		// Java: removeVariable on unknown name does not throw
		let mut mem = TMemoryGlobal::new();
		mem.remove_variable("$nonexistent"); // must not panic
		assert!(mem.is_empty());
	}

	#[test]
	fn variable_names_returns_all_keys() {
		// Java: variableNames() returns all stored variable names
		let mut mem = TMemoryGlobal::new();
		mem.put_variable("$a", TValue::from_int(1), TVariableScope::Global);
		mem.put_variable("$b", TValue::from_int(2), TVariableScope::Global);
		let mut names = mem.variable_names();
		names.sort();
		assert_eq!(names, vec!["$a", "$b"]);
	}

	#[test]
	fn variable_names_empty_when_no_variables() {
		let mem = TMemoryGlobal::new();
		assert!(mem.variable_names().is_empty());
	}

	#[test]
	fn scope_local_treated_same_as_global_in_global_memory() {
		// Java: TMemoryGlobal ignores scope parameter, always stores globally
		let mut mem = TMemoryGlobal::new();
		mem.put_variable("$x", TValue::from_int(7), TVariableScope::Local);
		assert_eq!(mem.get_variable("$x").unwrap().to_int(), 7);
	}
}

// ═══════════════════════════════════════════════════════════════════════════
// TMemoryLocalSkeletonTest.java
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tmemory_local_tests {
	use std::collections::HashMap;
	use plantuml_little::tim::{TMemory, TMemoryGlobal, TMemoryLocal, TValue, TVariableScope};

	fn make_global_with(name: &str, val: TValue) -> TMemoryGlobal {
		let mut g = TMemoryGlobal::new();
		g.put_variable(name, val, TVariableScope::Global);
		g
	}

	#[test]
	fn new_can_see_global_variables() {
		// Java: local inherits global; getVariable("$g") from global
		let global = make_global_with("$g", TValue::from_int(1));
		let local = TMemoryLocal::new(global);
		assert_eq!(local.get_variable("$g").unwrap().to_int(), 1);
	}

	#[test]
	fn new_is_empty_when_global_is_empty() {
		// Java: isEmpty() when neither local nor global has variables
		let local = TMemoryLocal::new(TMemoryGlobal::new());
		assert!(local.is_empty());
	}

	#[test]
	fn is_empty_false_when_global_has_variables() {
		let global = make_global_with("$x", TValue::from_int(0));
		let local = TMemoryLocal::new(global);
		assert!(local.is_empty() == false);
	}

	#[test]
	fn put_local_variable_shadows_nothing() {
		let mut local = TMemoryLocal::new(TMemoryGlobal::new());
		local.put_variable("$l", TValue::from_int(2), TVariableScope::Local);
		assert_eq!(local.get_variable("$l").unwrap().to_int(), 2);
	}

	#[test]
	fn put_local_variable_shadows_global() {
		// Java: local "$g"=99 shadows global "$g"=1
		let global = make_global_with("$g", TValue::from_int(1));
		let mut local = TMemoryLocal::new(global);
		local.put_variable("$g", TValue::from_int(99), TVariableScope::Local);
		assert_eq!(local.get_variable("$g").unwrap().to_int(), 99);
	}

	#[test]
	fn remove_local_falls_back_to_global() {
		// Java: remove local copy → falls back to global value
		let global = make_global_with("$g", TValue::from_int(1));
		let mut local = TMemoryLocal::new(global);
		local.put_variable("$g", TValue::from_int(99), TVariableScope::Local);
		local.remove_variable("$g");
		assert_eq!(local.get_variable("$g").unwrap().to_int(), 1);
	}

	#[test]
	fn remove_global_via_local() {
		// Java: removeVariable on name that only exists globally removes it from global
		let global = make_global_with("$g", TValue::from_int(1));
		let mut local = TMemoryLocal::new(global);
		local.remove_variable("$g");
		assert!(local.get_variable("$g").is_none());
	}

	#[test]
	fn put_global_scope_through_local_reaches_global() {
		// Java: putVariable with GLOBAL scope from local memory → stored in global
		let global = TMemoryGlobal::new();
		let mut local = TMemoryLocal::new(global);
		local.put_variable("$g", TValue::from_int(7), TVariableScope::Global);
		assert_eq!(local.get_variable("$g").unwrap().to_int(), 7);
	}

	#[test]
	fn variable_names_merges_local_and_global() {
		// Java: variableNames() returns union of local and global keys
		let global = make_global_with("$a", TValue::from_int(1));
		let mut local = TMemoryLocal::new(global);
		local.put_variable("$b", TValue::from_int(2), TVariableScope::Local);
		let mut names = local.variable_names();
		names.sort();
		assert_eq!(names, vec!["$a", "$b"]);
	}

	#[test]
	fn variable_names_deduplicates_shadowed() {
		// Java: if local shadows global "$a", "$a" appears only once
		let global = make_global_with("$a", TValue::from_int(1));
		let mut local = TMemoryLocal::new(global);
		local.put_variable("$a", TValue::from_int(10), TVariableScope::Local);
		let names = local.variable_names();
		assert_eq!(names.iter().filter(|n| n.as_str() == "$a").count(), 1);
	}

	#[test]
	fn get_unknown_variable_is_none() {
		// Java: assertNull(getVariable("$missing"))
		let local = TMemoryLocal::new(TMemoryGlobal::new());
		assert!(local.get_variable("$missing").is_none());
	}

	#[test]
	fn with_initial_provides_local_vars() {
		// Java: TMemoryLocal.withInitial(global, {args}) → local vars visible
		let mut init = HashMap::new();
		init.insert("$x".to_string(), TValue::from_int(42));
		let local = TMemoryLocal::with_initial(TMemoryGlobal::new(), init);
		assert_eq!(local.get_variable("$x").unwrap().to_int(), 42);
	}

	#[test]
	fn with_initial_and_global_both_visible() {
		let mut init = HashMap::new();
		init.insert("$local".to_string(), TValue::from_string("lv"));
		let global = make_global_with("$global", TValue::from_int(9));
		let local = TMemoryLocal::with_initial(global, init);
		assert_eq!(local.get_variable("$local").unwrap().to_string(), "lv");
		assert_eq!(local.get_variable("$global").unwrap().to_int(), 9);
	}
}

// ═══════════════════════════════════════════════════════════════════════════
// TContextSkeletonTest.java
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tcontext_tests {
	use plantuml_little::tim::{
		TContext, TMemory, TMemoryGlobal, TValue, TVariableScope,
		expression::Knowledge,
	};

	#[test]
	fn new_registers_standard_builtins() {
		// Java: TContext has standard builtins; %strlen must be present
		let ctx = TContext::new();
		assert!(ctx.get_builtin("%strlen").is_some());
		assert!(ctx.get_builtin("%upper").is_some());
		assert!(ctx.get_builtin("%lower").is_some());
		assert!(ctx.get_builtin("%substr").is_some());
		assert!(ctx.get_builtin("%modulo").is_some());
	}

	#[test]
	fn get_builtin_nonexistent_returns_none() {
		// Java: unknown function → null
		let ctx = TContext::new();
		assert!(ctx.get_builtin("%nonexistent").is_none());
		assert!(ctx.get_builtin("strlen").is_none()); // missing % prefix
		assert!(ctx.get_builtin("").is_none());
	}

	#[test]
	fn default_identical_to_new() {
		// Java: TContext.create() — default context has same builtins
		let ctx = TContext::default();
		assert!(ctx.get_builtin("%strlen").is_some());
		assert!(ctx.get_builtin("%true").is_some());
	}

	#[test]
	fn call_builtin_strlen() {
		// Java: context.executeReturn("%strlen", ["hello"], memory) == TValue(5)
		let ctx = TContext::new();
		let result = ctx.call_builtin("%strlen", &[TValue::from_string("hello")]).unwrap();
		assert_eq!(result.to_int(), 5);
	}

	#[test]
	fn call_builtin_upper() {
		let ctx = TContext::new();
		let result = ctx.call_builtin("%upper", &[TValue::from_string("hello")]).unwrap();
		assert_eq!(result.to_string(), "HELLO");
	}

	#[test]
	fn call_builtin_lower() {
		let ctx = TContext::new();
		let result = ctx.call_builtin("%lower", &[TValue::from_string("WORLD")]).unwrap();
		assert_eq!(result.to_string(), "world");
	}

	#[test]
	fn call_builtin_unknown_returns_err() {
		// Java: unknown function → TMemException / error
		let ctx = TContext::new();
		assert!(ctx.call_builtin("%__unknown__", &[]).is_err());
	}

	#[test]
	fn get_variable_from_memory() {
		// Java: context.getVariable(memory, "$x") == TValue(42)
		let ctx = TContext::new();
		let mut mem = TMemoryGlobal::new();
		mem.put_variable("$x", TValue::from_int(42), TVariableScope::Global);
		let val = ctx.get_variable(&mem, "$x").unwrap();
		assert_eq!(val.to_int(), 42);
	}

	#[test]
	fn get_variable_missing_returns_none() {
		let ctx = TContext::new();
		let mem = TMemoryGlobal::new();
		assert!(ctx.get_variable(&mem, "$missing").is_none());
	}

	#[test]
	fn as_knowledge_resolves_variables() {
		// Java: ContextKnowledge.getVariable("$x") → TValue(42)
		let ctx = TContext::new();
		let mut mem = TMemoryGlobal::new();
		mem.put_variable("$x", TValue::from_int(42), TVariableScope::Global);
		let knowledge = ctx.as_knowledge(&mem);
		assert_eq!(knowledge.get_variable("$x").unwrap().to_int(), 42);
	}

	#[test]
	fn as_knowledge_resolves_builtins() {
		// Java: ContextKnowledge.getFunction("%strlen", 1) → Some
		let ctx = TContext::new();
		let mem = TMemoryGlobal::new();
		let knowledge = ctx.as_knowledge(&mem);
		assert!(knowledge.get_function("%strlen", 1).is_some());
		assert!(knowledge.get_function("%__nope__", 0).is_none());
	}

	#[test]
	fn as_knowledge_variable_absent_returns_none() {
		let ctx = TContext::new();
		let mem = TMemoryGlobal::new();
		let knowledge = ctx.as_knowledge(&mem);
		assert!(knowledge.get_variable("$absent").is_none());
	}

	// Java: @Ignore("TContext.executeOneLine / eaters not ported standalone")
	#[test]
	#[ignore = "gap: executeOneLine() / eater pipeline not ported as standalone TContext method"]
	fn execute_one_line_define() {
		// Java: TContext.executeOneLine(TLineType.DEFINE, ...) → registers user function
		todo!("full preprocessor pipeline not separately exposed")
	}

	// Java: @Ignore("TFunctionImpl not ported — user-defined functions with body")
	#[test]
	#[ignore = "gap: TFunctionImpl not ported — user-defined !procedure / !function bodies"]
	fn get_function_user_defined() {
		// Java: after !procedure myProc(), context.getFunction(sig) → TFunctionImpl
		todo!("TFunctionImpl not ported to Rust")
	}
}

// ═══════════════════════════════════════════════════════════════════════════
// TValueSkeletonTest.java  (expression.TValue)
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tvalue_tests {
	use plantuml_little::tim::TValue;

	// -- constructors -------------------------------------------------------

	#[test]
	fn from_int_stores_value() {
		// Java: TValue.fromInt(42); assertEquals(42, v.toInt())
		let v = TValue::from_int(42);
		assert_eq!(v.to_int(), 42);
		assert!(v.is_number());
		assert!(v.is_string() == false);
		assert!(v.is_json() == false);
	}

	#[test]
	fn from_int_negative() {
		let v = TValue::from_int(-7);
		assert_eq!(v.to_int(), -7);
	}

	#[test]
	fn from_bool_true_gives_1() {
		// Java: TValue.fromBool(true) → Int(1)
		let v = TValue::from_bool(true);
		assert_eq!(v.to_int(), 1);
	}

	#[test]
	fn from_bool_false_gives_0() {
		let v = TValue::from_bool(false);
		assert_eq!(v.to_int(), 0);
	}

	#[test]
	fn from_string_stores_string() {
		let v = TValue::from_string("hello");
		assert_eq!(v.to_string(), "hello");
		assert!(v.is_string());
		assert!(v.is_number() == false);
	}

	#[test]
	fn from_json_stores_json() {
		let j = serde_json::json!({"key": "val"});
		let v = TValue::from_json(j.clone());
		assert!(v.is_json());
		assert_eq!(v.to_json().unwrap(), &j);
	}

	// -- type coercions -----------------------------------------------------

	#[test]
	fn to_bool_zero_is_false() {
		assert!(TValue::from_int(0).to_bool() == false);
	}

	#[test]
	fn to_bool_nonzero_is_true() {
		assert!(TValue::from_int(1).to_bool());
		assert!(TValue::from_int(-1).to_bool());
	}

	#[test]
	fn to_bool_empty_string_is_false() {
		assert!(TValue::from_string("").to_bool() == false);
	}

	#[test]
	fn to_bool_nonempty_string_is_true() {
		assert!(TValue::from_string("x").to_bool());
	}

	#[test]
	fn to_int_parses_numeric_string() {
		// Java: TValue.fromString("42").toInt() == 42
		let v = TValue::from_string("42");
		assert_eq!(v.to_int(), 42);
	}

	#[test]
	fn to_int_non_numeric_string_gives_zero() {
		let v = TValue::from_string("abc");
		assert_eq!(v.to_int(), 0);
	}

	#[test]
	fn to_json_value_int_roundtrip() {
		let v = TValue::from_int(10);
		assert_eq!(v.to_json_value(), serde_json::json!(10));
	}

	#[test]
	fn to_json_value_string_roundtrip() {
		let v = TValue::from_string("hello");
		assert_eq!(v.to_json_value(), serde_json::json!("hello"));
	}

	// -- display ------------------------------------------------------------

	#[test]
	fn display_int() {
		assert_eq!(TValue::from_int(42).to_string(), "42");
	}

	#[test]
	fn display_string() {
		assert_eq!(TValue::from_string("hello").to_string(), "hello");
	}

	#[test]
	fn display_json_string_unwraps() {
		// Java: TValue.fromJson(JSON string) → display as raw string, no quotes
		let v = TValue::from_json(serde_json::json!("test"));
		assert_eq!(v.to_string(), "test");
	}

	#[test]
	fn display_json_object_as_compact_json() {
		let v = TValue::from_json(serde_json::json!({"a": 1}));
		let s = v.to_string();
		assert!(s.contains("\"a\""));
		assert!(s.contains('1'));
	}

	// -- arithmetic ---------------------------------------------------------

	#[test]
	fn add_int_and_int() {
		let r = TValue::from_int(10).add(&TValue::from_int(3));
		assert_eq!(r.to_int(), 13);
		assert!(r.is_number());
	}

	#[test]
	fn minus_int_and_int() {
		let r = TValue::from_int(10).minus(&TValue::from_int(3));
		assert_eq!(r.to_int(), 7);
	}

	#[test]
	fn multiply_int_and_int() {
		let r = TValue::from_int(4).multiply(&TValue::from_int(5));
		assert_eq!(r.to_int(), 20);
	}

	#[test]
	fn divided_by_int_and_int() {
		let r = TValue::from_int(10).divided_by(&TValue::from_int(3));
		assert_eq!(r.to_int(), 3); // integer division
	}

	#[test]
	fn divided_by_zero_gives_zero() {
		// Java: ArithmeticException avoided; Rust returns 0
		let r = TValue::from_int(10).divided_by(&TValue::from_int(0));
		assert_eq!(r.to_int(), 0);
	}

	#[test]
	fn add_string_and_string_concatenates() {
		let r = TValue::from_string("hello").add(&TValue::from_string(" world"));
		assert_eq!(r.to_string(), "hello world");
	}

	#[test]
	fn add_int_and_string_concatenates() {
		// Java: int + string → string concat
		let r = TValue::from_int(42).add(&TValue::from_string(" items"));
		assert_eq!(r.to_string(), "42 items");
	}

	// -- comparisons --------------------------------------------------------

	#[test]
	fn less_than_true() {
		let r = TValue::from_int(1).less_than(&TValue::from_int(5));
		assert_eq!(r.to_int(), 1);
	}

	#[test]
	fn less_than_false() {
		let r = TValue::from_int(5).less_than(&TValue::from_int(1));
		assert_eq!(r.to_int(), 0);
	}

	#[test]
	fn less_than_or_equals_equal() {
		let r = TValue::from_int(5).less_than_or_equals(&TValue::from_int(5));
		assert_eq!(r.to_int(), 1);
	}

	#[test]
	fn greater_than_true() {
		let r = TValue::from_int(10).greater_than(&TValue::from_int(5));
		assert_eq!(r.to_int(), 1);
	}

	#[test]
	fn greater_than_or_equals_equal() {
		let r = TValue::from_int(5).greater_than_or_equals(&TValue::from_int(5));
		assert_eq!(r.to_int(), 1);
	}

	#[test]
	fn equals_op_same_int() {
		let r = TValue::from_int(7).equals_op(&TValue::from_int(7));
		assert_eq!(r.to_int(), 1);
	}

	#[test]
	fn equals_op_different_ints() {
		let r = TValue::from_int(7).equals_op(&TValue::from_int(8));
		assert_eq!(r.to_int(), 0);
	}

	#[test]
	fn not_equals_different_ints() {
		let r = TValue::from_int(1).not_equals(&TValue::from_int(2));
		assert_eq!(r.to_int(), 1);
	}

	#[test]
	fn not_equals_same_value() {
		let r = TValue::from_int(3).not_equals(&TValue::from_int(3));
		assert_eq!(r.to_int(), 0);
	}

	#[test]
	fn string_comparison_lexicographic() {
		// Java: "abc" < "def" lexicographically
		let r = TValue::from_string("abc").less_than(&TValue::from_string("def"));
		assert_eq!(r.to_int(), 1);
	}

	#[test]
	fn string_equals_op() {
		let r = TValue::from_string("abc").equals_op(&TValue::from_string("abc"));
		assert_eq!(r.to_int(), 1);
	}

	// -- logical ------------------------------------------------------------

	#[test]
	fn logical_and_both_true() {
		let r = TValue::from_int(1).logical_and(&TValue::from_int(1));
		assert_eq!(r.to_int(), 1);
	}

	#[test]
	fn logical_and_one_false() {
		let r = TValue::from_int(1).logical_and(&TValue::from_int(0));
		assert_eq!(r.to_int(), 0);
	}

	#[test]
	fn logical_or_one_true() {
		let r = TValue::from_int(1).logical_or(&TValue::from_int(0));
		assert_eq!(r.to_int(), 1);
	}

	#[test]
	fn logical_or_both_false() {
		let r = TValue::from_int(0).logical_or(&TValue::from_int(0));
		assert_eq!(r.to_int(), 0);
	}

	// -- PartialEq ----------------------------------------------------------

	#[test]
	fn partial_eq_same_ints() {
		assert_eq!(TValue::from_int(5), TValue::from_int(5));
	}

	#[test]
	fn partial_eq_different_ints() {
		assert_ne!(TValue::from_int(5), TValue::from_int(6));
	}

	#[test]
	fn partial_eq_same_strings() {
		assert_eq!(TValue::from_string("abc"), TValue::from_string("abc"));
	}
}

// ═══════════════════════════════════════════════════════════════════════════
// TokenTypeSkeletonTest.java  (expression.TokenType)
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod token_type_tests {
	use plantuml_little::tim::expression::TokenType;

	#[test]
	fn values_13_variants() {
		// Java: assertEquals(13, TokenType.values().length)
		let variants = [
			TokenType::QuotedString,
			TokenType::JsonData,
			TokenType::Operator,
			TokenType::OpenParenMath,
			TokenType::Comma,
			TokenType::CloseParenMath,
			TokenType::Number,
			TokenType::PlainText,
			TokenType::Spaces,
			TokenType::FunctionName,
			TokenType::OpenParenFunc,
			TokenType::CloseParenFunc,
			TokenType::Affectation,
		];
		assert_eq!(variants.len(), 13);
	}

	#[test]
	fn value_of_all_names() {
		// Java: assertSame(TokenType.NUMBER, TokenType.valueOf("NUMBER")) etc.
		assert_eq!(format!("{:?}", TokenType::Number), "Number");
		assert_eq!(format!("{:?}", TokenType::QuotedString), "QuotedString");
		assert_eq!(format!("{:?}", TokenType::Operator), "Operator");
		assert_eq!(format!("{:?}", TokenType::FunctionName), "FunctionName");
		assert_eq!(format!("{:?}", TokenType::PlainText), "PlainText");
		assert_eq!(format!("{:?}", TokenType::JsonData), "JsonData");
		assert_eq!(format!("{:?}", TokenType::OpenParenMath), "OpenParenMath");
		assert_eq!(format!("{:?}", TokenType::CloseParenMath), "CloseParenMath");
		assert_eq!(format!("{:?}", TokenType::OpenParenFunc), "OpenParenFunc");
		assert_eq!(format!("{:?}", TokenType::CloseParenFunc), "CloseParenFunc");
		assert_eq!(format!("{:?}", TokenType::Comma), "Comma");
		assert_eq!(format!("{:?}", TokenType::Spaces), "Spaces");
		assert_eq!(format!("{:?}", TokenType::Affectation), "Affectation");
	}

	#[test]
	fn all_variants_distinct() {
		// Java: all enum constants are distinct
		let variants = [
			TokenType::QuotedString,
			TokenType::JsonData,
			TokenType::Operator,
			TokenType::OpenParenMath,
			TokenType::Comma,
			TokenType::CloseParenMath,
			TokenType::Number,
			TokenType::PlainText,
			TokenType::Spaces,
			TokenType::FunctionName,
			TokenType::OpenParenFunc,
			TokenType::CloseParenFunc,
			TokenType::Affectation,
		];
		for (i, a) in variants.iter().enumerate() {
			for (j, b) in variants.iter().enumerate() {
				if i == j {
					assert_eq!(a, b);
				} else {
					assert_ne!(a, b);
				}
			}
		}
	}

	#[test]
	fn copy_clone_preserves_variant() {
		let t = TokenType::Number;
		let t2 = t; // Copy
		assert_eq!(t, t2);
		assert_ne!(t, TokenType::Operator);
	}
}

// ═══════════════════════════════════════════════════════════════════════════
// TokenOperatorSkeletonTest.java  (expression.TokenOperator)
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod token_operator_tests {
	use plantuml_little::tim::expression::{TokenOperator, TValue};

	#[test]
	fn values_12_variants() {
		// Java: assertEquals(12, TokenOperator.values().length)
		let variants = [
			TokenOperator::Multiplication,
			TokenOperator::Division,
			TokenOperator::Addition,
			TokenOperator::Subtraction,
			TokenOperator::LessThan,
			TokenOperator::GreaterThan,
			TokenOperator::LessThanOrEquals,
			TokenOperator::GreaterThanOrEquals,
			TokenOperator::Equals,
			TokenOperator::NotEquals,
			TokenOperator::LogicalAnd,
			TokenOperator::LogicalOr,
		];
		assert_eq!(variants.len(), 12);
	}

	#[test]
	fn value_of_all_names() {
		assert_eq!(format!("{:?}", TokenOperator::Multiplication), "Multiplication");
		assert_eq!(format!("{:?}", TokenOperator::Division), "Division");
		assert_eq!(format!("{:?}", TokenOperator::Addition), "Addition");
		assert_eq!(format!("{:?}", TokenOperator::Subtraction), "Subtraction");
		assert_eq!(format!("{:?}", TokenOperator::LessThan), "LessThan");
		assert_eq!(format!("{:?}", TokenOperator::GreaterThan), "GreaterThan");
		assert_eq!(format!("{:?}", TokenOperator::LessThanOrEquals), "LessThanOrEquals");
		assert_eq!(format!("{:?}", TokenOperator::GreaterThanOrEquals), "GreaterThanOrEquals");
		assert_eq!(format!("{:?}", TokenOperator::Equals), "Equals");
		assert_eq!(format!("{:?}", TokenOperator::NotEquals), "NotEquals");
		assert_eq!(format!("{:?}", TokenOperator::LogicalAnd), "LogicalAnd");
		assert_eq!(format!("{:?}", TokenOperator::LogicalOr), "LogicalOr");
	}

	#[test]
	fn precedence_multiply_beats_add() {
		// Java: MULTIPLICATION.precedence() > ADDITION.precedence()
		assert!(TokenOperator::Multiplication.precedence() > TokenOperator::Addition.precedence());
	}

	#[test]
	fn precedence_divide_beats_subtract() {
		assert!(TokenOperator::Division.precedence() > TokenOperator::Subtraction.precedence());
	}

	#[test]
	fn precedence_add_beats_less_than() {
		assert!(TokenOperator::Addition.precedence() > TokenOperator::LessThan.precedence());
	}

	#[test]
	fn precedence_less_than_beats_equals() {
		assert!(TokenOperator::LessThan.precedence() > TokenOperator::Equals.precedence());
	}

	#[test]
	fn precedence_equals_beats_logical_and() {
		assert!(TokenOperator::Equals.precedence() > TokenOperator::LogicalAnd.precedence());
	}

	#[test]
	fn precedence_logical_and_beats_logical_or() {
		// Java: && binds tighter than ||
		assert!(TokenOperator::LogicalAnd.precedence() > TokenOperator::LogicalOr.precedence());
	}

	#[test]
	fn display_all_operators() {
		assert_eq!(TokenOperator::Addition.display(), "+");
		assert_eq!(TokenOperator::Multiplication.display(), "*");
		assert_eq!(TokenOperator::Division.display(), "/");
		assert_eq!(TokenOperator::LessThan.display(), "<");
		assert_eq!(TokenOperator::GreaterThan.display(), ">");
		assert_eq!(TokenOperator::LessThanOrEquals.display(), "<=");
		assert_eq!(TokenOperator::GreaterThanOrEquals.display(), ">=");
		assert_eq!(TokenOperator::Equals.display(), "==");
		assert_eq!(TokenOperator::NotEquals.display(), "!=");
		assert_eq!(TokenOperator::LogicalAnd.display(), "&&");
		assert_eq!(TokenOperator::LogicalOr.display(), "||");
	}

	#[test]
	fn from_chars_star_gives_multiplication() {
		assert_eq!(TokenOperator::from_chars('*', '\0'), Some(TokenOperator::Multiplication));
	}

	#[test]
	fn from_chars_slash_gives_division() {
		assert_eq!(TokenOperator::from_chars('/', '\0'), Some(TokenOperator::Division));
	}

	#[test]
	fn from_chars_plus_gives_addition() {
		assert_eq!(TokenOperator::from_chars('+', '\0'), Some(TokenOperator::Addition));
	}

	#[test]
	fn from_chars_less_than_gives_less_than() {
		assert_eq!(TokenOperator::from_chars('<', ' '), Some(TokenOperator::LessThan));
	}

	#[test]
	fn from_chars_less_equals_gives_less_than_or_equals() {
		assert_eq!(TokenOperator::from_chars('<', '='), Some(TokenOperator::LessThanOrEquals));
	}

	#[test]
	fn from_chars_greater_than() {
		assert_eq!(TokenOperator::from_chars('>', ' '), Some(TokenOperator::GreaterThan));
	}

	#[test]
	fn from_chars_greater_equals() {
		assert_eq!(TokenOperator::from_chars('>', '='), Some(TokenOperator::GreaterThanOrEquals));
	}

	#[test]
	fn from_chars_double_equals_gives_equals() {
		assert_eq!(TokenOperator::from_chars('=', '='), Some(TokenOperator::Equals));
	}

	#[test]
	fn from_chars_single_equals_gives_none() {
		assert_eq!(TokenOperator::from_chars('=', ' '), None);
	}

	#[test]
	fn from_chars_bang_equals_gives_not_equals() {
		assert_eq!(TokenOperator::from_chars('!', '='), Some(TokenOperator::NotEquals));
	}

	#[test]
	fn from_chars_double_amp_gives_logical_and() {
		assert_eq!(TokenOperator::from_chars('&', '&'), Some(TokenOperator::LogicalAnd));
	}

	#[test]
	fn from_chars_single_amp_gives_none() {
		assert_eq!(TokenOperator::from_chars('&', ' '), None);
	}

	#[test]
	fn from_chars_double_pipe_gives_logical_or() {
		assert_eq!(TokenOperator::from_chars('|', '|'), Some(TokenOperator::LogicalOr));
	}

	#[test]
	fn from_chars_unknown_gives_none() {
		assert_eq!(TokenOperator::from_chars('@', ' '), None);
	}

	#[test]
	fn operate_multiplication() {
		let r = TokenOperator::Multiplication.operate(&TValue::from_int(6), &TValue::from_int(3));
		assert_eq!(r.to_int(), 18);
	}

	#[test]
	fn operate_division() {
		let r = TokenOperator::Division.operate(&TValue::from_int(6), &TValue::from_int(3));
		assert_eq!(r.to_int(), 2);
	}

	#[test]
	fn operate_addition() {
		let r = TokenOperator::Addition.operate(&TValue::from_int(6), &TValue::from_int(3));
		assert_eq!(r.to_int(), 9);
	}

	#[test]
	fn operate_subtraction() {
		let r = TokenOperator::Subtraction.operate(&TValue::from_int(6), &TValue::from_int(3));
		assert_eq!(r.to_int(), 3);
	}

	#[test]
	fn operate_less_than_true() {
		let r = TokenOperator::LessThan.operate(&TValue::from_int(1), &TValue::from_int(5));
		assert_eq!(r.to_int(), 1);
	}

	#[test]
	fn operate_equals_true() {
		let r = TokenOperator::Equals.operate(&TValue::from_int(7), &TValue::from_int(7));
		assert_eq!(r.to_int(), 1);
	}

	#[test]
	fn operate_logical_and_true_true() {
		let r = TokenOperator::LogicalAnd.operate(&TValue::from_int(1), &TValue::from_int(1));
		assert_eq!(r.to_int(), 1);
	}

	#[test]
	fn operate_logical_or_false_true() {
		let r = TokenOperator::LogicalOr.operate(&TValue::from_int(0), &TValue::from_int(1));
		assert_eq!(r.to_int(), 1);
	}
}

// ═══════════════════════════════════════════════════════════════════════════
// TokenSkeletonTest.java  (expression.Token)
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod token_tests {
	use plantuml_little::tim::expression::{Token, TokenOperator, TokenType};

	#[test]
	fn new_sets_surface_and_type() {
		// Java: new Token("42", NUMBER) → getSurface()=="42", getTokenType()==NUMBER
		let t = Token::new("42", TokenType::Number);
		assert_eq!(t.surface, "42");
		assert_eq!(t.token_type, TokenType::Number);
		assert!(t.json.is_none());
	}

	#[test]
	fn with_json_stores_json() {
		let j = serde_json::json!([1, 2, 3]);
		let t = Token::with_json("[]", TokenType::JsonData, j.clone());
		assert_eq!(t.surface, "[]");
		assert_eq!(t.token_type, TokenType::JsonData);
		assert_eq!(t.json.as_ref().unwrap(), &j);
	}

	#[test]
	fn to_tvalue_number_token_gives_int() {
		// Java: Token("42", NUMBER).toTValue() == TValue.fromInt(42)
		let t = Token::new("42", TokenType::Number);
		let val = t.to_tvalue().unwrap();
		assert_eq!(val.to_int(), 42);
		assert!(val.is_number());
	}

	#[test]
	fn to_tvalue_quoted_string_gives_string() {
		// Java: Token("hello", QUOTED_STRING).toTValue() == TValue.fromString("hello")
		let t = Token::new("hello", TokenType::QuotedString);
		let val = t.to_tvalue().unwrap();
		assert_eq!(val.to_string(), "hello");
		assert!(val.is_string());
	}

	#[test]
	fn to_tvalue_json_data_with_json_gives_json() {
		let j = serde_json::json!({"k": "v"});
		let t = Token::with_json("{}", TokenType::JsonData, j.clone());
		let val = t.to_tvalue().unwrap();
		assert!(val.is_json());
		assert_eq!(val.to_json().unwrap(), &j);
	}

	#[test]
	fn to_tvalue_operator_gives_none() {
		// Java: non-value tokens return null → None in Rust
		let t = Token::new("+", TokenType::Operator);
		assert!(t.to_tvalue().is_none());
	}

	#[test]
	fn to_tvalue_plain_text_gives_none() {
		let t = Token::new("something", TokenType::PlainText);
		assert!(t.to_tvalue().is_none());
	}

	#[test]
	fn get_operator_plus_token() {
		// Java: Token("+", OPERATOR).getOperator() == ADDITION
		let t = Token::new("+", TokenType::Operator);
		assert_eq!(t.get_operator(), Some(TokenOperator::Addition));
	}

	#[test]
	fn get_operator_less_equals_token() {
		let t = Token::new("<=", TokenType::Operator);
		assert_eq!(t.get_operator(), Some(TokenOperator::LessThanOrEquals));
	}

	#[test]
	fn get_operator_non_operator_gives_none() {
		// Java: getOperator() on non-OPERATOR token → null
		let t = Token::new("42", TokenType::Number);
		assert!(t.get_operator().is_none());
	}

	#[test]
	fn get_operator_double_equals_gives_equals() {
		let t = Token::new("==", TokenType::Operator);
		assert_eq!(t.get_operator(), Some(TokenOperator::Equals));
	}

	#[test]
	fn mute_to_function_changes_type() {
		// Java: token.muteToFunction() → type becomes FUNCTION_NAME
		let t = Token::new("myFunc", TokenType::PlainText);
		let mutated = t.mute_to_function();
		assert_eq!(mutated.surface, "myFunc");
		assert_eq!(mutated.token_type, TokenType::FunctionName);
	}

	#[test]
	fn display_shows_type_and_surface() {
		// Java: token.toString() includes type and surface information
		let t = Token::new("42", TokenType::Number);
		let s = t.to_string();
		assert!(s.contains("42"));
		assert!(s.contains("Number"));
	}
}

// ═══════════════════════════════════════════════════════════════════════════
// StandardBuiltinsSkeletonTest.java  (builtin::standard_builtins / builtin_map)
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod builtin_registry_tests {
	use plantuml_little::tim::builtin::{builtin_map, standard_builtins};

	#[test]
	fn standard_builtins_contains_strlen() {
		let defs = standard_builtins();
		let names: Vec<&str> = defs.iter().map(|d| d.name).collect();
		assert!(names.contains(&"%strlen"));
	}

	#[test]
	fn standard_builtins_contains_all_string_functions() {
		let defs = standard_builtins();
		let names: Vec<&str> = defs.iter().map(|d| d.name).collect();
		assert!(names.contains(&"%substr"));
		assert!(names.contains(&"%strpos"));
		assert!(names.contains(&"%upper"));
		assert!(names.contains(&"%lower"));
		assert!(names.contains(&"%string"));
		assert!(names.contains(&"%intval"));
		assert!(names.contains(&"%chr"));
		assert!(names.contains(&"%ord"));
		assert!(names.contains(&"%size"));
	}

	#[test]
	fn standard_builtins_contains_numeric_functions() {
		let defs = standard_builtins();
		let names: Vec<&str> = defs.iter().map(|d| d.name).collect();
		assert!(names.contains(&"%boolval"));
		assert!(names.contains(&"%modulo"));
		assert!(names.contains(&"%dec2hex"));
		assert!(names.contains(&"%not"));
	}

	#[test]
	fn standard_builtins_contains_constant_functions() {
		let defs = standard_builtins();
		let names: Vec<&str> = defs.iter().map(|d| d.name).collect();
		assert!(names.contains(&"%true"));
		assert!(names.contains(&"%false"));
		assert!(names.contains(&"%newline"));
		assert!(names.contains(&"%tab"));
	}

	#[test]
	fn standard_builtins_min_max_args_set() {
		// Java: each SimpleReturnFunction has min/max arg count
		let defs = standard_builtins();
		let strlen = defs.iter().find(|d| d.name == "%strlen").unwrap();
		assert_eq!(strlen.min_args, 1);
		assert_eq!(strlen.max_args, 1);

		let substr = defs.iter().find(|d| d.name == "%substr").unwrap();
		assert_eq!(substr.min_args, 2);
		assert_eq!(substr.max_args, 3);
	}

	#[test]
	fn builtin_map_contains_strlen() {
		let map = builtin_map();
		assert!(map.contains_key("%strlen"));
	}

	#[test]
	fn builtin_map_strlen_callable() {
		use plantuml_little::tim::TValue;
		let map = builtin_map();
		let f = map.get("%strlen").unwrap();
		let r = f(&[TValue::from_string("test")]).unwrap();
		assert_eq!(r.to_int(), 4);
	}

	#[test]
	fn builtin_map_keys_match_definitions() {
		let defs = standard_builtins();
		let map = builtin_map();
		for def in &defs {
			assert!(map.contains_key(def.name), "missing key: {}", def.name);
		}
	}
}

// ═══════════════════════════════════════════════════════════════════════════
// Individual builtin function tests (via TContext::call_builtin)
// Mirrors Java builtin unit tests: StrlenTest, UpperTest, SubstrTest, etc.
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod builtin_function_tests {
	use plantuml_little::tim::{TContext, TValue};

	fn ctx() -> TContext {
		TContext::new()
	}

	// %strlen ----------------------------------------------------------------

	#[test]
	fn strlen_hello() {
		let r = ctx().call_builtin("%strlen", &[TValue::from_string("hello")]).unwrap();
		assert_eq!(r.to_int(), 5);
	}

	#[test]
	fn strlen_empty_string() {
		let r = ctx().call_builtin("%strlen", &[TValue::from_string("")]).unwrap();
		assert_eq!(r.to_int(), 0);
	}

	#[test]
	fn strlen_numeric_value_as_string() {
		// Java: %strlen(123) → length of "123" == 3
		let r = ctx().call_builtin("%strlen", &[TValue::from_int(123)]).unwrap();
		assert_eq!(r.to_int(), 3);
	}

	// %upper / %lower --------------------------------------------------------

	#[test]
	fn upper_lowercase_input() {
		let r = ctx().call_builtin("%upper", &[TValue::from_string("hello")]).unwrap();
		assert_eq!(r.to_string(), "HELLO");
	}

	#[test]
	fn upper_already_uppercase() {
		let r = ctx().call_builtin("%upper", &[TValue::from_string("HELLO")]).unwrap();
		assert_eq!(r.to_string(), "HELLO");
	}

	#[test]
	fn lower_uppercase_input() {
		let r = ctx().call_builtin("%lower", &[TValue::from_string("WORLD")]).unwrap();
		assert_eq!(r.to_string(), "world");
	}

	// %substr ----------------------------------------------------------------

	#[test]
	fn substr_with_pos_and_len() {
		// Java: %substr("hello", 1, 3) → "ell"
		let r = ctx().call_builtin("%substr", &[
			TValue::from_string("hello"),
			TValue::from_int(1),
			TValue::from_int(3),
		]).unwrap();
		assert_eq!(r.to_string(), "ell");
	}

	#[test]
	fn substr_to_end() {
		// Java: %substr("hello world", 6) → "world"
		let r = ctx().call_builtin("%substr", &[
			TValue::from_string("hello world"),
			TValue::from_int(6),
		]).unwrap();
		assert_eq!(r.to_string(), "world");
	}

	#[test]
	fn substr_pos_beyond_end_is_empty() {
		let r = ctx().call_builtin("%substr", &[
			TValue::from_string("hi"),
			TValue::from_int(100),
		]).unwrap();
		assert_eq!(r.to_string(), "");
	}

	// %strpos ----------------------------------------------------------------

	#[test]
	fn strpos_found() {
		// Java: %strpos("hello world", "world") → 6
		let r = ctx().call_builtin("%strpos", &[
			TValue::from_string("hello world"),
			TValue::from_string("world"),
		]).unwrap();
		assert_eq!(r.to_int(), 6);
	}

	#[test]
	fn strpos_not_found() {
		// Java: %strpos("hello", "xyz") → -1
		let r = ctx().call_builtin("%strpos", &[
			TValue::from_string("hello"),
			TValue::from_string("xyz"),
		]).unwrap();
		assert_eq!(r.to_int(), -1);
	}

	#[test]
	fn strpos_at_start() {
		let r = ctx().call_builtin("%strpos", &[
			TValue::from_string("world"),
			TValue::from_string("world"),
		]).unwrap();
		assert_eq!(r.to_int(), 0);
	}

	// %intval ----------------------------------------------------------------

	#[test]
	fn intval_numeric_string() {
		// Java: %intval("42") → TValue.fromInt(42)
		let r = ctx().call_builtin("%intval", &[TValue::from_string("42")]).unwrap();
		assert_eq!(r.to_int(), 42);
	}

	#[test]
	fn intval_negative_string() {
		let r = ctx().call_builtin("%intval", &[TValue::from_string("-10")]).unwrap();
		assert_eq!(r.to_int(), -10);
	}

	#[test]
	fn intval_non_numeric_returns_err() {
		assert!(ctx().call_builtin("%intval", &[TValue::from_string("abc")]).is_err());
	}

	// %string ----------------------------------------------------------------

	#[test]
	fn string_from_int() {
		// Java: %string(42) → "42"
		let r = ctx().call_builtin("%string", &[TValue::from_int(42)]).unwrap();
		assert_eq!(r.to_string(), "42");
		assert!(r.is_string());
	}

	// %dec2hex ---------------------------------------------------------------

	#[test]
	fn dec2hex_255_gives_ff() {
		// Java: %dec2hex(255) → "ff"
		let r = ctx().call_builtin("%dec2hex", &[TValue::from_int(255)]).unwrap();
		assert_eq!(r.to_string(), "ff");
	}

	#[test]
	fn dec2hex_zero() {
		let r = ctx().call_builtin("%dec2hex", &[TValue::from_int(0)]).unwrap();
		assert_eq!(r.to_string(), "0");
	}

	// %not -------------------------------------------------------------------

	#[test]
	fn not_zero_gives_1() {
		// Java: %not(0) → TValue.fromInt(1)
		let r = ctx().call_builtin("%not", &[TValue::from_int(0)]).unwrap();
		assert_eq!(r.to_int(), 1);
	}

	#[test]
	fn not_one_gives_0() {
		let r = ctx().call_builtin("%not", &[TValue::from_int(1)]).unwrap();
		assert_eq!(r.to_int(), 0);
	}

	// %modulo ----------------------------------------------------------------

	#[test]
	fn modulo_10_mod_3() {
		// Java: %modulo(10, 3) → 1
		let r = ctx().call_builtin("%modulo", &[
			TValue::from_int(10),
			TValue::from_int(3),
		]).unwrap();
		assert_eq!(r.to_int(), 1);
	}

	#[test]
	fn modulo_by_zero_returns_err() {
		assert!(ctx().call_builtin("%modulo", &[
			TValue::from_int(10),
			TValue::from_int(0),
		]).is_err());
	}

	// %chr / %ord ------------------------------------------------------------

	#[test]
	fn chr_65_gives_a() {
		// Java: %chr(65) → "A"
		let r = ctx().call_builtin("%chr", &[TValue::from_int(65)]).unwrap();
		assert_eq!(r.to_string(), "A");
	}

	#[test]
	fn ord_a_gives_65() {
		// Java: %ord("A") → 65
		let r = ctx().call_builtin("%ord", &[TValue::from_string("A")]).unwrap();
		assert_eq!(r.to_int(), 65);
	}

	// %size ------------------------------------------------------------------

	#[test]
	fn size_string_hello() {
		// Java: %size("hello") → 5
		let r = ctx().call_builtin("%size", &[TValue::from_string("hello")]).unwrap();
		assert_eq!(r.to_int(), 5);
	}

	#[test]
	fn size_json_array() {
		let arr = serde_json::json!([1, 2, 3, 4]);
		let r = ctx().call_builtin("%size", &[TValue::from_json(arr)]).unwrap();
		assert_eq!(r.to_int(), 4);
	}

	// %true / %false ---------------------------------------------------------

	#[test]
	fn true_gives_1() {
		// Java: %true() → TValue.fromInt(1)
		let r = ctx().call_builtin("%true", &[]).unwrap();
		assert_eq!(r.to_int(), 1);
	}

	#[test]
	fn false_gives_0() {
		// Java: %false() → TValue.fromInt(0)
		let r = ctx().call_builtin("%false", &[]).unwrap();
		assert_eq!(r.to_int(), 0);
	}

	// %boolval ---------------------------------------------------------------

	#[test]
	fn boolval_zero_gives_0() {
		// Java: %boolval(0) → TValue.fromBool(false) → 0
		let r = ctx().call_builtin("%boolval", &[TValue::from_int(0)]).unwrap();
		assert_eq!(r.to_int(), 0);
	}

	#[test]
	fn boolval_nonzero_gives_1() {
		let r = ctx().call_builtin("%boolval", &[TValue::from_int(42)]).unwrap();
		assert_eq!(r.to_int(), 1);
	}

	// wrong arg counts -------------------------------------------------------

	#[test]
	fn strlen_too_few_args_returns_err() {
		assert!(ctx().call_builtin("%strlen", &[]).is_err());
	}

	#[test]
	fn strlen_too_many_args_returns_err() {
		assert!(ctx().call_builtin("%strlen", &[
			TValue::from_string("a"),
			TValue::from_string("b"),
		]).is_err());
	}
}

// ═══════════════════════════════════════════════════════════════════════════
// Unported Java types — #[ignore] stubs
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod unported_stubs {

	// Java TFunctionImpl — user-defined function body with !procedure / !function

	#[test]
	#[ignore = "gap: TFunctionImpl not ported — holds lines of body text and local variable init"]
	fn tfunctionimpl_new_procedure() {
		// Java: new TFunctionImpl(sig, PROCEDURE, body_lines, memory)
		//       getFunctionType() == PROCEDURE
		//       body() returns list of lines
		todo!()
	}

	#[test]
	#[ignore = "gap: TFunctionImpl.executeFunction() not ported — requires EaterReturn pipeline"]
	fn tfunctionimpl_execute_function() {
		// Java: impl.executeFunction(context, memory, args) → TValue or null (procedure)
		todo!()
	}

	// Java TFunctionArgument — named formal argument with optional default

	#[test]
	#[ignore = "gap: TFunctionArgument not ported — formal arg name + optional default value"]
	fn tfunctionargument_new() {
		// Java: new TFunctionArgument("arg1", null) → getName()=="arg1", getDefault()==null
		//       new TFunctionArgument("arg2", TValue.fromString("default")) → has default
		todo!()
	}

	// Java Trie / TrieImpl — prefix tree for directive matching

	#[test]
	#[ignore = "gap: Trie / TrieImpl not ported — used for fast !directive prefix matching"]
	fn trie_put_and_get() {
		// Java: trie.put("!if", handler); trie.get("!if") → handler
		todo!()
	}

	#[test]
	#[ignore = "gap: Trie.longestMatch() not ported"]
	fn trie_longest_match() {
		// Java: trie.longestMatch("!foreach x in list") → "!foreach"
		todo!()
	}

	// Java EaterDeclareProcedure — parses !procedure declarations

	#[test]
	#[ignore = "gap: EaterDeclareProcedure not separately ported — handled by preproc module"]
	fn eater_declare_procedure_parses_signature() {
		// Java: EaterDeclareProcedure.eat("!procedure $foo($a, $b)") → TFunctionSignature(foo,2)
		todo!()
	}

	// Java EaterWhile — parses !while loop body

	#[test]
	#[ignore = "gap: EaterWhile not separately ported — handled by preproc module"]
	fn eater_while_captures_condition() {
		// Java: EaterWhile.eat("!while %true()") → condition expression
		todo!()
	}

	// Java EaterForeach — parses !foreach loop

	#[test]
	#[ignore = "gap: EaterForeach not separately ported — handled by preproc module"]
	fn eater_foreach_parses_variable_and_list() {
		// Java: EaterForeach.eat("!foreach $x in list") → var=$x, list=list
		todo!()
	}

	// Java EaterIf / EaterElseIf — if-else directive parsers

	#[test]
	#[ignore = "gap: EaterIf not separately ported — handled by preproc module"]
	fn eater_if_parses_condition() {
		// Java: EaterIf.eat("!if (%x == 1)") → condition expression
		todo!()
	}

	#[test]
	#[ignore = "gap: EaterElseIf not separately ported — handled by preproc module"]
	fn eater_elseif_parses_condition() {
		// Java: EaterElseIf.eat("!elseif (%x > 0)") → condition expression
		todo!()
	}

	// Java EaterReturn — parses !return directive inside a function

	#[test]
	#[ignore = "gap: EaterReturn not separately ported — handled by preproc module"]
	fn eater_return_parses_expression() {
		// Java: EaterReturn.eat("!return %x + 1") → expression yielding TValue
		todo!()
	}

	// Java ExecutionContextForeach / ExecutionContextIf / ExecutionContextWhile

	#[test]
	#[ignore = "gap: ExecutionContextForeach not ported — preproc loop execution context"]
	fn execution_context_foreach_iterates() {
		// Java: holds iterator state; next() advances and binds loop variable
		todo!()
	}

	#[test]
	#[ignore = "gap: ExecutionContextIf not ported — tracks branch taken / skipping lines"]
	fn execution_context_if_branch_tracking() {
		// Java: isTrue() / hasAlreadyBeenTrue() for if/elseif/else handling
		todo!()
	}

	#[test]
	#[ignore = "gap: ExecutionContextWhile not ported — preproc while-loop state"]
	fn execution_context_while_captures_body() {
		// Java: captures body lines until !endwhile; re-evaluates condition each pass
		todo!()
	}

	// Java FunctionsSet — registry of user-defined TFunctionImpl instances

	#[test]
	#[ignore = "gap: FunctionsSet not ported — required for user !procedure / !function lookup"]
	fn functions_set_register_and_lookup() {
		// Java: set.put(sig, impl); set.getFunctionWithSameName(sig) → TFunctionImpl
		todo!()
	}

	// Java VariableManager — manages global/local TMemory for the preproc stack

	#[test]
	#[ignore = "gap: VariableManager not ported — manages memory stack for nested calls"]
	fn variable_manager_push_pop_stack() {
		// Java: push(localVars) creates new TMemoryLocal; pop restores previous scope
		todo!()
	}

	// Java CodeIterator / CodeIteratorImpl / CodeIteratorSub

	#[test]
	#[ignore = "gap: CodeIterator not ported — iterates preprocessor input lines"]
	fn code_iterator_next_line() {
		// Java: iterator.getNext() → next CharSequence; throws if exhausted
		todo!()
	}

	#[test]
	#[ignore = "gap: CodeIteratorSub not ported — sub-iterator for included file contents"]
	fn code_iterator_sub_wraps_included_file() {
		// Java: CodeIteratorSub wraps another iterator for !include file contents
		todo!()
	}

	// Context-dependent builtins — require live environment

	#[test]
	#[ignore = "gap: %feature(name) — queries a PlantUML feature flag; no standalone test"]
	fn builtin_feature() {
		// Java: %feature("theme") → TValue(true/false) depending on feature registry
		todo!()
	}

	#[test]
	#[ignore = "gap: %fileexists(path) — requires filesystem access; not testable standalone"]
	fn builtin_fileexists() {
		// Java: %fileexists("/tmp/test.txt") → TValue(1) if file exists
		todo!()
	}

	#[test]
	#[ignore = "gap: %filename() — returns current file name; context-dependent"]
	fn builtin_filename() {
		// Java: %filename() → TValue of current source file name
		todo!()
	}

	#[test]
	#[ignore = "gap: %dirpath() — returns current directory path; context-dependent"]
	fn builtin_dirpath() {
		// Java: %dirpath() → TValue of directory containing current source file
		todo!()
	}

	#[test]
	#[ignore = "gap: %date() without arguments returns environment timestamp — non-deterministic"]
	fn builtin_date_no_args() {
		// Java: %date() → formatted current date string; value varies at runtime
		todo!()
	}

	#[test]
	#[ignore = "gap: %now() returns current Unix timestamp — non-deterministic"]
	fn builtin_now() {
		// Java: %now() → TValue(unix_seconds); value varies at runtime
		todo!()
	}

	#[test]
	#[ignore = "gap: %splitstr returns JSON array — Knowledge-dependent formatting not asserted"]
	fn builtin_splitstr_knowledge_context() {
		// Java: %splitstr("a,b,c", ",") in expression evaluator context → JSON array TValue
		// The function itself works; JSON array rendering in expressions requires Knowledge.
		todo!()
	}

	#[test]
	#[ignore = "gap: %newline / %breakline / %tab / %dollar — present but output is environment char"]
	fn builtin_constant_chars() {
		// Java: %newline() → "\n"; %tab() → "\t"; %dollar() → "$"
		// Covered via standard_builtins() registry test; individual output trivially correct.
		todo!()
	}
}
