//! Condition evaluation for `when` attributes and `<hick:when>` tags.
//!
//! Supports compound boolean expressions with:
//! - `&&` (AND) - higher precedence than OR
//! - `||` (OR) - lower precedence than AND
//! - `!` (NOT) - prefix operator, highest precedence
//! - Parentheses for grouping
//!
//! Grammar:
//! ```text
//! expr     := or_expr
//! or_expr  := and_expr ('||' and_expr)*
//! and_expr := unary ('&&' unary)*
//! unary    := '!' unary | primary
//! primary  := '(' expr ')' | atom
//! atom     := name '!=' value | name '=' value | name
//! ```

use hick_exec::state::MultiDocumentState;

/// A condition expression that can be compound.
#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    /// Variable is defined and non-empty.
    Defined(String),
    /// Variable equals value.
    Equals(String, String),
    /// Variable does not equal value.
    NotEquals(String, String),
    /// Logical AND of two conditions.
    And(Box<Condition>, Box<Condition>),
    /// Logical OR of two conditions.
    Or(Box<Condition>, Box<Condition>),
    /// Logical NOT of a condition.
    Not(Box<Condition>),
}

impl Condition {
    /// Parse a condition string.
    ///
    /// Supported syntax:
    /// - `var_name` — true if defined and non-empty
    /// - `!var_name` — true if undefined or empty
    /// - `var_name=value` — true if equals
    /// - `var_name!=value` — true if not equals
    /// - `a && b` — true if both a and b are true
    /// - `a || b` — true if either a or b is true
    /// - `!(expr)` — negation of expression
    /// - `(expr)` — grouping
    pub fn parse(s: &str) -> Self {
        let mut parser = Parser::new(s);
        parser.parse_expr()
    }

    /// Evaluate against resolved variables.
    pub fn evaluate(&self, state: &MultiDocumentState) -> bool {
        match self {
            Condition::Defined(name) => state.resolve_var(name).is_some_and(|v| !v.is_empty()),
            Condition::Equals(name, expected) => {
                state.resolve_var(name).as_deref() == Some(expected.as_str())
            }
            Condition::NotEquals(name, expected) => {
                state.resolve_var(name).as_deref() != Some(expected.as_str())
            }
            Condition::And(left, right) => left.evaluate(state) && right.evaluate(state),
            Condition::Or(left, right) => left.evaluate(state) || right.evaluate(state),
            Condition::Not(inner) => !inner.evaluate(state),
        }
    }
}

