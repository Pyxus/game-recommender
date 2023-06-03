#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;

mod game_rec;

use std::collections::HashMap;

use futures::lock::Mutex;
use game_rec::cb_filtering::{Game, RatedGame};
use game_rec::Recommender;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::serde::json::Json;
use rocket::State;
use rocket::{get, routes};
use rocket::{Request, Response};

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "http://localhost:5173"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, PATCH, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[get("/world")]
async fn index(client: &State<Mutex<Recommender>>) -> &'static str {
    client.lock().await.init().await;
    "Hello, world!"
}

#[post("/recommend", format = "json", data = "<games_json>")]
async fn recommend_game(
    games_json: Json<HashMap<u64, f64>>,
    client: &State<Mutex<Recommender>>,
) -> Json<Vec<RatedGame>> {
    let client = client.lock().await;
    let games = games_json.into_inner();

    Json(client.get_recommended_games(&games).await)
}

/// Catches all OPTION requests in order to get the CORS related Fairing triggered.
#[options("/<_..>")]
fn all_options() {
    /* Intentionally left empty */
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
        .mount("/", routes![index, all_options, search_game, recommend_game])
        .attach(CORS)
}

async fn _test() {
    let mut rec = Recommender::new();
    rec.init().await;

    let rating_by_ids = HashMap::from([(7334, 1.0)]);
    let _ = rec.get_recommended_games(&rating_by_ids).await;
}
