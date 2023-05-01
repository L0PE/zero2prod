use env_logger::Env;
use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::configuration::Settings;
use zero2prod::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = Settings::new().expect("Failed to read configuration");

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address).expect("Failed to bind a random port");

    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let database_pool = PgPool::connect(&configuration.database_settings.get_connection_string())
        .await
        .expect("Unable to connect to the database");

    run(listener, database_pool)?.await
}
