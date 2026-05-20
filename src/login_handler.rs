use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use sqlx::PgPool;

use crate::{middleware::AuthenticatedUser, models::users::User};

pub async fn get_profile(
    State(pool): State<PgPool>,
    auth_user: AuthenticatedUser, // axum SAM SPRAWDZIŁ TOKEN!
) -> Result<Json<User>, (StatusCode, String)> {
    // Zobaczysz ten napis w terminalu Rusta, jeśli tylko React poprawnie wyśle token!
    println!(">>> get_profile się odpaliło dla: {} <<<", auth_user.email);

    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (clerk_id, email)
        VALUES ($1, $2)
        ON CONFLICT (clerk_id) 
        DO UPDATE SET email = EXCLUDED.email
        RETURNING id, clerk_id, email, is_admin, created_at
        "#,
        auth_user.clerk_id,
        auth_user.email
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        // TA LINIJKA JEST KLUCZOWA! Wyrzuci na ekran dokładny powód błędu SQL
        eprintln!(">>> KRYTYCZNY BŁĄD SQL W GET_PROFILE: {:?} <<<", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Błąd bazy danych: {}", e), // Wysyłamy błąd też do Reacta!
        )
    })?;

    Ok(Json(user))
}
//
// pub async fn google_login_handler(
//     State(pool): State<PgPool>,
//     Json(payload): Json<GoogleLoginPayload>,
// ) -> Result<Json<User>, (StatusCode, String)> {
//     let user = sqlx::query_as!(
//         User,
//         r#"
//         INSERT INTO users (clerk_id, email)
//         VALUES ($1, $2)
//         ON CONFLICT (clerk_id)
//         DO UPDATE SET email = EXCLUDED.email
//         RETURNING id, clerk_id, email, is_admin, created_at
//         "#,
//         payload.clerk_id,
//         payload.email
//     )
//     .fetch_one(&pool)
//     .await;
//
//     match user {
//         Ok(user_data) => {
//             // W tym miejscu użytkownik na pewno istnieje w bazie (nowy lub stary).
//             // Tutaj możesz wygenerować swój JWT/ciasteczko dla Reacta.
//             Ok(Json(user_data))
//         }
//         Err(e) => {
//             eprintln!("Błąd podczas rejestracji/logowania Google: {:?}", e);
//             Err((
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 "Nie udało się przetworzyć logowania".to_string(),
//             ))
//         }
//     }
// }
//
pub async fn get_user(
    State(pool): State<PgPool>,
    Path(user_id): Path<i32>,
) -> Result<Json<User>, (StatusCode, String)> {
    // Wykonanie zapytania asynchronicznie z użyciem puli
    let result = sqlx::query_as!(
        User,
        "SELECT id, clerk_id, email, is_admin, created_at FROM users WHERE id = $1",
        user_id
    )
    .fetch_optional(&pool) // fetch_optional zwraca Option<User>
    .await;

    // Obsługa błędów bazy i zwracanie JSONa
    match result {
        Ok(Some(user)) => Ok(Json(user)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            format!("Użytkownik z ID {} nie został znaleziony", user_id),
        )),
        Err(e) => {
            // W środowisku produkcyjnym lepiej logować błąd, a userowi dać ogólny komunikat
            eprintln!("Błąd bazy danych: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Wewnętrzny błąd serwera".to_string(),
            ))
        }
    }
}