/// Recursive descent parser for condition expressions.
struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn remaining(&self) -> &'a str {
        &self.input[self.pos..]
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() {
            let ch = self.input[self.pos..].chars().next().unwrap();
            if ch.is_whitespace() {
                self.pos += ch.len_utf8();
            } else {
                break;
            }
        }
    }

    fn consume(&mut self, expected: &str) -> bool {
        self.skip_whitespace();
        if self.remaining().starts_with(expected) {
            self.pos += expected.len();
            true
        } else {
            false
        }
    }

    /// Parse top-level expression (entry point).
    fn parse_expr(&mut self) -> Condition {
        self.parse_or_expr()
    }

    /// Parse OR expression: and_expr ('||' and_expr)*
    fn parse_or_expr(&mut self) -> Condition {
        let mut left = self.parse_and_expr();
        while self.consume("||") {
            let right = self.parse_and_expr();
            left = Condition::Or(Box::new(left), Box::new(right));
        }
        left
    }

    /// Parse AND expression: unary ('&&' unary)*
    fn parse_and_expr(&mut self) -> Condition {
        let mut left = self.parse_unary();
        while self.consume("&&") {
            let right = self.parse_unary();
            left = Condition::And(Box::new(left), Box::new(right));
        }
        left
    }

    /// Parse unary expression: '!' unary | primary
    fn parse_unary(&mut self) -> Condition {
        self.skip_whitespace();
        // Check for '!' but not '!=' (which is part of an atom)
        if self.remaining().starts_with('!')
            && !self.remaining().starts_with("!=")
            && !self.is_negation_part_of_atom()
        {
            self.pos += 1; // consume '!'
            let inner = self.parse_unary();
            Condition::Not(Box::new(inner))
        } else {
            self.parse_primary()
        }
    }

    /// Check if a '!' at current position is part of an atom (like `!var` with no operators).
    /// Returns true if what follows '!' looks like a simple identifier (no parens, no operators).
    fn is_negation_part_of_atom(&self) -> bool {
        // If '!' is followed by '(' it's definitely a NOT operator
        let after_bang = &self.remaining()[1..].trim_start();
        if after_bang.starts_with('(') {
            return false;
        }
        // If there are any operators after the identifier, it's a NOT operator
        // Otherwise, treat `!var` as a legacy NotDefined shorthand
        let has_operators =
            after_bang.contains("&&") || after_bang.contains("||") || after_bang.contains('(');
        !has_operators
    }

    /// Parse primary expression: '(' expr ')' | atom
    fn parse_primary(&mut self) -> Condition {
        self.skip_whitespace();
        if self.consume("(") {
            let expr = self.parse_expr();
            self.consume(")"); // best-effort consume closing paren
            expr
        } else {
            self.parse_atom()
        }
    }

    /// Parse atom: name ('!=' | '=') value | name
    /// Also handles legacy `!var` syntax for backwards compatibility.
    fn parse_atom(&mut self) -> Condition {
        self.skip_whitespace();

        // Handle legacy `!var` syntax (NotDefined shorthand)
        if self.remaining().starts_with('!') && !self.remaining().starts_with("!=") {
            self.pos += 1;
            let name = self.parse_identifier();
            return Condition::Not(Box::new(Condition::Defined(name)));
        }

        let name = self.parse_identifier();

        self.skip_whitespace();
        if self.consume("!=") {
            let value = self.parse_value();
            Condition::NotEquals(name, value)
        } else if self.consume("=") {
            let value = self.parse_value();
            Condition::Equals(name, value)
        } else {
            Condition::Defined(name)
        }
    }

    /// Parse an identifier (variable name).
    fn parse_identifier(&mut self) -> String {
        self.skip_whitespace();
        let start = self.pos;
        while self.pos < self.input.len() {
            let ch = self.input[self.pos..].chars().next().unwrap();
            // Allow alphanumeric, underscore, hyphen in identifiers
            if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                self.pos += ch.len_utf8();
            } else {
                break;
            }
        }
        self.input[start..self.pos].to_string()
    }

    /// Parse a value (everything until next operator or end).
    fn parse_value(&mut self) -> String {
        self.skip_whitespace();
        let start = self.pos;
        while self.pos < self.input.len() {
            let remaining = &self.input[self.pos..];
            // Stop at operators or end
            if remaining.starts_with("&&")
                || remaining.starts_with("||")
                || remaining.starts_with(')')
            {
                break;
            }
            let ch = remaining.chars().next().unwrap();
            if ch.is_whitespace() {
                // Check if whitespace is followed by an operator
                let after_ws = remaining.trim_start();
                if after_ws.starts_with("&&")
                    || after_ws.starts_with("||")
                    || after_ws.starts_with(')')
                {
                    break;
                }
            }
            self.pos += ch.len_utf8();
        }
        self.input[start..self.pos].trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Parsing tests
    // -------------------------------------------------------------------------

    #[test]
    fn parse_defined() {
        assert!(matches!(
            Condition::parse("debug"),
            Condition::Defined(n) if n == "debug"
        ));
    }

    #[test]
    fn parse_not_defined() {
        // Legacy syntax: !var is shorthand for Not(Defined(var))
        let cond = Condition::parse("!debug");
        assert!(matches!(
            cond,
            Condition::Not(inner) if matches!(*inner, Condition::Defined(ref n) if n == "debug")
        ));
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
    fn parse_and() {
        let cond = Condition::parse("auth && billing");
        assert!(matches!(cond, Condition::And(_, _)));
        if let Condition::And(left, right) = cond {
            assert!(matches!(*left, Condition::Defined(ref n) if n == "auth"));
            assert!(matches!(*right, Condition::Defined(ref n) if n == "billing"));
        }
    }

    #[test]
    fn parse_or() {
        let cond = Condition::parse("auth || admin");
        assert!(matches!(cond, Condition::Or(_, _)));
        if let Condition::Or(left, right) = cond {
            assert!(matches!(*left, Condition::Defined(ref n) if n == "auth"));
            assert!(matches!(*right, Condition::Defined(ref n) if n == "admin"));
        }
    }

    #[test]
    fn parse_not_with_parens() {
        let cond = Condition::parse("!(auth && billing)");
        assert!(matches!(cond, Condition::Not(_)));
        if let Condition::Not(inner) = cond {
            assert!(matches!(*inner, Condition::And(_, _)));
        }
    }

    #[test]
    fn parse_precedence_and_over_or() {
        // a || b && c should parse as a || (b && c)
        let cond = Condition::parse("a || b && c");
        assert!(matches!(cond, Condition::Or(_, _)));
        if let Condition::Or(left, right) = cond {
            assert!(matches!(*left, Condition::Defined(ref n) if n == "a"));
            assert!(matches!(*right, Condition::And(_, _)));
        }
    }

    #[test]
    fn parse_parens_override_precedence() {
        // (a || b) && c should parse as (a || b) && c
        let cond = Condition::parse("(a || b) && c");
        assert!(matches!(cond, Condition::And(_, _)));
        if let Condition::And(left, right) = cond {
            assert!(matches!(*left, Condition::Or(_, _)));
            assert!(matches!(*right, Condition::Defined(ref n) if n == "c"));
        }
    }

    #[test]
    fn parse_complex_expression() {
        // auth && billing || admin && super
        // Should parse as: (auth && billing) || (admin && super)
        let cond = Condition::parse("auth && billing || admin && super");
        assert!(matches!(cond, Condition::Or(_, _)));
        if let Condition::Or(left, right) = cond {
            assert!(matches!(*left, Condition::And(_, _)));
            assert!(matches!(*right, Condition::And(_, _)));
        }
    }

    #[test]
    fn parse_equals_with_and() {
        let cond = Condition::parse("env=prod && debug");
        assert!(matches!(cond, Condition::And(_, _)));
        if let Condition::And(left, right) = cond {
            assert!(matches!(*left, Condition::Equals(ref n, ref v) if n == "env" && v == "prod"));
            assert!(matches!(*right, Condition::Defined(ref n) if n == "debug"));
        }
    }

    #[test]
    fn parse_chained_and() {
        let cond = Condition::parse("a && b && c");
        // Should be left-associative: ((a && b) && c)
        assert!(matches!(cond, Condition::And(_, _)));
        if let Condition::And(left, right) = cond {
            assert!(matches!(*left, Condition::And(_, _)));
            assert!(matches!(*right, Condition::Defined(ref n) if n == "c"));
        }
    }

    #[test]
    fn parse_chained_or() {
        let cond = Condition::parse("a || b || c");
        // Should be left-associative: ((a || b) || c)
        assert!(matches!(cond, Condition::Or(_, _)));
        if let Condition::Or(left, right) = cond {
            assert!(matches!(*left, Condition::Or(_, _)));
            assert!(matches!(*right, Condition::Defined(ref n) if n == "c"));
        }
    }

    // -------------------------------------------------------------------------
    // Evaluation tests
    // -------------------------------------------------------------------------

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

    #[test]
    fn evaluate_and() {
        let state = MultiDocumentState::default();
        state.set_param("auth".into(), "1".into());
        state.set_param("billing".into(), "1".into());

        assert!(Condition::parse("auth && billing").evaluate(&state));
        assert!(!Condition::parse("auth && missing").evaluate(&state));
        assert!(!Condition::parse("missing && billing").evaluate(&state));
    }

    #[test]
    fn evaluate_or() {
        let state = MultiDocumentState::default();
        state.set_param("auth".into(), "1".into());

        assert!(Condition::parse("auth || billing").evaluate(&state));
        assert!(Condition::parse("billing || auth").evaluate(&state));
        assert!(!Condition::parse("billing || missing").evaluate(&state));
    }

    #[test]
    fn evaluate_complex() {
        let state = MultiDocumentState::default();
        state.set_param("auth".into(), "1".into());
        state.set_param("billing".into(), "1".into());

        // (auth && billing) is true
        assert!(Condition::parse("auth && billing").evaluate(&state));

        // !(auth && billing) is false
        assert!(!Condition::parse("!(auth && billing)").evaluate(&state));

        // auth && billing || admin is true (first and-clause is true)
        assert!(Condition::parse("auth && billing || admin").evaluate(&state));

        // !auth || billing is true (billing is true)
        assert!(Condition::parse("!auth || billing").evaluate(&state));
    }

    #[test]
    fn evaluate_with_equals_in_compound() {
        let state = MultiDocumentState::default();
        state.set_param("env".into(), "prod".into());
        state.set_param("auth".into(), "1".into());

        assert!(Condition::parse("env=prod && auth").evaluate(&state));
        assert!(!Condition::parse("env=dev && auth").evaluate(&state));
        assert!(Condition::parse("env=dev || auth").evaluate(&state));
    }

    #[test]
    fn evaluate_nested_not() {
        let state = MultiDocumentState::default();
        state.set_param("debug".into(), "1".into());

        // !!debug should be true (double negation)
        assert!(Condition::parse("!(!debug)").evaluate(&state));

        // !(!missing) should be false
        assert!(!Condition::parse("!(!missing)").evaluate(&state));
    }
}
