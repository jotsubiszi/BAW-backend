mod admin_handler;
mod collection_handler;
mod config;
mod login_handler;
mod middleware;
mod models;

use axum::{
    Router,
    extract::Path,
    http::{Method, StatusCode},
    response::Json,
    routing::{delete, get, patch},
};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::postgres::PgPoolOptions;
use std::env;
use tcgdex_api::{CardBrief, Query, Tcgdex};
use tower_http::cors::{Any, CorsLayer};

use crate::{admin_handler::*, collection_handler::*, login_handler::*};

#[tokio::main]
async fn main() {
    let db_url = env::var("DATABASE_URL").expect("Brak zmiennej DATABASE_URL!");

    // utworzenie puli połączeń
    let pool = PgPoolOptions::new()
        .max_connections(5) //Set maximum number of connections that this pool should maintain.
        .connect(&db_url)
        .await
        .expect("Nie udało się połączyć z bazą...");

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any);

    let app = Router::new()
        .route("/pokeapi/{poke}", get(rustemon_api_handler))
        .route("/tcgapi/{poke}", get(tcg_api_handler))
        .route("/users/{id}", get(get_user))
        .route("/auth/clerk", get(get_profile))
        .route(
            "/api/collection",
            get(get_user_collection).post(add_card_to_collection),
        )
        .route("/api/admin/users", get(get_all_users))
        .route("/api/admin/users/{clerk_id}", delete(delete_user))
        .route("/api/admin/users/{clerk_id}/role", patch(update_user_role))
        .route(
            "/api/admin/users/{clerk_id}/collection",
            get(get_user_collection_admin),
        )
        .route(
            "/api/admin/users/{clerk_id}/collection/{card_id}",
            delete(remove_card_from_user),
        )
        .with_state(pool)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3333").await.unwrap();

    println!("Server works on 0.0.0.0:3333");
    axum::serve(listener, app).await.unwrap();
}

#[derive(serde::Serialize, Deserialize)]
pub struct CardBriefResponse {
    pub id: String,
    pub local_id: String,
    pub name: String,
    pub image: String,
}

impl From<CardBrief> for CardBriefResponse {
    fn from(c: CardBrief) -> Self {
        Self {
            id: c.id,
            local_id: c.local_id,
            name: c.name,
            image: c.image,
        }
    }
}

async fn rustemon_api_handler(
    Path(poke_name): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let rustemon_client = rustemon::client::RustemonClient::default();
    let pokemone = rustemon::pokemon::pokemon::get_by_name(&poke_name, &rustemon_client).await;

    match pokemone {
        Ok(p) => Ok(Json(json!(p))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e))),
    }
}

async fn tcg_api_handler(
    Path(poke_card_name): Path<String>,
) -> Result<Json<Vec<CardBriefResponse>>, (StatusCode, String)> {
    let result = tokio::task::spawn_blocking(move || {
        let tcgdex = Tcgdex::new();
        let filter_str = format!("name={}", &poke_card_name);
        let filter = Query::new().with_filtering(vec![filter_str.as_str()]);

        tcgdex.cards().fetch::<Vec<CardBrief>>(Some(&filter))
    })
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Join error: {}", e),
        )
    })?;

    match result {
        Ok(c) => Ok(Json(c.into_iter().map(CardBriefResponse::from).collect())),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Fetch error: {}", e),
        )),
    }
}
