use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use zero2prod::configuration::Settings;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = Settings::new().expect("Failed to read configuration");

    let address = format!(
        "{}:{}",
        configuration.application_settings.host, configuration.application_settings.port
    );
    let listener = TcpListener::bind(address).expect("Failed to bind a random port");

    let database_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy(
            &configuration
                .database_settings
                .get_connection_string()
                .expose_secret(),
        )
        .expect("Unable to connect to the database");

    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    run(listener, database_pool)?.await
}
