//! Condition evaluation for `when` attributes and `<hick:when>` tags.

use hick_exec::state::MultiDocumentState;

/// A simple condition on a variable.
#[derive(Debug, Clone)]
pub enum Condition {
    /// Variable is defined and non-empty.
    Defined(String),
    /// Variable is undefined or empty.
    NotDefined(String),
    /// Variable equals value.
    Equals(String, String),
    /// Variable does not equal value.
    NotEquals(String, String),
}

impl Condition {
    /// Parse a condition string.
    ///
    /// Supported syntax:
    /// - `var_name` — true if defined and non-empty
    /// - `!var_name` — true if undefined or empty
    /// - `var_name=value` — true if equals
    /// - `var_name!=value` — true if not equals
    pub fn parse(s: &str) -> Self {
        let s = s.trim();
        if let Some(rest) = s.strip_prefix('!') {
            Condition::NotDefined(rest.to_string())
        } else if let Some((name, value)) = s.split_once("!=") {
            Condition::NotEquals(name.to_string(), value.to_string())
        } else if let Some((name, value)) = s.split_once('=') {
            Condition::Equals(name.to_string(), value.to_string())
        } else {
            Condition::Defined(s.to_string())
        }
    }

    /// Evaluate against resolved variables.
    pub fn evaluate(&self, state: &MultiDocumentState) -> bool {
        match self {
            Condition::Defined(name) => state.resolve_var(name).is_some_and(|v| !v.is_empty()),
            Condition::NotDefined(name) => state.resolve_var(name).is_none_or(|v| v.is_empty()),
            Condition::Equals(name, expected) => {
                state.resolve_var(name).as_deref() == Some(expected.as_str())
            }
            Condition::NotEquals(name, expected) => {
                state.resolve_var(name).as_deref() != Some(expected.as_str())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_defined() {
        assert!(matches!(Condition::parse("debug"), Condition::Defined(n) if n == "debug"));
    }

    #[test]
    fn parse_not_defined() {
        assert!(matches!(Condition::parse("!debug"), Condition::NotDefined(n) if n == "debug"));
    }

    #[test]
    fn parse_equals() {
        if let Condition::Equals(n, v) = Condition::parse("env=prod") {
            assert_eq!(n, "env");
            assert_eq!(v, "prod");
        } else {
            panic!("expected Equals");
        }
    }

    #[test]
    fn parse_not_equals() {
        if let Condition::NotEquals(n, v) = Condition::parse("env!=dev") {
            assert_eq!(n, "env");
            assert_eq!(v, "dev");
        } else {
            panic!("expected NotEquals");
        }
    }

    #[test]
    fn evaluate_defined() {
        let state = MultiDocumentState::default();
        state.set_param("debug".into(), "1".into());

        assert!(Condition::parse("debug").evaluate(&state));
        assert!(!Condition::parse("!debug").evaluate(&state));
        assert!(!Condition::parse("missing").evaluate(&state));
        assert!(Condition::parse("!missing").evaluate(&state));
    }

    #[test]
    fn evaluate_equals() {
        let state = MultiDocumentState::default();
        state.set_param("env".into(), "prod".into());

        assert!(Condition::parse("env=prod").evaluate(&state));
        assert!(!Condition::parse("env=dev").evaluate(&state));
        assert!(!Condition::parse("env!=prod").evaluate(&state));
        assert!(Condition::parse("env!=dev").evaluate(&state));
    }
}
