use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use std::result;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}
#[tracing::instrument(name = "Confirm a pending subscriber", skip(parameters, connection))]
pub async fn confirm(
    parameters: web::Query<Parameters>,
    connection: web::Data<PgPool>,
) -> HttpResponse {
    let id = match get_subscriber_id_from_token(&connection, &parameters.subscription_token).await {
        Ok(id) => id,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    return match id {
        None => HttpResponse::Unauthorized().finish(),
        Some(subscriber_id) => {
            if confirm_subscriber(&connection, &subscriber_id)
                .await
                .is_err()
            {
                return HttpResponse::InternalServerError().finish();
            }

            HttpResponse::Ok().finish()
        }
    };
}

#[tracing::instrument(
    name = "Get subscriber_id from token",
    skip(connection, subscription_token)
)]
async fn get_subscriber_id_from_token(
    connection: &PgPool,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1",
        subscription_token
    )
    .fetch_optional(connection)
    .await
    .map_err(|error| {
        tracing::error!("Failed to execute query {:?}!", error);
        error
    })?;

    Ok(result.map(|record| record.subscriber_id))
}

#[tracing::instrument(
    name = "Mark subscriber as confirmed",
    skip(connection, subscription_id)
)]
async fn confirm_subscriber(
    connection: &PgPool,
    subscription_id: &Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscription_id
    )
    .execute(connection)
    .await
    .map_err(|error| {
        tracing::error!("Failed to execute query {:?}!", error);
        error
    })?;

    Ok(())
}
