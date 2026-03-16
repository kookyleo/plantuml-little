//! TIM built-in functions.
//!
//! Port of Java PlantUML's `net.sourceforge.plantuml.tim.builtin` package.
//! Each function mirrors the Java `SimpleReturnFunction` pattern:
//! `%name(args...) -> TValue`.
//!
//! The actual preprocessing invocation of these functions happens in
//! `crate::preproc`; this module exposes them as standalone callables
//! with the Java-compatible interface for use by other modules.

use std::collections::HashMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use super::expression::{BuiltinFn, TValue};

// ---------------------------------------------------------------------------
// Function registry
// ---------------------------------------------------------------------------

/// A registered TIM built-in function descriptor.
#[derive(Clone)]
pub struct BuiltinDef {
    /// Function name including the `%` prefix, e.g. `"%strlen"`.
    pub name: &'static str,
    /// Minimum number of positional arguments.
    pub min_args: usize,
    /// Maximum number of positional arguments.
    pub max_args: usize,
    /// The implementation.
    pub func: BuiltinFn,
}

/// Build the standard set of built-in function definitions.
///
/// This mirrors the `addStandardFunctions()` method in Java `TContext`.
/// Only functions that can be evaluated without external context (file system,
/// TIM memory) are included; context-dependent ones (e.g. `%filename`,
/// `%dirpath`) are handled by the preprocessor in `crate::preproc`.
pub fn standard_builtins() -> Vec<BuiltinDef> {
    vec![
        // -- string functions -----------------------------------------------
        BuiltinDef {
            name: "%strlen",
            min_args: 1,
            max_args: 1,
            func: builtin_strlen,
        },
        BuiltinDef {
            name: "%substr",
            min_args: 2,
            max_args: 3,
            func: builtin_substr,
        },
        BuiltinDef {
            name: "%strpos",
            min_args: 2,
            max_args: 2,
            func: builtin_strpos,
        },
        BuiltinDef {
            name: "%upper",
            min_args: 1,
            max_args: 1,
            func: builtin_upper,
        },
        BuiltinDef {
            name: "%lower",
            min_args: 1,
            max_args: 1,
            func: builtin_lower,
        },
        BuiltinDef {
            name: "%string",
            min_args: 1,
            max_args: 1,
            func: builtin_string,
        },
        BuiltinDef {
            name: "%intval",
            min_args: 1,
            max_args: 1,
            func: builtin_intval,
        },
        BuiltinDef {
            name: "%chr",
            min_args: 1,
            max_args: 1,
            func: builtin_chr,
        },
        BuiltinDef {
            name: "%ord",
            min_args: 1,
            max_args: 1,
            func: builtin_ord,
        },
        BuiltinDef {
            name: "%size",
            min_args: 1,
            max_args: 1,
            func: builtin_size,
        },
        BuiltinDef {
            name: "%splitstr",
            min_args: 2,
            max_args: 2,
            func: builtin_splitstr,
        },
        // -- numeric / logic ------------------------------------------------
        BuiltinDef {
            name: "%boolval",
            min_args: 1,
            max_args: 1,
            func: builtin_boolval,
        },
        BuiltinDef {
            name: "%modulo",
            min_args: 2,
            max_args: 2,
            func: builtin_modulo,
        },
        BuiltinDef {
            name: "%dec2hex",
            min_args: 1,
            max_args: 1,
            func: builtin_dec2hex,
        },
        BuiltinDef {
            name: "%hex2dec",
            min_args: 1,
            max_args: 1,
            func: builtin_hex2dec,
        },
        BuiltinDef {
            name: "%not",
            min_args: 1,
            max_args: 1,
            func: builtin_not,
        },
        // -- constant functions ---------------------------------------------
        BuiltinDef {
            name: "%true",
            min_args: 0,
            max_args: 0,
            func: builtin_true,
        },
        BuiltinDef {
            name: "%false",
            min_args: 0,
            max_args: 0,
            func: builtin_false,
        },
        BuiltinDef {
            name: "%newline",
            min_args: 0,
            max_args: 0,
            func: builtin_newline,
        },
        BuiltinDef {
            name: "%n",
            min_args: 0,
            max_args: 0,
            func: builtin_newline,
        },
        BuiltinDef {
            name: "%backslash",
            min_args: 0,
            max_args: 0,
            func: builtin_backslash,
        },
        BuiltinDef {
            name: "%tab",
            min_args: 0,
            max_args: 0,
            func: builtin_tab,
        },
        // -- date / time ----------------------------------------------------
        BuiltinDef {
            name: "%date",
            min_args: 0,
            max_args: 2,
            func: builtin_date,
        },
        BuiltinDef {
            name: "%now",
            min_args: 0,
            max_args: 0,
            func: builtin_now,
        },
        // -- version --------------------------------------------------------
        BuiltinDef {
            name: "%version",
            min_args: 0,
            max_args: 0,
            func: builtin_version,
        },
    ]
}

