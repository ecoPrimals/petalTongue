// SPDX-License-Identifier: AGPL-3.0-or-later
//! OrExit trait for zero-panic validation binaries.
//!
//! Extends `Result` and `Option` with `or_exit(context)` to log errors and exit cleanly.

/// Extends `Result` and `Option` with a method that logs the error and exits cleanly.
pub trait OrExit<T> {
    /// Returns the value, or prints an error to stderr and exits with code 1.
    fn or_exit(self, context: &str) -> T;
}

impl<T, E> OrExit<T> for Result<T, E>
where
    E: std::fmt::Display,
{
    fn or_exit(self, context: &str) -> T {
        self.unwrap_or_else(|e| {
            eprintln!("error: {context}: {e}");
            std::process::exit(1);
        })
    }
}

impl<T> OrExit<T> for Option<T> {
    fn or_exit(self, context: &str) -> T {
        self.unwrap_or_else(|| {
            eprintln!("error: {context}: value was None");
            std::process::exit(1);
        })
    }
}

#[cfg(test)]
mod tests {
    use super::OrExit;

    #[test]
    fn result_ok_returns_value() {
        let v: Result<i32, String> = Ok(42);
        assert_eq!(v.or_exit("test"), 42);
    }

    #[test]
    fn option_some_returns_value() {
        let v: Option<i32> = Some(42);
        assert_eq!(v.or_exit("test"), 42);
    }

    #[test]
    fn result_with_display_error_compiles() {
        let _: i32 = Ok::<i32, std::io::Error>(42).or_exit("io");
    }

    #[test]
    fn error_message_format() {
        let context = "loading config";
        let err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let msg = format!("error: {context}: {err}");
        assert_eq!(msg, "error: loading config: file not found");
    }
}
