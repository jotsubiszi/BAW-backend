use axum::{Router, extract::Path, http::StatusCode, response::Json, routing::get};
use pokemon_tcg_sdk::{
    card::{GetCardRequest, SearchCardsRequest},
    client::Client,
};
use serde_json::{Value, json};

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new().route("/pokeapi/{poke}", get(print_pokemon));
    // .route("/tcgapi/{api}/card/{card}", get(tcg_api_handler));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn print_pokemon(Path(poke_name): Path<String>) -> Result<Json<Value>, (StatusCode, String)> {
    let rustemon_client = rustemon::client::RustemonClient::default();
    let pokemone = rustemon::pokemon::pokemon::get_by_name(&poke_name, &rustemon_client).await;

    match pokemone {
        Ok(p) => Ok(Json(json!(p.moves))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e))),
    }
}

// async fn tcg_api_handler(
//     Path(api_key): Path<String>,
//     Path(card_id): Path<String>,
// ) -> Result<Json<Value>, (StatusCode, String)> {
//     let client = Client::with_api_key(&api_key);
//     let carde = client
//         .SearchCardsRequest::new("name:{}", &card_id)
//         .await;
//
//     match carde {
//         // Card
//         Ok(c) => Ok(Json(json!(c))),
//         // Will be a 'ClientError' enum
//         Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e))),
//     }
// }
