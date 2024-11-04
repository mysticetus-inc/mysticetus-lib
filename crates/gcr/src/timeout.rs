use std::sync::OnceLock;

// using a macro instead of a const so we can statically concat in the log messages
macro_rules! timeout_env_var {
    () => {
        "CLOUD_RUN_TIMEOUT_SECONDS"
    };
}

/// Gets the cloud run request timeout from the environment.
/// Lazily initializes the value, so subsequent calls will return the cached value.
///
/// Returns [`None`] if the value is missing or invalid. If the value is invalid, a warning will be
/// emitted (once).
pub fn get() -> Option<timestamp::Duration> {
    fn read_timeout() -> Option<timestamp::Duration> {
        let var = match std::env::var(timeout_env_var!()) {
            Ok(var) => var,
            Err(std::env::VarError::NotPresent) => return None,
            Err(std::env::VarError::NotUnicode(invalid)) => {
                tracing::warn!(
                    message = concat!("invalid unicode found in '", timeout_env_var!(), "'"),
                    ?invalid
                );
                return None;
            }
        };

        match var.parse() {
            Ok(seconds) => Some(timestamp::Duration::from_seconds(seconds)),
            Err(error) => {
                tracing::warn!(message = concat!("error parsing value in '", timeout_env_var!(), "'"), %var, ?error);
                None
            }
        }
    }

    static TIMEOUT_FROM_ENV: OnceLock<Option<timestamp::Duration>> = OnceLock::new();

    *TIMEOUT_FROM_ENV.get_or_init(read_timeout)
}
