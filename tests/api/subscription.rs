use crate::helpers::spawn_app;
use sqlx::query;

#[tokio::test]
async fn subscribe_returns_200_for_valid_request_data() {
    let test_app = spawn_app().await;

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let response = test_app.subscribe_request(body.into()).await;

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
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = test_app.subscribe_request(invalid_body.into()).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}
#[tokio::test]
async fn subscribe_returns_400_when_fields_is_present_but_invalid() {
    let test_app = spawn_app().await;
    let test_cases = vec![
        ("name=&email=email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=le%20guin&email=", "empty email"),
        (
            "name=Ursula&email=not-valid-email",
            "missing both name and email",
        ),
    ];

    for (body, error_message) in test_cases {
        let response = test_app.subscribe_request(body.into()).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}