/// Build a lookup map from function name to `BuiltinFn`.
pub fn builtin_map() -> HashMap<String, BuiltinFn> {
    standard_builtins()
        .into_iter()
        .map(|def| (def.name.to_string(), def.func))
        .collect()
}

// ---------------------------------------------------------------------------
// Implementation of individual built-in functions
// ---------------------------------------------------------------------------

/// `%strlen(s)` — string length.
fn builtin_strlen(args: &[TValue]) -> Result<TValue, String> {
    check_args("%strlen", args, 1, 1)?;
    Ok(TValue::from_int(args[0].to_string().len() as i64))
}

/// `%substr(s, pos [, len])` — substring extraction.
fn builtin_substr(args: &[TValue]) -> Result<TValue, String> {
    check_args("%substr", args, 2, 3)?;
    let full = args[0].to_string();
    let pos = args[1].to_int() as usize;
    if pos >= full.len() {
        return Ok(TValue::from_string(""));
    }
    let rest = &full[pos..];
    if args.len() == 3 {
        let len = args[2].to_int() as usize;
        let end = len.min(rest.len());
        Ok(TValue::from_string(&rest[..end]))
    } else {
        Ok(TValue::from_string(rest))
    }
}

/// `%strpos(haystack, needle)` — find position of substring (-1 if not found).
fn builtin_strpos(args: &[TValue]) -> Result<TValue, String> {
    check_args("%strpos", args, 2, 2)?;
    let haystack = args[0].to_string();
    let needle = args[1].to_string();
    let pos = haystack
        .find(&needle)
        .map(|p| p as i64)
        .unwrap_or(-1);
    Ok(TValue::from_int(pos))
}

/// `%upper(s)` — convert to uppercase.
fn builtin_upper(args: &[TValue]) -> Result<TValue, String> {
    check_args("%upper", args, 1, 1)?;
    Ok(TValue::from_string(args[0].to_string().to_uppercase()))
}

/// `%lower(s)` — convert to lowercase.
fn builtin_lower(args: &[TValue]) -> Result<TValue, String> {
    check_args("%lower", args, 1, 1)?;
    Ok(TValue::from_string(args[0].to_string().to_lowercase()))
}

/// `%string(v)` — convert to string.
fn builtin_string(args: &[TValue]) -> Result<TValue, String> {
    check_args("%string", args, 1, 1)?;
    Ok(TValue::from_string(args[0].to_string()))
}

/// `%intval(s)` — parse an integer from a string.
fn builtin_intval(args: &[TValue]) -> Result<TValue, String> {
    check_args("%intval", args, 1, 1)?;
    let s = args[0].to_string();
    let n = s
        .trim()
        .parse::<i64>()
        .map_err(|_| format!("Cannot convert '{}' to integer", s))?;
    Ok(TValue::from_int(n))
}

/// `%chr(n)` — character from code point.
fn builtin_chr(args: &[TValue]) -> Result<TValue, String> {
    check_args("%chr", args, 1, 1)?;
    let n = args[0].to_int() as u32;
    let ch = char::from_u32(n).unwrap_or('?');
    Ok(TValue::from_string(ch.to_string()))
}

