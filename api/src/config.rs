use eyre::{Context as _, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Configuration {
    #[serde(default)]
    pub env: Environment,

    pub database_url: String,
}

impl Configuration {
    /// Loads the configuration from environment variables, and configuration files.
    pub fn load() -> Result<Self> {
        let mut cfg =
            config::Config::builder().add_source(config::Environment::with_prefix("FHIR"));

        if let Ok(env) = std::env::var("FHIR_CONFIG_FILE") {
            cfg = cfg.add_source(config::File::with_name(&env));
        }

        let cfg = cfg
            .build()
            .wrap_err("failed to build config")?
            .try_deserialize::<Self>()
            .wrap_err("failed to deserialize config")?;

        Ok(cfg)
    }

    #[inline]
    pub fn is_production(&self) -> bool {
        self.env == Environment::Production
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Environment {
    #[serde(rename = "development")]
    Development,

    #[serde(rename = "production")]
    #[default]
    Production,
}
