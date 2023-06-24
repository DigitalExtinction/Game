use std::fmt;

use anyhow::Error;
use de_core::fs;
use thiserror::Error as ErrorDerive;

#[derive(Debug, ErrorDerive)]
pub enum ConfigLoadError {
    DirectoryError(#[from] fs::DirError),
    CheckErrors(Vec<(String, String)>),
    Other(#[from] Error),
}

impl fmt::Display for ConfigLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Other(err) => write!(f, "Configuration error: {err}")?,
            Self::DirectoryError(err) => write!(f, "Configuration directory error: {err}")?,
            Self::CheckErrors(errors) => {
                write!(f, "Configuration validation error(s):")?;
                for err in errors {
                    write!(f, "\n  - {}", err.1)?;
                }
            }
        }

        Ok(())
    }
}

/// Bundles configuration neatly into a single struct.
///
/// The most important part is the generated `load` method which loads the
/// configuration from a file.
///
/// It also manages converting into final desired data structure.
///
/// - `name` is the name of the field in the Configuration struct
/// - `type_from` is the implementor of Config
/// - `type_into` is the final desired data structure must be able to fulfil
///   the TryInto trait
#[macro_export]
macro_rules! bundle_config {
    ($($name:ident : $type_into:ty : $type_from:ty),*) => {
        use $crate::io::load_conf_text;
        use $crate::macros::ConfigLoadError;
        use tracing::{trace, debug};
        use paste::paste;
        use bevy::prelude::Resource;
        use serde::{Deserialize as MacroDeserialize, Serialize as MacroSerialize};


        #[derive(Resource, Debug, Clone)]
        pub struct Configuration {
            $(
                $name: $type_into,
            )*
        }

        #[derive(Debug, Clone, MacroSerialize)]
        pub struct RawConfiguration {
            $(
                $name: $type_from,
            )*
        }

        impl TryInto<Configuration> for RawConfiguration {
            type Error = Error;

            fn try_into(self) -> Result<Configuration, Self::Error> {
                Ok(Configuration {
                    $(
                        $name: self.$name.try_into()?,
                    )*
                })
            }
        }

        #[derive(MacroDeserialize, Serialize, Debug, Clone, Default)]
        struct PartialConfiguration {
            $(
                $name: Option<paste! {[<Partial $type_from>]}>,
            )*
        }

        impl Configuration {
            $(
                pub fn $name(&self) -> &$type_into {
                    &self.$name
                }
            )*

            pub async fn load(path: &Path) ->  Result<Self, ConfigLoadError> {
                let from = RawConfiguration::load(path).await?;
                debug!("Loaded raw configuration: \n{}", serde_yaml::to_string(&from).expect("Failed to serialize raw configuration"));
                Ok(from.try_into().unwrap())
            }
        }

        impl RawConfiguration {
            pub async fn load(path: &Path) ->  Result<Self, ConfigLoadError> {
                match load_conf_text(path).await? {
                    Some(text) => {
                        let partial: PartialConfiguration =
                            serde_yaml::from_str(text.as_str()).context("Failed to parse DE configuration")?;
                        let config: Self = partial.try_into().context("Failed to convert partial configuration")?;

                        let mut errors = vec![];
                        $(
                            let value = &config.$name;
                            let check = value.check();
                            if let Err(err) = check {
                                for err in err {
                                    trace!("Failed check: {:?}", err);
                                    errors.push(err);
                                }
                            }
                        )*
                        if !errors.is_empty() {
                            trace!("Failed checks:");
                            for err in &errors {
                                trace!("{}", err.0);
                            }
                            return Err(ConfigLoadError::CheckErrors(errors));
                        }

                        Ok(config)
                    }
                    None => Ok(Self::default()),
                }
            }
        }

        impl TryFrom<PartialConfiguration> for RawConfiguration {
            type Error = ConfigLoadError;

            fn try_from(value: PartialConfiguration) -> Result<Self, ConfigLoadError> {
                Ok(Self {
                    $(
                        $name: match value.$name{
                            Some(v) => v.try_into().context(concat!("`", stringify!($name), "` validation failed"))?,
                            None => Default::default(),
                        },
                    )*
                })
            }
        }

        impl Default for RawConfiguration {
            fn default() -> Self {
                PartialConfiguration::default().try_into().unwrap()
            }
        }

        impl Default for Configuration {
            fn default() -> Self {
                RawConfiguration::default().try_into().unwrap()
            }
        }
    };
}
