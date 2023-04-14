#![allow(dead_code, unused_imports)]
pub mod twitch;
use serde::{Deserialize, Serialize};

use dotenv::dotenv;
use nalgebra::{DMatrix, DVector, U12};
use nalgebra_sparse::{CooMatrix, CscMatrix};
use std::collections::HashMap;
use std::env;
use twitch::igdb;

struct RatedGame {
    id: u32,
    rating: f32,
}

#[repr(u32)]
enum GameGenres {
    Adventure = 31,
    Arcade = 33,
    CardAndBoardGame = 35,
    Fighting = 4,
    HackNSlash = 25,
    Indie = 32,
    MOBA = 36,
    Music = 7,
    Pinball = 30,
    Platform = 8,
    PointAndClick = 2,
    Puzzle = 9,
    RPG = 12,
    Racing = 10,
    Shooter = 5,
    Simulator = 13,
    Sport = 14,
    Strategy = 15,
    TBS = 16,
    Tactical = 24,
    Trivia = 26,
    VisualNovel = 34,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let client_id = env::var("TWITCH_CLIENT_ID").expect("Failed to get twitch client id.");
    let client_secret =
        env::var("TWITCH_CLIENT_SECRET").expect("Failed to get twitch client secret.");
    let mut igdb_wrapper = igdb::IGDBWrapper::new(client_id, client_secret);
    igdb_wrapper.refresh_auth().await;

    #[derive(Serialize, Deserialize)]
    struct DBQuery {
        id: u64,
        genres: Vec<u64>,
    }

    let res = igdb_wrapper
        .query::<Vec<DBQuery>>(
            "games",
            "
            fields genres;
            where id = (7334, 119133, 125764);
            ",
        )
        .await
        .expect("Query failed.");

    for dbq in res {
        println!("{}", e.id);
    }

    /*
    let games = vec![
        RatedGame {
            id: 7334,
            rating: 1.0,
        },
        RatedGame {
            id: 119133,
            rating: 1.0,
        },
        RatedGame {
            id: 125764,
            rating: 1.0,
        },
    ];

    let genres = vec![
        GameGenre::PointAndClick,
        GameGenre::Fighting,
        GameGenre::Shooter,
        GameGenre::Music,
        GameGenre::Platform,
        GameGenre::Puzzle,
        GameGenre::Racing,
        GameGenre::RPG,
        GameGenre::Simulator,
        GameGenre::Sport,
        GameGenre::Strategy,
        GameGenre::TBS,
        GameGenre::Tactical,
        GameGenre::HackNSlash,
        GameGenre::Trivia,
        GameGenre::Pinball,
        GameGenre::Adventure,
        GameGenre::Indie,
        GameGenre::Arcade,
        GameGenre::VisualNovel,
        GameGenre::CardAndBoardGame,
        GameGenre::MOBA,
    ];

    let row_to_game: HashMap<usize, &RatedGame> =
        games.iter().enumerate().map(|(i, g)| (i, g)).collect();

    let col_to_genre: HashMap<usize, &GameGenre> =
        genres.iter().enumerate().map(|(i, g)| (i, g)).collect();

    let rating_iter = games.iter().map(|g| g.rating);
    let user_input_vec = DVector::from_iterator(games.len(), rating_iter);

    let mut game_mat = CooMatrix::<f64>::new(games.len(), 22);
    */
}
/*
Recommendation Features
- rating : rating_count exists, could factor in number of ratings to generate a rating score
- genres
 */
