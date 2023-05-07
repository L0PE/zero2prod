use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct SubscribeData {
    name: String,
    email: String,
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
    match insert_subscriber(&connection, &subscribe_data).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details to the database!",
    skip(subscribe_data, connection)
)]
pub async fn insert_subscriber(
    connection: &PgPool,
    subscribe_data: &SubscribeData,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, name, email, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        subscribe_data.name,
        subscribe_data.email,
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
