use crate::{collection_handler::UserCardDto, middleware::AuthenticatedUser, models::users::User};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use sqlx::PgPool;

// Pomocnicza funkcja: Sprawdza, czy osoba wywołująca endpoint ma is_admin = true w bazie
async fn check_is_admin(pool: &PgPool, clerk_id: &str) -> Result<(), (StatusCode, String)> {
    let is_admin = sqlx::query_scalar!("SELECT is_admin FROM users WHERE clerk_id = $1", clerk_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Błąd bazy danych przy sprawdzaniu uprawnień".to_string(),
            )
        })?;

    match is_admin.flatten() {
        Some(true) => Ok(()),
        _ => Err((
            StatusCode::FORBIDDEN,
            "Brak uprawnień administratora".to_string(),
        )),
    }
}

// 1. Pobieranie wszystkich użytkowników
pub async fn get_all_users(
    State(pool): State<PgPool>,
    auth_user: AuthenticatedUser,
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    check_is_admin(&pool, &auth_user.clerk_id).await?;
    let users = sqlx::query_as!(
        User,
        "SELECT id, clerk_id, email, is_admin, created_at FROM users ORDER BY id ASC"
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Błąd pobierania".to_string(),
        )
    })?;
    Ok(Json(users))
}

#[derive(serde::Deserialize)]
pub struct RoleUpdate {
    pub is_admin: bool,
}

// 2. Zmiana statusu admina (Daj / Zabierz)
pub async fn update_user_role(
    State(pool): State<PgPool>,
    auth_user: AuthenticatedUser,
    Path(target_clerk_id): Path<String>,
    Json(payload): Json<RoleUpdate>,
) -> Result<StatusCode, (StatusCode, String)> {
    check_is_admin(&pool, &auth_user.clerk_id).await?;
    sqlx::query!(
        "UPDATE users SET is_admin = $1 WHERE clerk_id = $2",
        payload.is_admin,
        target_clerk_id
    )
    .execute(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Błąd aktualizacji".to_string(),
        )
    })?;
    Ok(StatusCode::OK)
}

// 3. Całkowite usunięcie użytkownika
pub async fn delete_user(
    State(pool): State<PgPool>,
    auth_user: AuthenticatedUser,
    Path(target_clerk_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    check_is_admin(&pool, &auth_user.clerk_id).await?;
    sqlx::query!("DELETE FROM users WHERE clerk_id = $1", target_clerk_id)
        .execute(&pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Błąd usuwania".to_string(),
            )
        })?;
    Ok(StatusCode::OK)
}

// 4. Przeglądanie kolekcji konkretnego gracza
pub async fn get_user_collection_admin(
    State(pool): State<PgPool>,
    auth_user: AuthenticatedUser,
    Path(target_clerk_id): Path<String>,
) -> Result<Json<Vec<UserCardDto>>, (StatusCode, String)> {
    check_is_admin(&pool, &auth_user.clerk_id).await?;
    let cards = sqlx::query_as!(
        UserCardDto,
        r#"
        SELECT c.id, c.local_id, c.name, c.image
        FROM user_cards uc
        JOIN cards c ON uc.card_id = c.id
        JOIN users u ON uc.user_id = u.id
        WHERE u.clerk_id = $1
        ORDER BY c.name ASC
        "#,
        target_clerk_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Błąd kolekcji".to_string(),
        )
    })?;
    Ok(Json(cards))
}

// 5. Usuwanie konkretnej karty z cudzej kolekcji
pub async fn remove_card_from_user(
    State(pool): State<PgPool>,
    auth_user: AuthenticatedUser,
    Path((target_clerk_id, card_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, String)> {
    check_is_admin(&pool, &auth_user.clerk_id).await?;
    sqlx::query!(
        r#"
        DELETE FROM user_cards
        WHERE card_id = $1 AND user_id = (SELECT id FROM users WHERE clerk_id = $2)
        "#,
        card_id,
        target_clerk_id
    )
    .execute(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Błąd usunięcia karty".to_string(),
        )
    })?;
    Ok(StatusCode::OK)
}
