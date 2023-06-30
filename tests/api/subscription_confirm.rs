use crate::helpers::spawn_app;
use wiremock::matchers::{method, path};
use wiremock::{Mock, Request, ResponseTemplate};

#[tokio::test]
async fn confirmation_without_token_reject_with_400() {
    let test_app = spawn_app().await;

    let response = reqwest::get(format!("{}/subscriptions/confirm", test_app.address))
        .await
        .unwrap();

    assert_eq!(400, response.status());
}

#[tokio::test]
async fn the_link_returned_by_subscribe_returns_a_200_if_called() {
    let test_app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/v3/smtp/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    test_app.subscribe_request(body.into()).await;

    let email_request: &Request = &test_app.email_server.received_requests().await.unwrap()[0];
    let confirmation_link = test_app.get_confirmation_link(email_request);

    let response = reqwest::get(confirmation_link.html_confirmation_link)
        .await
        .unwrap();

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn clicking_on_the_confirmation_link_confirms_a_subscriber() {
    let test_app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/v3/smtp/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    test_app.subscribe_request(body.into()).await;

    let email_request: &Request = &test_app.email_server.received_requests().await.unwrap()[0];
    let confirmation_link = test_app.get_confirmation_link(email_request);

    reqwest::get(confirmation_link.html_confirmation_link)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions")
        .fetch_one(&test_app.db_poll)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!("le guin", saved.name);
    assert_eq!("ursula_le_guin@gmail.com", saved.email);
    assert_eq!("confirmed", saved.status);
}
