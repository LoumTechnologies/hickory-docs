use crate::env::ShellEnvironment;

/// Expand a word value, performing variable substitution.
///
/// Phase 0 supports:
/// - Literal text (no expansion needed)
/// - `$VAR` and `${VAR}` simple variable references
/// - `$?` for last exit code
pub fn expand_word(word: &str, env: &ShellEnvironment) -> String {
    if !word.contains('$') && !word.contains('\\') && !word.contains('\'') && !word.contains('"') {
        return word.to_string();
    }

    let mut result = String::with_capacity(word.len());
    let mut chars = word.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '$' => {
                if chars.peek() == Some(&'{') {
                    // ${VAR} form
                    chars.next(); // consume '{'
                    let var_name: String = chars.by_ref().take_while(|&c| c != '}').collect();
                    result.push_str(&resolve_var(&var_name, env));
                } else {
                    // $VAR form — use peek to avoid consuming the delimiter
                    let mut var_name = String::new();
                    while let Some(&c) = chars.peek() {
                        if c.is_alphanumeric() || c == '_' || c == '?' {
                            var_name.push(c);
                            chars.next();
                            // $? is always exactly one char
                            if c == '?' {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    if var_name.is_empty() {
                        result.push('$');
                    } else {
                        result.push_str(&resolve_var(&var_name, env));
                    }
                }
            }
            '\'' => {
                // Single-quoted: literal, no expansion
                for c in chars.by_ref() {
                    if c == '\'' {
                        break;
                    }
                    result.push(c);
                }
            }
            '"' => {
                // Double-quoted: expand variables but preserve spaces
                while let Some(&c) = chars.peek() {
                    if c == '"' {
                        chars.next();
                        break;
                    } else if c == '$' {
                        chars.next();
                        if chars.peek() == Some(&'{') {
                            chars.next();
                            let var_name: String =
                                chars.by_ref().take_while(|&c| c != '}').collect();
                            result.push_str(&resolve_var(&var_name, env));
                        } else {
                            let mut var_name = String::new();
                            while let Some(&c) = chars.peek() {
                                if c.is_alphanumeric() || c == '_' || c == '?' {
                                    var_name.push(c);
                                    chars.next();
                                    if c == '?' {
                                        break;
                                    }
                                } else {
                                    break;
                                }
                            }
                            if var_name.is_empty() {
                                result.push('$');
                            } else {
                                result.push_str(&resolve_var(&var_name, env));
                            }
                        }
                    } else if c == '\\' {
                        chars.next();
                        if let Some(&next) = chars.peek() {
                            match next {
                                '$' | '"' | '\\' | '`' => {
                                    result.push(next);
                                    chars.next();
                                }
                                _ => result.push('\\'),
                            }
                        }
                    } else {
                        result.push(c);
                        chars.next();
                    }
                }
            }
            '\\' => {
                if let Some(next) = chars.next() {
                    result.push(next);
                } else {
                    result.push('\\');
                }
            }
            _ => result.push(ch),
        }
    }

    result
}

fn resolve_var(name: &str, env: &ShellEnvironment) -> String {
    if name == "?" {
        return env.last_exit_code.to_string();
    }
    env.get_var(name).unwrap_or("").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn make_env() -> ShellEnvironment {
        let mut vars = HashMap::new();
        vars.insert("HOME".to_string(), "/home/test".to_string());
        vars.insert("NAME".to_string(), "world".to_string());
        ShellEnvironment::new(PathBuf::from("/tmp"), &vars)
    }

    #[test]
    fn literal_word() {
        let env = make_env();
        assert_eq!(expand_word("hello", &env), "hello");
    }

    #[test]
    fn simple_var() {
        let env = make_env();
        assert_eq!(expand_word("$NAME", &env), "world");
    }

    #[test]
    fn braced_var() {
        let env = make_env();
        assert_eq!(expand_word("${NAME}", &env), "world");
    }

    #[test]
    fn var_in_string() {
        let env = make_env();
        assert_eq!(expand_word("hello $NAME!", &env), "hello world!");
    }

    #[test]
    fn undefined_var() {
        let env = make_env();
        assert_eq!(expand_word("$UNDEFINED", &env), "");
    }

    #[test]
    fn exit_code_var() {
        let mut env = make_env();
        env.last_exit_code = 42;
        assert_eq!(expand_word("$?", &env), "42");
    }

    #[test]
    fn single_quoted() {
        let env = make_env();
        assert_eq!(expand_word("'$NAME'", &env), "$NAME");
    }

    #[test]
    fn double_quoted_expands() {
        let env = make_env();
        assert_eq!(expand_word("\"hello $NAME\"", &env), "hello world");
    }
}
