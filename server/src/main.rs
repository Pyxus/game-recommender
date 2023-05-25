#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;

mod game_rec;

use std::time::Duration;

use game_rec::cb_filtering::{Game, RatedGame};
use game_rec::Recommender;
use rocket::serde::json::Json;
use rocket::serde::{Serialize, Deserialize};
use tokio::time::sleep;

#[derive(FromForm, Serialize, Deserialize)]
struct InputGame{
    id: i64,
    rating: f64,
}

#[get("/world")]
fn index() -> &'static str {
    "Hello, world!"
}

#[post("/recommend", format = "json", data = "<games>")]
async fn recommend_game(games: Json<Vec<InputGame>>) -> &'static str {
    sleep(Duration::from_millis(1000)).await;
    "Games added successfully!"
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, recommend_game])
}

async fn _test(){
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
        },
    }];
    let _ = rec.get_recommended_games(&rated_games).await;
}