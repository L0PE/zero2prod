use crate::domain::SubscriberEmail;
use config::{Config, ConfigError, File};
use secrecy::ExposeSecret;
use secrecy::Secret;
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use sqlx::ConnectOptions;
use std::time::Duration;

#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub database_settings: DatabaseSettings,
    pub application_settings: ApplicationSettings,
    pub email_client: EmailClientSettings,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let base_dir = std::env::current_dir().expect("Failed to determine the current directory.");
        let configuration_directory = base_dir.join("configuration");
        let environment: Environment = std::env::var("APP_ENVIRONMENT")
            .unwrap_or_else(|_| "local".into())
            .try_into()
            .expect("Failed to parse APP_ENVIRONMENT.");

        let settings = Config::builder()
            .add_source(File::from(
                configuration_directory.join(environment.as_str()),
            ))
            .add_source(config::Environment::with_prefix("APP").separator("__"))
            .build()?;

        settings.try_deserialize()
    }
}

#[derive(serde::Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    pub fn get_connection_options_without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };

        PgConnectOptions::new()
            .host(&self.host)
            .port(self.port)
            .username(&self.username)
            .password(self.password.expose_secret())
            .ssl_mode(ssl_mode)
    }

    pub fn get_connection_options_with_db(&self) -> PgConnectOptions {
        let mut options = self
            .get_connection_options_without_db()
            .database(&self.database_name);
        options.log_statements(tracing::log::LevelFilter::Trace);
        options
    }
}

#[derive(serde::Deserialize, Clone)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct EmailClientSettings {
    pub base_url: String,
    pub sender_email: String,
    pub api_token: Secret<String>,
    pub timeout_milliseconds: u64,
}

impl EmailClientSettings {
    pub fn sender(&self) -> Result<SubscriberEmail, String> {
        SubscriberEmail::parse(self.sender_email.clone())
    }

    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout_milliseconds)
    }
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(environment: String) -> Result<Self, Self::Error> {
        match environment.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`.",
                other
            )),
        }
    }
}
