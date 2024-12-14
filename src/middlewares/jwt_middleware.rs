use actix_web::{
    body::BoxBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::Next,
    HttpResponse,
};

use jsonwebtoken::{decode, DecodingKey, TokenData, Validation};
use serde_json::json;

use crate::utils::jwt_token_generation::Claims;

pub async fn jwt_middleware(
    req: ServiceRequest,
    next: Next<BoxBody>,
) -> Result<ServiceResponse<BoxBody>, actix_web::Error> {
    let secret_key = match std::env::var("JWT_SECRET") {
        Ok(secret) => secret,
        Err(_) => {
            return Ok(req.into_response(
                HttpResponse::InternalServerError()
                    .json(json!({
                        "error": "Internal Server Error",
                        "message": "JWT_SECRET is not set in the environment variables"
                    }))
                    .map_into_boxed_body(),
            ));
        }
    };

    let token: String = match req.cookie("token") {
        Some(cookie) => {
            let c = cookie.value().to_string();
            c
        }
        None => {
            return Ok(req.into_response(
                HttpResponse::Unauthorized()
                    .json(json!({
                        "error": "Unauthorized",
                        "message": "No token found in cookies"
                    }))
                    .map_into_boxed_body(),
            ));
        }
    };

    if let Err(_) = validate_jwt(&token, &secret_key) {
        return Ok(req.into_response(
            HttpResponse::Unauthorized()
                .json(serde_json::json!({
                    "error": "Unauthorized",
                    "message": "Invalid or expired token"
                }))
                .map_into_boxed_body(),
        ));
    }

    next.call(req).await
}

// JWT validation function
fn validate_jwt(
    token: &str,
    secret_key: &str,
) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
    let decoding_key = DecodingKey::from_secret(secret_key.as_ref());

    let mut validation = Validation::default();

    validation.validate_exp = true;

    decode(token, &decoding_key, &validation)
}
