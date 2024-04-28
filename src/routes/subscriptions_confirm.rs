use actix_web::{web, HttpResponse, ResponseError};
use anyhow::Context;
use reqwest::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}
#[tracing::instrument(name = "Confirm a pending subscriber", skip(parameters))]
pub async fn confirm(
    parameters: web::Query<Parameters>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ConfirmSubscriberError> {
    let id = get_subscriber_id_from_token(&pool, &parameters.subscription_token)
        .await
        .context("failed to acquire subscriber id from database")?;
    match id {
        // Non-existing token!
        None => Err(ConfirmSubscriberError::InvalidTokenError),
        Some(subscriber_id) => {
            confirm_subscriber(&pool, subscriber_id)
                .await
                .context("failed to update status of subscriber in the database")?;
            Ok(HttpResponse::Ok().finish())
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ConfirmSubscriberError {
    #[error("Invalid token")]
    InvalidTokenError,
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}
impl ResponseError for ConfirmSubscriberError {
    fn status_code(&self) -> StatusCode {
        match self {
            ConfirmSubscriberError::InvalidTokenError => StatusCode::UNAUTHORIZED,
            ConfirmSubscriberError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[tracing::instrument(name = "Mark subscriber as confirmed", skip(subscriber_id, pool))]
pub async fn confirm_subscriber(pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id,
    )
    .execute(pool)
    .await?;
    // .map_err(|e| {
    //     tracing::error!("Failed to execute query: {:?}", e);
    //     e
    // })?;
    Ok(())
}
#[tracing::instrument(name = "Get subscriber_id from token", skip(subscription_token, pool))]
pub async fn get_subscriber_id_from_token(
    pool: &PgPool,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT subscriber_id FROM subscription_tokens \
    WHERE subscription_token = $1",
        subscription_token,
    )
    .fetch_optional(pool)
    .await?;
    // .map_err(|e| {
    // tracing::error!("Failed to execute query: {:?}", e);
    // e
    // })?;
    Ok(result.map(|r| r.subscriber_id))
}
