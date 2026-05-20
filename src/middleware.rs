use axum::{
    extract::FromRequestParts,
    http::{StatusCode, header::AUTHORIZATION, request::Parts},
};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde_json::Value;

pub struct AuthenticatedUser {
    pub clerk_id: String,
    pub email: String,
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .filter(|value| value.starts_with("Bearer "))
            .ok_or((StatusCode::UNAUTHORIZED, "Brak tokenu Bearer".to_string()))?;

        let token = &auth_header[7..];

        let pem_key = include_bytes!("clerk_public_key.pem");

        let decoding_key = DecodingKey::from_rsa_pem(pem_key).map_err(|e| {
            eprintln!("Błąd klucza PEM: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Błąd konfiguracji klucza serwera".to_string(),
            )
        })?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = true;
        validation.validate_aud = false;

        // Dekodujemy do surowego Value zamiast własnej struktury
        let token_data =
            jsonwebtoken::decode::<Value>(token, &decoding_key, &validation).map_err(|e| {
                eprintln!(">>> BŁĄD DEKODOWANIA TOKENU JWT: {:?} <<<", e);
                (StatusCode::UNAUTHORIZED, "Niewazny token".to_string())
            })?;

        let claims = token_data.claims;

        let clerk_id = claims["sub"]
            .as_str()
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "Brak pola sub w tokenie".to_string(),
            ))?
            .to_string();

        let email = claims["email"]
            .as_str()
            .unwrap_or("brak_emaila@test.com")
            .to_string();

        Ok(AuthenticatedUser { clerk_id, email })
    }
}
