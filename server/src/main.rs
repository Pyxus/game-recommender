#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;

mod game_rec;

use std::time::Duration;

use futures::lock::Mutex;
use game_rec::cb_filtering::{Game, RatedGame};
use game_rec::Recommender;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::State;
use tokio::time::sleep;

#[derive(FromForm, Serialize, Deserialize)]
struct InputGame {
    id: i64,
    rating: f64,
}

#[get("/world")]
async fn index(client: &State<Mutex<Recommender>>) -> &'static str {
    client.lock().await.init().await;
    "Hello, world!"
}

#[post("/recommend", format = "json", data = "<games>")]
async fn recommend_game(games: Json<Vec<RatedGame>>, client: &State<Mutex<Recommender>>) -> Json<Vec<RatedGame>> {
    let client = client.lock().await;
    Json(client.get_recommended_games(&games).await)
}

#[get("/search_game?<name>")]
async fn search_game(name: String, client: &State<Mutex<Recommender>>) -> Json<Vec<Game>> {
    let client = client.lock().await;
    Json(client.search_game(&name).await)
}

#[launch]
async fn rocket() -> _ {
    let mut rec = Recommender::new();
    rec.init().await;

    rocket::build()
        .manage(Mutex::new(rec))
        .mount("/", routes![index, search_game, recommend_game])
}

async fn _test() {
    let mut rec = Recommender::new();
    rec.init().await;

    let rated_games = vec![RatedGame {
        rating: 1.0,
        game: Game {
            name: String::from("Bloodborne"),
            id: 7334,
            genres: vec![2],
            themes: vec![1],
            player_perspectives: vec![1],
            ..Default::default()
        },
    }];
    let _ = rec.get_recommended_games(&rated_games).await;
}
