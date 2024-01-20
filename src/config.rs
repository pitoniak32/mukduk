use std::{env, path::PathBuf};

use thiserror::Error;

/// # Use this for reading config from Environment Variables
/// The goal with this enum is to provide a way to access typed configuration from Environment
/// variables.
///
/// This will allow the type to be validated before it is used by the program.
///
/// ## Steps to add new Environment Variables:
/// 1. Add the key name to this enum.
/// 1. Add the new variant in the `as_str` impl
///   (use the name of the env var you would like to provide).
/// 1. Implement the 'From' trait. You should implement this for the value
///   that you would like the Env Var to be read as.
///
/// ### Valid Examples
/// This is what using an env variable for a boolean would look like.
/// ```
/// use std::env;
/// use poc_rear_config_lib::config_env::ConfigEnvKey;
///
/// env::set_var(ConfigEnvKey::DevMode.as_str(), "true");
/// let is_dev_mode = bool::from(ConfigEnvKey::DevMode);
///
/// assert_eq!(is_dev_mode, true);
/// ```
///
/// And if no value is provided you can choose to add a default value.
/// ```
/// use std::env;
/// use poc_rear_config_lib::config_env::ConfigEnvKey;
///
/// // In this case the default for `ConfigEnvKey` is `false`.
/// env::remove_var(ConfigEnvKey::DevMode.as_str());
/// let is_dev_mode = bool::from(ConfigEnvKey::DevMode);
///
/// assert_eq!(is_dev_mode, false);
/// ```
/// ### Panic Examples
/// If you try to read an invalid value into your program, it *SHOULD* panic at config time.
/// ```should_panic
/// use std::env;
/// use poc_rear_config_lib::config_env::ConfigEnvKey;
///
/// // In this case the default for `ConfigEnvKey` is `false`.
/// env::set_var(ConfigEnvKey::DevMode.as_str(), "123not_bool");
/// let is_dev_mode = bool::from(ConfigEnvKey::DevMode);
/// ```
pub enum ConfigEnvKey {
    Home,
    XDGConfig,
    XDGData,
    XDGState,
}

impl ConfigEnvKey {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Home => "HOME",
            Self::XDGConfig => "XDG_CONFIG_HOME",
            Self::XDGData => "XDG_DATA_HOME",
            Self::XDGState => "XDG_STATE_HOME",
        }
    }

    pub const fn default_value(&self) -> &'static str {
        match self {
            Self::Home => "",
            Self::XDGConfig => "",
            Self::XDGData => "",
            Self::XDGState => "",
        }
    }
}

const DEFAULT_PANIC_MSG: &str =
    "Check the impl block for the type you are trying to use and make sure the key is implemented.";

/// This is what using an env variable for a String would look like.
/// ```
/// use std::env;
/// use poc_rear_config_lib::config_env::ConfigEnvKey;
///
/// // I am not using the literal here to avoid breaking tests if the name changes.
/// env::set_var(ConfigEnvKey::OtelCollectorUrl.as_str(), "tcp://localhost:4317");
///
/// let otel_col_url = String::from(ConfigEnvKey::OtelCollectorUrl);
///
/// assert_eq!(otel_col_url, "tcp://localhost:4317");
/// ```
impl TryFrom<ConfigEnvKey> for PathBuf {
    type Error = ConfigError;
    fn try_from(env_key: ConfigEnvKey) -> Result<Self, ConfigError> {
        match env_key {
            ConfigEnvKey::Home => Ok(Self::from(
                env::var(ConfigEnvKey::Home.as_str()).expect("HOME env var should be set"),
            )),
            ConfigEnvKey::XDGConfig => match env::var(ConfigEnvKey::XDGConfig.as_str()) {
                Ok(config_dir) => Ok(Self::from(config_dir)),
                Err(_err) => {
                    let mut home = Self::try_from(ConfigEnvKey::Home)?;
                    home.push(".config");
                    log::trace!(
                        "Error: error reading ${}. Using [{}]",
                        ConfigEnvKey::XDGConfig.as_str(),
                        home.as_os_str().to_string_lossy()
                    );
                    Ok(home)
                }
            },
            ConfigEnvKey::XDGData => Ok(Self::from(
                env::var(ConfigEnvKey::XDGData.as_str())
                    .expect("XDG_DATA_HOME env var should be set"),
            )),
            ConfigEnvKey::XDGState => Ok(Self::from(
                env::var(ConfigEnvKey::XDGState.as_str())
                    .expect("XDG_STATE_HOME env var should be set"),
            )),
            #[allow(unreachable_patterns)]
            // This is allowed because not all enum variants are guaranteed to be this type in the
            // futrue.
            _ => panic!("this key cannot be converted to PathBuf. {DEFAULT_PANIC_MSG}"),
        }
    }
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("genertic")]
    Generic,
}
