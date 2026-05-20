use axum::{Json, extract::State, http::StatusCode};
use sqlx::PgPool;

use crate::{CardBriefResponse, middleware::AuthenticatedUser};

// Ta struktura reprezentuje kartę w kolekcji konkretnego użytkownika.
// Dzięki #[derive(Serialize)], Axum automatycznie zamieni to na JSON dla Reacta.
#[derive(serde::Serialize)]
pub struct UserCardDto {
    pub id: String,               // ID karty (np. "base1-4")
    pub local_id: Option<String>, // Numer w secie (Option na wypadek starych danych w bazie)
    pub name: String,             // Nazwa karty
    pub image: Option<String>,    // Zmienione z image_url na image
}

pub async fn add_card_to_collection(
    State(pool): State<PgPool>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<CardBriefResponse>,
) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query!(
        r#"
        INSERT INTO users (clerk_id, email)
        VALUES ($1, $2)
        ON CONFLICT (clerk_id) DO NOTHING
        "#,
        auth_user.clerk_id,
        auth_user.email
    )
    .execute(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Błąd weryfikacji konta w bazie".to_string(),
        )
    })?;

    sqlx::query!(
        r#"
        INSERT INTO cards (id, name, image)
        VALUES ($1, $2, $3)
        ON CONFLICT (id) DO NOTHING
        "#,
        payload.id,
        payload.name,
        payload.image
    )
    .execute(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Błąd podczas zapisu karty".to_string(),
        )
    })?;

    // KROK 2: Przypisujemy kartę do użytkownika w `user_cards`
    // Używamy subquery, żeby wyciągnąć jego `id` z tabeli `users` na podstawie `clerk_id`
    let result = sqlx::query!(
        r#"
        INSERT INTO user_cards (user_id, card_id)
        VALUES (
            (SELECT id FROM users WHERE clerk_id = $1), 
            $2
        )
        ON CONFLICT (user_id, card_id) DO NOTHING
        "#,
        auth_user.clerk_id,
        payload.id
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        eprintln!("Błąd przypisywania karty: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Błąd przypisywania karty do usera".to_string(),
        )
    })?;

    // Sprawdzamy, czy wiersz faktycznie się dodał (czy może już go miał)
    if result.rows_affected() == 0 {
        // Możesz zwrócić 200 OK lub np. 409 Conflict, jeśli chcesz poinformować frontend: "Hej, już masz tę kartę!"
        return Ok(StatusCode::ALREADY_REPORTED); // Zwróci kod 208
    }

    // Sukces! Zwracamy kod 201 Created
    Ok(StatusCode::CREATED)
}

pub async fn get_user_collection(
    State(pool): State<PgPool>,
    auth_user: AuthenticatedUser,
) -> Result<Json<Vec<UserCardDto>>, (StatusCode, String)> {
    let cards = sqlx::query_as!(
        UserCardDto,
        r#"
        SELECT 
            c.id, 
            c.local_id, 
            c.name, 
            c.image
        FROM user_cards uc
        JOIN cards c ON uc.card_id = c.id
        JOIN users u ON uc.user_id = u.id
        WHERE u.clerk_id = $1
        ORDER BY c.name ASC -- Zmieniamy sortowanie np. na alfabetyczne po nazwie karty
        "#,
        auth_user.clerk_id // <- Wstawiamy clerk_id wyciągnięte bezpiecznie z tokena
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        eprintln!("błąd przy pobieraniu kolekcji: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "nie udało się pobrać kolekcji".to_string(),
        )
    })?;
    Ok(Json(cards))
}
