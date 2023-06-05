#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;

mod game_rec;
mod igdb;
mod util;

use std::collections::HashMap;

use futures::lock::Mutex;
use game_rec::cb_filtering::RatedGame;
use game_rec::Recommender;
use igdb::{Game, IGDBClient, TwitchClient};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::serde::json::Json;
use rocket::State;
use rocket::{get, routes};
use rocket::{Request, Response};

struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

/// Catches all OPTION requests in order to get the CORS related Fairing triggered.
#[options("/<_..>")]
fn all_options() {
    /* Intentionally left empty */
}

#[post("/recommend_games", format = "json", data = "<rating_by_game_id_json>")]
async fn recommend_games(
    rating_by_game_id_json: Json<HashMap<u64, f64>>,
    rec_mutex: &State<Mutex<Recommender>>,
) -> Json<Vec<RatedGame>> {
    let recommender = rec_mutex.lock().await;
    let rating_by_game_id = rating_by_game_id_json.into_inner();

    Json(recommender.get_recommended_games(&rating_by_game_id).await)
}

#[get("/search_games?<name>")]
async fn search_games(name: String, _rec_mutex: &State<Mutex<Recommender>>) -> Json<Vec<Game>> {
    let recommender = _rec_mutex.lock().await;

    match recommender.get_igdb().search_game(&name).await {
        Ok(games) => Json(games),
        Err(error) => {
            eprintln!("Error searching for games: {:?}", error);
            Json(vec![])
        }
    }
}

#[launch]
async fn rocket() -> _ {
	
    let twitch_client = TwitchClient::from_dotenv(
        String::from("TWITCH_CLIENT_ID"),
        String::from("TWITCH_CLIENT_SECRET"),
    );
    let mut igdb_client = IGDBClient::new(twitch_client);
    igdb_client.refresh_auth().await;

    let rec = Recommender::new(igdb_client);

    rocket::build()
        .attach(CORS)
        .manage(Mutex::new(rec))
        .mount("/", routes![all_options, search_games, recommend_games])
}
