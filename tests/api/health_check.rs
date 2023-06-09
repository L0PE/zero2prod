use crate::helpers::spawn_app;

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
