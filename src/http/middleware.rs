use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde_json::Value;
use std::collections::HashMap;

pub async fn get_user_id(token: &str) -> Result<i64, String> {
    let secret = "secret";
    let validation = Validation::default();

    match decode::<HashMap<String, Value>>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &validation,
    ) {
        Ok(token_data) => {
            if let Some(user_id) = token_data.claims.get("user_id") {
                if let Some(user_id) = user_id.as_i64() {
                    Ok(user_id)
                } else {
                    Err("Invalid user_id type".to_string())
                }
            } else {
                Err("user_id not found in token".to_string())
            }
        }
        Err(err) => Err(format!("Invalid token: {}", err)),
    }
}

pub async fn token_middleware<B>(mut req: Request<B>, next: Next<B>) -> impl IntoResponse {
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                match get_user_id(token).await {
                    Ok(user_id) => {
                        req.extensions_mut().insert(user_id);
                        return next.run(req).await;
                    }
                    Err(err) => {
                        eprintln!("Token validation failed: {}", err);
                    }
                }
            }
        }
    }
    (StatusCode::UNAUTHORIZED, "Invalid or missing token").into_response()
}
