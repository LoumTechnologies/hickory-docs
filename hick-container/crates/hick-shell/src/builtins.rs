use crate::env::ShellEnvironment;
use crate::expand::expand_word;

/// Result of running a builtin command.
pub struct BuiltinResult {
    pub stdout: String,
    pub exit_code: i32,
}

/// Check if a command name is a shell builtin.
pub fn is_builtin(name: &str) -> bool {
    matches!(
        name,
        "echo" | "true" | "false" | "exit" | "export" | "cd" | "set" | ":"
    )
}

/// Execute a builtin command.
pub fn run_builtin(
    name: &str,
    args: &[String],
    env: &mut ShellEnvironment,
) -> BuiltinResult {
    match name {
        "echo" => builtin_echo(args),
        "true" | ":" => BuiltinResult {
            stdout: String::new(),
            exit_code: 0,
        },
        "false" => BuiltinResult {
            stdout: String::new(),
            exit_code: 1,
        },
        "exit" => {
            let code = args
                .first()
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);
            BuiltinResult {
                stdout: String::new(),
                exit_code: code,
            }
        }
        "export" => builtin_export(args, env),
        "cd" => builtin_cd(args, env),
        "set" => builtin_set(args, env),
        _ => BuiltinResult {
            stdout: String::new(),
            exit_code: 127,
        },
    }
}

fn builtin_echo(args: &[String]) -> BuiltinResult {
    let mut no_newline = false;
    let mut start = 0;

    if args.first().map(|s| s.as_str()) == Some("-n") {
        no_newline = true;
        start = 1;
    }

    let mut output = args[start..].join(" ");
    if !no_newline {
        output.push('\n');
    }

    BuiltinResult {
        stdout: output,
        exit_code: 0,
    }
}

fn builtin_export(args: &[String], env: &mut ShellEnvironment) -> BuiltinResult {
    for arg in args {
        if let Some((name, value)) = arg.split_once('=') {
            let expanded = expand_word(value, env);
            env.set_var(name.to_string(), expanded);
            env.exports.insert(name.to_string());
        } else {
            // Export existing variable
            env.exports.insert(arg.clone());
        }
    }
    BuiltinResult {
        stdout: String::new(),
        exit_code: 0,
    }
}

fn builtin_cd(args: &[String], env: &mut ShellEnvironment) -> BuiltinResult {
    let target = if let Some(dir) = args.first() {
        std::path::PathBuf::from(dir)
    } else if let Some(home) = env.get_var("HOME") {
        std::path::PathBuf::from(home)
    } else {
        return BuiltinResult {
            stdout: String::new(),
            exit_code: 1,
        };
    };

    let new_cwd = if target.is_absolute() {
        target
    } else {
        env.cwd.join(target)
    };

    env.cwd = new_cwd;
    BuiltinResult {
        stdout: String::new(),
        exit_code: 0,
    }
}

fn builtin_set(args: &[String], env: &mut ShellEnvironment) -> BuiltinResult {
    // Minimal: `set -- arg1 arg2` sets positional params
    if args.first().map(|s| s.as_str()) == Some("--") {
        env.positional_params = args[1..].to_vec();
    }
    BuiltinResult {
        stdout: String::new(),
        exit_code: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn make_env() -> ShellEnvironment {
        ShellEnvironment::new(PathBuf::from("/tmp"), &HashMap::new())
    }

    #[test]
    fn echo_simple() {
        let mut env = make_env();
        let result = run_builtin("echo", &["hello".into(), "world".into()], &mut env);
        assert_eq!(result.stdout, "hello world\n");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn echo_no_newline() {
        let mut env = make_env();
        let result = run_builtin("echo", &["-n".into(), "hello".into()], &mut env);
        assert_eq!(result.stdout, "hello");
    }

    #[test]
    fn echo_empty() {
        let mut env = make_env();
        let result = run_builtin("echo", &[], &mut env);
        assert_eq!(result.stdout, "\n");
    }

    #[test]
    fn true_builtin() {
        let mut env = make_env();
        let result = run_builtin("true", &[], &mut env);
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn false_builtin() {
        let mut env = make_env();
        let result = run_builtin("false", &[], &mut env);
        assert_eq!(result.exit_code, 1);
    }

    #[test]
    fn exit_with_code() {
        let mut env = make_env();
        let result = run_builtin("exit", &["42".into()], &mut env);
        assert_eq!(result.exit_code, 42);
    }
}
