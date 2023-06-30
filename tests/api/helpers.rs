use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use wiremock::{MockServer, Request};
use zero2prod::configuration::{DatabaseSettings, Settings};
use zero2prod::startup::{get_connection, Application};
use zero2prod::telemetry::{get_subscriber, init_subscriber};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub db_poll: PgPool,
    pub email_server: MockServer,
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let email_server = MockServer::start().await;
    let configuration = {
        let mut c: Settings = Settings::new().expect("Failed to read configuration");

        c.database_settings.database_name = Uuid::new_v4().to_string();
        c.application_settings.port = 0;
        c.email_client.base_url = email_server.uri();

        c
    };

    configure_database(&configuration.database_settings).await;

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application.");
    let application_port = application.port();

    let _ = tokio::spawn(application.run_until_stopped());

    TestApp {
        address: format!("http://127.0.0.1:{}", application_port),
        port: application_port,
        db_poll: get_connection(&configuration.database_settings),
        email_server,
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut db_connection = PgConnection::connect_with(&config.get_connection_options_without_db())
        .await
        .expect("Failed to connect to the Postgres.");

    db_connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, &config.database_name).as_str())
        .await
        .expect("Failed to create datbase.");

    let db_pool = PgPool::connect_with(config.get_connection_options_with_db())
        .await
        .expect("Failed to connect to the database.");

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to Migrate database");

    db_pool
}

impl TestApp {
    pub async fn subscribe_request(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(format!("{}/subscribe", self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to send the error")
    }

    pub fn get_confirmation_link(&self, email_request: &Request) -> ConfirmationLink {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

        let get_link = |s: &str| {
            let link: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();

            assert_eq!(link.len(), 1);

            let raw_link = link[0].as_str().to_owned();
            let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();

            assert_eq!("127.0.0.1", confirmation_link.host_str().unwrap());

            confirmation_link.set_port(Some(self.port)).unwrap();

            confirmation_link
        };

        let html_confirmation_link = get_link(&body["htmlContent"].as_str().unwrap());

        ConfirmationLink {
            html_confirmation_link,
        }
    }
}

pub struct ConfirmationLink {
    pub html_confirmation_link: reqwest::Url,
}
