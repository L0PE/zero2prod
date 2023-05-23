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
        .connect_lazy_with(
            configuration.database_settings.get_connection_options_with_db()
        );

    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    run(listener, database_pool)?.await
}