/// `%ord(s)` — code point of first character.
fn builtin_ord(args: &[TValue]) -> Result<TValue, String> {
    check_args("%ord", args, 1, 1)?;
    let s = args[0].to_string();
    let code = s.chars().next().map(|c| c as i64).unwrap_or(0);
    Ok(TValue::from_int(code))
}

/// `%size(v)` — size of a JSON array/object, or string length.
fn builtin_size(args: &[TValue]) -> Result<TValue, String> {
    check_args("%size", args, 1, 1)?;
    let v = &args[0];
    match v {
        TValue::Int(_) => Ok(TValue::from_int(0)),
        TValue::Str(s) => Ok(TValue::from_int(s.len() as i64)),
        TValue::Json(json) => {
            let sz = match json {
                serde_json::Value::Array(arr) => arr.len() as i64,
                serde_json::Value::Object(obj) => obj.len() as i64,
                _ => 0,
            };
            Ok(TValue::from_int(sz))
        }
    }
}

/// `%splitstr(s, sep)` — split string into a JSON array.
fn builtin_splitstr(args: &[TValue]) -> Result<TValue, String> {
    check_args("%splitstr", args, 2, 2)?;
    let s = args[0].to_string();
    let sep = args[1].to_string();
    let parts: Vec<serde_json::Value> = s
        .split(&sep)
        .map(|p| serde_json::Value::String(p.to_string()))
        .collect();
    Ok(TValue::from_json(serde_json::Value::Array(parts)))
}

/// `%boolval(v)` — convert to boolean (0 or 1).
fn builtin_boolval(args: &[TValue]) -> Result<TValue, String> {
    check_args("%boolval", args, 1, 1)?;
    Ok(TValue::from_bool(args[0].to_bool()))
}

/// `%modulo(a, b)` — integer modulo.
fn builtin_modulo(args: &[TValue]) -> Result<TValue, String> {
    check_args("%modulo", args, 2, 2)?;
    let a = args[0].to_int();
    let b = args[1].to_int();
    if b == 0 {
        return Err("Division by zero in %modulo".to_string());
    }
    Ok(TValue::from_int(a % b))
}

/// `%dec2hex(n)` — decimal to hexadecimal string.
fn builtin_dec2hex(args: &[TValue]) -> Result<TValue, String> {
    check_args("%dec2hex", args, 1, 1)?;
    let n = args[0].to_int();
    Ok(TValue::from_string(format!("{:x}", n)))
}

/// `%hex2dec(s)` — hexadecimal string to decimal integer.
fn builtin_hex2dec(args: &[TValue]) -> Result<TValue, String> {
    check_args("%hex2dec", args, 1, 1)?;
    let s = args[0].to_string();
    let hex = s
        .trim()
        .strip_prefix("0x")
        .or_else(|| s.trim().strip_prefix("0X"))
        .or_else(|| s.trim().strip_prefix('#'))
        .unwrap_or(s.trim());
    let n = i64::from_str_radix(hex, 16).unwrap_or(0);
    Ok(TValue::from_int(n))
}

/// `%not(v)` — logical NOT.
fn builtin_not(args: &[TValue]) -> Result<TValue, String> {
    check_args("%not", args, 1, 1)?;
    Ok(TValue::from_bool(!args[0].to_bool()))
}

/// `%true()` — boolean true (1).
fn builtin_true(args: &[TValue]) -> Result<TValue, String> {
    check_args("%true", args, 0, 0)?;
    Ok(TValue::from_int(1))
}

/// `%false()` — boolean false (0).
fn builtin_false(args: &[TValue]) -> Result<TValue, String> {
    check_args("%false", args, 0, 0)?;
    Ok(TValue::from_int(0))
}

/// `%newline()` / `%n()` — newline character.
fn builtin_newline(args: &[TValue]) -> Result<TValue, String> {
    let _ = args; // 0 args expected
    Ok(TValue::from_string("\n"))
}

