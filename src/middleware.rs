use axum::{
    extract::FromRequestParts,
    http::{StatusCode, header::AUTHORIZATION, request::Parts},
};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
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
        let decoding_key = DecodingKey::from_rsa_pem(pem_key).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Błąd klucza serwera".to_string(),
            )
        })?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = true;
        validation.validate_aud = false;

        // Używamy Value, więc Rust przyjmie wszystko!
        let token_data = jsonwebtoken::dangerous::insecure_decode::<serde_json::Value>(token)
            .map_err(|e| {
                eprintln!(">>> NAWET WYMUSZONE DEKODOWANIE PADŁO: {:?} <<<", e);
                (StatusCode::UNAUTHORIZED, "Niewazny token".to_string())
            })?;

        println!(">>> SUKCES! ZAWARTOŚĆ TOKENU: {:#?} <<<", token_data.claims);

        let sub = token_data
            .claims
            .get("sub")
            .and_then(|v| v.as_str())
            .unwrap_or("brak_id")
            .to_string();
        let email = token_data
            .claims
            .get("email")
            .and_then(|v| v.as_str())
            .unwrap_or("brak_maila")
            .to_string();

        println!(">>> MIDDLEWARE PRZEPUŚCIŁ USERA: {} <<<", email);

        Ok(AuthenticatedUser {
            clerk_id: sub,
            email,
        })
    }
}
