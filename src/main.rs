use axum::{
    Router,
    extract::Path,
    http::{Method, StatusCode},
    response::Json,
    routing::get,
};
use serde_json::{Value, json};
use tcgdex_api::{CardBrief, Query, Tcgdex};
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any);

    let app = Router::new()
        .route("/pokeapi/{poke}", get(rustemon_api_handler))
        .route("/tcgapi/{poke}", get(tcg_api_handler))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3333").await.unwrap();
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
