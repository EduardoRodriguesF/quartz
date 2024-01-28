use toml::Value;

pub fn json(content: &str) -> Result<(), Box<dyn std::error::Error>> {
    serde_json::from_str(content)?;

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
/// let content = r#"
///     title = 'TOML Example'
///
///     [owner]
///     name = 'Lisa'
/// "#;
///
/// assert_eq!(validator::toml(content).is_ok(), true);
/// ```
pub fn toml(content: &str) -> Result<(), Box<dyn std::error::Error>> {
    toml_as::<Value>(content)
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
/// let content = r#"
///     title = 'TOML Example'
///
///     [owner]
///     name = 'Lisa'
/// "#;
///
/// let content_missing = r#"
///     title = 'TOML Example'
///
///     [owner]
/// "#;
///
///
/// assert_eq!(validator::toml_as::<Config>(content).is_ok(), true);
/// assert_eq!(validator::toml_as::<Config>(content_missing).is_err(), true);
/// ```
pub fn toml_as<T>(content: &str) -> Result<(), Box<dyn std::error::Error>>
where
    T: serde::de::DeserializeOwned,
{
    toml::from_str::<T>(content)?;

    Ok(())
}
