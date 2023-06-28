use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct SubscribeData {
    name: String,
    email: String,
}

impl TryFrom<SubscribeData> for NewSubscriber {
    type Error = String;

    fn try_from(value: SubscribeData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;

        Ok(NewSubscriber { name, email })
    }
}

#[tracing::instrument(
    name = "Adding new subscriber.",
    skip(subscribe_data, connection),
    fields(
        subscriber_name = %subscribe_data.name,
        subscriber_email = %subscribe_data.email
    )
)]
pub async fn subscribe(
    subscribe_data: web::Form<SubscribeData>,
    connection: web::Data<PgPool>,
) -> impl Responder {
    let new_subscriber: NewSubscriber = match subscribe_data.0.try_into() {
        Ok(new_subscriber) => new_subscriber,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    match insert_subscriber(&connection, &new_subscriber).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details to the database!",
    skip(new_subscriber, connection)
)]
pub async fn insert_subscriber(
    connection: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, name, email, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'confirmed')
        "#,
        Uuid::new_v4(),
        new_subscriber.name.as_ref(),
        new_subscriber.email.as_ref(),
        Utc::now()
    )
    .execute(connection)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query {:?}!", e);
        e
    })?;

    Ok(())
}
