mod login_handler;
mod models;

use axum::{
    Router,
    extract::Path,
    http::{Method, StatusCode},
    response::Json,
    routing::{get, post},
};
use serde_json::{Value, json};
use sqlx::postgres::PgPoolOptions;
use std::env;
use tcgdex_api::{CardBrief, Query, Tcgdex};
use tower_http::cors::{Any, CorsLayer};

use crate::login_handler::{get_user, google_login_handler};

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
        .route("/auth/google", post(google_login_handler))
        .with_state(pool)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3333").await.unwrap();

    println!("Server works on 0.0.0.0:3333");
    axum::serve(listener, app).await.unwrap();
}

#[derive(serde::Serialize)]
struct CardBriefResponse {
    id: String,
    local_id: String,
    name: String,
    image: String,
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
