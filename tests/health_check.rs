use sqlx::{query, PgPool};
use std::net::TcpListener;
use zero2prod::configuration::Settings;

pub struct TestApp {
    pub address: String,
    pub db_poll: PgPool,
}
#[tokio::test]
async fn health_check_returns_expected_result() {
    let test_app = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health", &test_app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_request_data() {
    let test_app = spawn_app().await;

    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let response = client
        .post(&format!("{}/subscribe", &test_app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    let saved = query!("SELECT email, name FROM subscriptions")
        .fetch_one(&test_app.db_poll)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(200, response.status().as_u16());
    assert_eq!("le guin", saved.name);
    assert_eq!("ursula_le_guin@gmail.com", saved.email);
}

#[tokio::test]
async fn subscribe_returns_400_for_not_valid_request_data() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscribe", &test_app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind a random port");
    let port = listener.local_addr().unwrap().port();

    let configuration = Settings::new().expect("Failed to read configuration");
    let db_connection = PgPool::connect(&configuration.database_settings.get_connection_string())
        .await
        .expect("Failed to connect to the database.");
    let server =
        zero2prod::startup::run(listener, db_connection.clone()).expect("Failed to start server");

    let _ = tokio::spawn(server);

    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        db_poll: db_connection,
    }
}