/// `%backslash()` — backslash character.
fn builtin_backslash(args: &[TValue]) -> Result<TValue, String> {
    let _ = args;
    Ok(TValue::from_string("\\"))
}

/// `%tab()` — tab character.
fn builtin_tab(args: &[TValue]) -> Result<TValue, String> {
    let _ = args;
    Ok(TValue::from_string("\t"))
}

/// `%date([format [, timestamp]])` — formatted date string.
fn builtin_date(args: &[TValue]) -> Result<TValue, String> {
    if args.is_empty() {
        // No format: return Unix timestamp as integer
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        return Ok(TValue::from_int(now));
    }
    let format = args[0].to_string();
    let timestamp_secs = if args.len() >= 2 {
        args[1].to_int()
    } else {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64
    };
    let formatted = format_unix_timestamp(&format, timestamp_secs);
    Ok(TValue::from_string(formatted))
}

/// `%now()` — current Unix timestamp in seconds.
fn builtin_now(args: &[TValue]) -> Result<TValue, String> {
    let _ = args;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    Ok(TValue::from_int(now))
}

/// `%version()` — PlantUML version string.
fn builtin_version(args: &[TValue]) -> Result<TValue, String> {
    let _ = args;
    Ok(TValue::from_string(env!("CARGO_PKG_VERSION")))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn check_args(name: &str, args: &[TValue], min: usize, max: usize) -> Result<(), String> {
    if args.len() < min || args.len() > max {
        Err(format!(
            "{} expects {}-{} arguments, got {}",
            name,
            min,
            max,
            args.len()
        ))
    } else {
        Ok(())
    }
}

/// Format a Unix timestamp using Java-style SimpleDateFormat patterns.
///
/// Supports: `yyyy`, `yy`, `MM`, `dd`, `HH`, `mm`, `ss`.
fn format_unix_timestamp(pattern: &str, timestamp_secs: i64) -> String {
    let days = timestamp_secs.div_euclid(86400);
    let time_of_day = timestamp_secs.rem_euclid(86400);
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Civil date from unix days (Howard Hinnant's algorithm)
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };

    let mut result = pattern.to_string();
    result = result.replace("yyyy", &format!("{year:04}"));
    result = result.replace("yy", &format!("{:02}", year % 100));
    result = result.replace("MM", &format!("{m:02}"));
    result = result.replace("dd", &format!("{d:02}"));
    result = result.replace("HH", &format!("{hours:02}"));
    result = result.replace("mm", &format!("{minutes:02}"));
    result = result.replace("ss", &format!("{seconds:02}"));
    result
}

// ---------------------------------------------------------------------------
// Context-dependent builtins (constructed with captured state)
// ---------------------------------------------------------------------------

/// Create a `%filename()` function that returns the given filename.
pub fn make_filename_fn(filename: String) -> BuiltinFn {
    // We can't capture state in a plain fn pointer, so we use a static
    // approach: the caller should use `BuiltinDef` with the function.
    // For now, provide a standalone approach via closure-based registration
    // in `SimpleKnowledge`.
    //
    // Fallback: return empty string.
    let _ = filename;
    builtin_filename_empty
}

fn builtin_filename_empty(args: &[TValue]) -> Result<TValue, String> {
    let _ = args;
    Ok(TValue::from_string(""))
}

/// Create a `%dirpath()` function that returns the given directory path.
pub fn make_dirpath_fn(dirpath: String) -> BuiltinFn {
    let _ = dirpath;
    builtin_dirpath_empty
}

fn builtin_dirpath_empty(args: &[TValue]) -> Result<TValue, String> {
    let _ = args;
    Ok(TValue::from_string(""))
}

/// Helper to create a context-aware `%filename()` value from a file path.
pub fn filename_from_path(path: &Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string()
}

/// Helper to create a context-aware `%dirpath()` value from a file path.
pub fn dirpath_from_path(path: &Path) -> String {
    path.parent()
        .and_then(|p| p.to_str())
        .unwrap_or("")
        .to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strlen() {
        let r = builtin_strlen(&[TValue::from_string("hello")]).unwrap();
        assert_eq!(r.to_int(), 5);

        let r = builtin_strlen(&[TValue::from_string("")]).unwrap();
        assert_eq!(r.to_int(), 0);

        let r = builtin_strlen(&[TValue::from_int(123)]).unwrap();
        assert_eq!(r.to_int(), 3); // "123".len()
    }

    #[test]
    fn test_substr() {
        let r = builtin_substr(&[
            TValue::from_string("hello world"),
            TValue::from_int(6),
        ])
        .unwrap();
        assert_eq!(r.to_string(), "world");

        let r = builtin_substr(&[
            TValue::from_string("hello world"),
            TValue::from_int(0),
            TValue::from_int(5),
        ])
        .unwrap();
        assert_eq!(r.to_string(), "hello");

        // pos beyond end
        let r = builtin_substr(&[
            TValue::from_string("hi"),
            TValue::from_int(100),
        ])
        .unwrap();
        assert_eq!(r.to_string(), "");
    }

    #[test]
    fn test_strpos() {
        let r = builtin_strpos(&[
            TValue::from_string("hello world"),
            TValue::from_string("world"),
        ])
        .unwrap();
        assert_eq!(r.to_int(), 6);

        let r = builtin_strpos(&[
            TValue::from_string("hello"),
            TValue::from_string("xyz"),
        ])
        .unwrap();
        assert_eq!(r.to_int(), -1);
    }

    #[test]
    fn test_upper_lower() {
        let r = builtin_upper(&[TValue::from_string("hello")]).unwrap();
        assert_eq!(r.to_string(), "HELLO");

        let r = builtin_lower(&[TValue::from_string("HELLO")]).unwrap();
        assert_eq!(r.to_string(), "hello");
    }

    #[test]
    fn test_intval() {
        let r = builtin_intval(&[TValue::from_string("42")]).unwrap();
        assert_eq!(r.to_int(), 42);

        let r = builtin_intval(&[TValue::from_string("-10")]).unwrap();
        assert_eq!(r.to_int(), -10);

        assert!(builtin_intval(&[TValue::from_string("abc")]).is_err());
    }

    #[test]
    fn test_chr_ord() {
        let r = builtin_chr(&[TValue::from_int(65)]).unwrap();
        assert_eq!(r.to_string(), "A");

        let r = builtin_ord(&[TValue::from_string("A")]).unwrap();
        assert_eq!(r.to_int(), 65);
    }

    #[test]
    fn test_size() {
        let r = builtin_size(&[TValue::from_string("abc")]).unwrap();
        assert_eq!(r.to_int(), 3);

        let r = builtin_size(&[TValue::from_int(42)]).unwrap();
        assert_eq!(r.to_int(), 0);

        let json_arr = serde_json::json!([1, 2, 3]);
        let r = builtin_size(&[TValue::from_json(json_arr)]).unwrap();
        assert_eq!(r.to_int(), 3);

        let json_obj = serde_json::json!({"a": 1, "b": 2});
        let r = builtin_size(&[TValue::from_json(json_obj)]).unwrap();
        assert_eq!(r.to_int(), 2);
    }

    #[test]
    fn test_splitstr() {
        let r = builtin_splitstr(&[
            TValue::from_string("a,b,c"),
            TValue::from_string(","),
        ])
        .unwrap();
        if let TValue::Json(serde_json::Value::Array(arr)) = r {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0].as_str().unwrap(), "a");
            assert_eq!(arr[2].as_str().unwrap(), "c");
        } else {
            panic!("Expected JSON array");
        }
    }

    #[test]
    fn test_boolval() {
        let r = builtin_boolval(&[TValue::from_int(0)]).unwrap();
        assert_eq!(r.to_int(), 0);

        let r = builtin_boolval(&[TValue::from_int(42)]).unwrap();
        assert_eq!(r.to_int(), 1);

        let r = builtin_boolval(&[TValue::from_string("")]).unwrap();
        assert_eq!(r.to_int(), 0);

        let r = builtin_boolval(&[TValue::from_string("x")]).unwrap();
        assert_eq!(r.to_int(), 1);
    }

    #[test]
    fn test_modulo() {
        let r = builtin_modulo(&[TValue::from_int(10), TValue::from_int(3)]).unwrap();
        assert_eq!(r.to_int(), 1);

        assert!(builtin_modulo(&[TValue::from_int(10), TValue::from_int(0)]).is_err());
    }

    #[test]
    fn test_dec2hex() {
        let r = builtin_dec2hex(&[TValue::from_int(255)]).unwrap();
        assert_eq!(r.to_string(), "ff");

        let r = builtin_dec2hex(&[TValue::from_int(0)]).unwrap();
        assert_eq!(r.to_string(), "0");
    }

    #[test]
    fn test_hex2dec() {
        let r = builtin_hex2dec(&[TValue::from_string("ff")]).unwrap();
        assert_eq!(r.to_int(), 255);

        let r = builtin_hex2dec(&[TValue::from_string("0xFF")]).unwrap();
        assert_eq!(r.to_int(), 255);
    }

    #[test]
    fn test_not() {
        let r = builtin_not(&[TValue::from_int(0)]).unwrap();
        assert!(r.to_bool());

        let r = builtin_not(&[TValue::from_int(1)]).unwrap();
        assert!(!r.to_bool());
    }

    #[test]
    fn test_true_false() {
        assert_eq!(builtin_true(&[]).unwrap().to_int(), 1);
        assert_eq!(builtin_false(&[]).unwrap().to_int(), 0);
    }

    #[test]
    fn test_newline_backslash_tab() {
        assert_eq!(builtin_newline(&[]).unwrap().to_string(), "\n");
        assert_eq!(builtin_backslash(&[]).unwrap().to_string(), "\\");
        assert_eq!(builtin_tab(&[]).unwrap().to_string(), "\t");
    }

    #[test]
    fn test_now() {
        let r = builtin_now(&[]).unwrap();
        assert!(r.to_int() > 0);
    }

    #[test]
    fn test_version() {
        let r = builtin_version(&[]).unwrap();
        let v = r.to_string();
        assert!(!v.is_empty());
    }

    #[test]
    fn test_date_with_format_and_timestamp() {
        // 2020-01-01 00:00:00 UTC = 1577836800
        let r = builtin_date(&[
            TValue::from_string("yyyy-MM-dd"),
            TValue::from_int(1577836800),
        ])
        .unwrap();
        assert_eq!(r.to_string(), "2020-01-01");
    }

    #[test]
    fn test_standard_builtins_registry() {
        let builtins = standard_builtins();
        let names: Vec<&str> = builtins.iter().map(|b| b.name).collect();
        assert!(names.contains(&"%strlen"));
        assert!(names.contains(&"%substr"));
        assert!(names.contains(&"%upper"));
        assert!(names.contains(&"%lower"));
        assert!(names.contains(&"%date"));
        assert!(names.contains(&"%now"));
    }

    #[test]
    fn test_builtin_map() {
        let map = builtin_map();
        assert!(map.contains_key("%strlen"));
        assert!(map.contains_key("%upper"));

        let strlen = map.get("%strlen").unwrap();
        let r = strlen(&[TValue::from_string("test")]).unwrap();
        assert_eq!(r.to_int(), 4);
    }

    #[test]
    fn test_filename_dirpath_helpers() {
        let p = Path::new("/tmp/diagrams/test.puml");
        assert_eq!(filename_from_path(p), "test.puml");
        assert_eq!(dirpath_from_path(p), "/tmp/diagrams");
    }

    #[test]
    fn test_wrong_arg_count() {
        assert!(builtin_strlen(&[]).is_err());
        assert!(builtin_strlen(&[TValue::from_int(1), TValue::from_int(2)]).is_err());
        assert!(builtin_substr(&[TValue::from_string("x")]).is_err());
    }
}
