use serde_json;
use toml::Value as TomlValue;

use crate::QuartzResult;

/// Validator for files that don't have to do any checks. It is
/// garanteed to return [`Ok`].
///
/// # Examples
///
/// ```
/// use quartz_cli::validator;
///
/// // Totally broken JSON input
/// let input = r#"
/// {
///     "value":
/// "#;
///
/// assert!(validator::infallible(input).is_ok());
/// ```
pub fn infallible(_input: &str) -> QuartzResult {
    Ok(())
}

/// Checks if a string is valid TOML.
///
/// Alias for `toml_as::<toml::Value>()`.
///
/// # Examples
///
/// ```
/// use quartz_cli::validator;
///
/// let input = r#"
/// {
///     "value": 10
/// }
/// "#;
///
/// assert!(validator::json(input).is_ok());
///
/// let input = r#"
/// {
///     "value": 10,
/// }
/// "#;
///
/// assert!(validator::json(input).is_err());
/// ```
pub fn json(input: &str) -> QuartzResult {
    serde_json::from_str::<serde_json::Value>(input)?;

    Ok(())
}

/// Checks if a string is valid TOML.
///
/// Alias for `toml_as::<toml::Value>()`.
///
/// # Examples
///
/// ```
/// use quartz_cli::validator;
///
/// let input = r#"
///     title = 'TOML Example'
///
///     [owner]
///     name = 'Lisa'
/// "#;
///
/// assert!(validator::toml(input).is_ok());
/// ```
pub fn toml(input: &str) -> QuartzResult {
    toml_as::<TomlValue>(input)
}

/// Checks if a string is valid TOML for `T`.
///
/// # Examples
///
/// ```
/// use serde::Deserialize;
/// use quartz_cli::validator;
///
/// #[derive(Deserialize)]
/// struct Config {
///     title: String,
///     owner: Owner,
/// }
///
/// #[derive(Deserialize)]
/// struct Owner {
///     name: String,
/// }
///
/// let input = r#"
///     title = 'TOML Example'
///
///     [owner]
///     name = 'Lisa'
/// "#;
///
/// let input_missing = r#"
///     title = 'TOML Example'
///
///     [owner]
/// "#;
///
///
/// assert!(validator::toml_as::<Config>(input).is_ok());
/// assert!(validator::toml_as::<Config>(input_missing).is_err());
/// ```
pub fn toml_as<T>(input: &str) -> QuartzResult
where
    T: serde::de::DeserializeOwned,
{
    toml::from_str::<T>(input)?;

    Ok(())
}
