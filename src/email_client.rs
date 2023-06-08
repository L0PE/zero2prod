use crate::domain::SubscriberEmail;
use reqwest::Client;
use secrecy::{ExposeSecret, Secret};

#[derive(Clone)]
pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail,
    api_token: Secret<String>,
}

impl EmailClient {
    pub fn new(
        base_url: String,
        sender: SubscriberEmail,
        api_token: Secret<String>,
        timeout: std::time::Duration,
    ) -> Self {
        Self {
            http_client: Client::builder().timeout(timeout).build().unwrap(),
            base_url,
            sender,
            api_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/v3/smtp/email", self.base_url);
        let request_body = SendEmailRequest {
            sender: SenderData {
                email: self.sender.as_ref(),
            },
            to: vec![ReceiverData {
                email: recipient.as_ref(),
            }],
            subject,
            html_content,
        };

        self.http_client
            .post(&url)
            .header("api-key", self.api_token.expose_secret())
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SendEmailRequest<'a> {
    sender: SenderData<'a>,
    to: Vec<ReceiverData<'a>>,
    subject: &'a str,
    html_content: &'a str,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]

struct SenderData<'a> {
    email: &'a str,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ReceiverData<'a> {
    email: &'a str,
}

#[cfg(test)]
mod test {
    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;
    use claim::{assert_err, assert_ok};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use secrecy::Secret;
    use wiremock::matchers::{any, header, header_exists, method, path};
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);

            if let Ok(body) = result {
                body.get("sender")
                    .is_some_and(|value| value.get("email").is_some())
                    && body.get("to").is_some_and(|value| value.is_array())
                    && body.get("subject").is_some()
                    && body.get("htmlContent").is_some()
            } else {
                false
            }
        }
    }

    fn get_email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    fn get_subject() -> String {
        Sentence(1..2).fake()
    }

    fn get_content() -> String {
        Paragraph(1..10).fake()
    }

    fn get_email_client(base_url: String) -> EmailClient {
        EmailClient::new(
            base_url,
            get_email(),
            Secret::new(Faker.fake()),
            std::time::Duration::from_millis(200),
        )
    }

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        let mock_server = MockServer::start().await;
        let email_client = get_email_client(mock_server.uri());

        Mock::given(header_exists("api-key"))
            .and(header("Content-Type", "application/json"))
            .and(path("/v3/smtp/email"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let _ = email_client
            .send_email(get_email(), &get_subject(), &get_content())
            .await;
    }

    #[tokio::test]
    async fn send_email_succeeds_if_the_server_returns_200() {
        let mock_server = MockServer::start().await;
        let email_client = get_email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(get_email(), &get_subject(), &get_content())
            .await;

        assert_ok!(outcome);
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        let mock_server = MockServer::start().await;
        let email_client = get_email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(get_email(), &get_subject(), &get_content())
            .await;

        assert_err!(outcome);
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_takes_to_long() {
        let mock_server = MockServer::start().await;
        let email_client = get_email_client(mock_server.uri());
        let response = ResponseTemplate::new(500).set_delay(std::time::Duration::from_secs(180));

        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(get_email(), &get_subject(), &get_content())
            .await;

        assert_err!(outcome);
    }
}
