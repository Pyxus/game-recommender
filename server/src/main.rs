#![allow(dead_code, unused_imports)]
pub mod twitch;

use bimap::{BiHashMap, BiMap};
use dotenv::dotenv;
use nalgebra::{DMatrix, DVector, RowDVector};
use nalgebra_sparse::{CooMatrix, CscMatrix};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fmt::Display;
use twitch::igdb::{GameGenres, IGDBWrapper};

struct Client {
    id: String,
    secret: String,
}

#[derive(Clone)]
struct RatedGame {
    game: Game,
    rating: f64,
}

#[derive(Clone, Serialize, Deserialize)]
struct Game {
    id: u64,
    genres: Vec<u64>,
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ID: {}", self.id)
    }
}

#[tokio::main]
async fn main() {
    let client = create_client();
    let mut igdb_wrapper = IGDBWrapper::new(client.id, client.secret);
    igdb_wrapper.refresh_auth().await;

    let rated_games = vec![
        RatedGame {
            game: Game {
                id: 7334,
                genres: vec![12, 31],
            }, // Bloodborne
            rating: 2.0,
        },
        RatedGame {
            game: Game {
                id: 119133,
                genres: vec![12, 31],
            }, // Elden Ring
            rating: 1.0,
        },
        RatedGame {
            game: Game {
                id: 125764,
                genres: vec![4],
            }, // Strive
            rating: 1.5,
        },
    ];
    let games = rated_games
        .iter()
        .map(|rg| rg.game.clone())
        .collect::<Vec<_>>();
    let rating_mat =
        DVector::from_iterator(rated_games.len(), rated_games.iter().map(|g| g.rating));
    let feat_mat = create_feature_mat(&games).await;
    let weighted_feat_mat = &feat_mat.transpose() * &rating_mat;
    let user_profile = &weighted_feat_mat / weighted_feat_mat.sum();
    let candidate_genres = vec![12, 31, 4];
    let where_genre_str = format!(
        "{}",
        candidate_genres
            .iter()
            .map(|g| g.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );
    let candidate_games = igdb_wrapper
        .query::<Vec<Game>>(
            "games",
            format!(
                "
            fields genres;
            where genres = ({where_genre_str}) & rating > 6;
            limit 500;
            "
            )
            .as_str(),
        )
        .await
        .expect("Failed to query database.");

    let candidate_mat = create_feature_mat(&candidate_games).await;
    let recomendation_list = &candidate_mat * &user_profile;
    let mut rated_rec_list = Vec::<RatedGame>::new();
    for (i, rating) in recomendation_list.column(0).iter().enumerate() {
        rated_rec_list.push(RatedGame {
            game: candidate_games[i].clone(),
            rating: *rating,
        })
    }

    rated_rec_list.sort_by(|a, b| a.rating.partial_cmp(&b.rating).unwrap());

    let where_genre_str = format!(
        "{}",
        rated_rec_list
            .iter()
            .map(|rg| rg.game.id.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    #[derive(Serialize, Deserialize)]
    struct NamedGame {
        id: u64,
        name: String,
    }

    let recomendation_list = igdb_wrapper
        .query::<Vec<NamedGame>>(
            "games",
            format!("fields name; where id = ({where_genre_str}) & follows > 2; limit 500;").as_str(),
        )
        .await
        .expect("Failed to query database");
    
    for (i, game) in recomendation_list.iter().enumerate() {
        println!("{} ({})", &game.name, &rated_rec_list[i].rating);
    }
}

fn create_client() -> Client {
    dotenv().ok();
    Client {
        id: env::var("TWITCH_CLIENT_ID").expect("Failed to get twitch client id."),
        secret: env::var("TWITCH_CLIENT_SECRET").expect("Failed to get twitch client secret."),
    }
}

async fn create_feature_mat(games: &Vec<Game>) -> CscMatrix<f64> {
    let genres = vec![
        GameGenres::PointAndClick,
        GameGenres::Fighting,
        GameGenres::Shooter,
        GameGenres::Music,
        GameGenres::Platform,
        GameGenres::Puzzle,
        GameGenres::Racing,
        GameGenres::RPG,
        GameGenres::Simulator,
        GameGenres::Sport,
        GameGenres::Strategy,
        GameGenres::TBS,
        GameGenres::Tactical,
        GameGenres::HackNSlash,
        GameGenres::Trivia,
        GameGenres::Pinball,
        GameGenres::Adventure,
        GameGenres::Indie,
        GameGenres::Arcade,
        GameGenres::VisualNovel,
        GameGenres::CardAndBoardGame,
        GameGenres::MOBA,
    ];

    let mut feat_mat = CooMatrix::<f64>::zeros(games.len(), genres.len());

    for (row, game) in games.iter().enumerate() {
        for (col, genre) in genres.iter().enumerate() {
            let genre_id = *genre as u64;
            if game.genres.contains(&genre_id) {
                feat_mat.push(row, col, 1.0);
            }
        }
    }

    return CscMatrix::from(&feat_mat);
}

async fn create_candidate_mat(db: &IGDBWrapper, genres: &Vec<u64>) -> CscMatrix<f64> {
    let where_genre_str = format!(
        "{}",
        genres
            .iter()
            .map(|g| g.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let games = db
        .query::<Vec<Game>>(
            "games",
            format!(
                "
            fields genres;
            where genres = ({where_genre_str}) & rating > 6;;
            sort release_dates.date desc;
            limit 100;
            "
            )
            .as_str(),
        )
        .await
        .expect("Failed to query database.");

    return create_feature_mat(&games).await;
}

async fn get_genres_by_game(db: &IGDBWrapper, game_ids: &Vec<u64>) -> HashMap<u64, Vec<u64>> {
    let where_genre_str = format!(
        "{}",
        game_ids
            .iter()
            .map(|gid| gid.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let query_result = db
        .query::<Vec<Game>>(
            "games",
            format!(
                "
                fields genres;
                where id = ({where_genre_str});
                "
            )
            .as_str(),
        )
        .await
        .expect("Query failed.")
        .iter()
        .map(|g| (g.id, g.genres.clone()))
        .collect::<HashMap<_, _>>();

    return query_result;
}

/*
async fn there_was_an_attempt() {
    #[derive(Copy, Clone)]
    struct RatedGame {
        id: u64,
        rating: f32,
    }

    dotenv().ok();
    let client_id = env::var("TWITCH_CLIENT_ID").expect("Failed to get twitch client id.");
    let client_secret =
        env::var("TWITCH_CLIENT_SECRET").expect("Failed to get twitch client secret.");
    let mut igdb_wrapper = igdb::IGDBWrapper::new(client_id, client_secret);
    igdb_wrapper.refresh_auth().await;

    let games = vec![
        RatedGame {
            id: 7334, // Bloodborne
            rating: 1.0,
        },
        RatedGame {
            id: 119133, // Elden Ring
            rating: 1.0,
        },
        RatedGame {
            id: 125764, // Strive
            rating: 1.0,
        },
    ];

    let genres = vec![
        igdb::GameGenres::PointAndClick,
        igdb::GameGenres::Fighting,
        igdb::GameGenres::Shooter,
        igdb::GameGenres::Music,
        igdb::GameGenres::Platform,
        igdb::GameGenres::Puzzle,
        igdb::GameGenres::Racing,
        igdb::GameGenres::RPG,
        igdb::GameGenres::Simulator,
        igdb::GameGenres::Sport,
        igdb::GameGenres::Strategy,
        igdb::GameGenres::TBS,
        igdb::GameGenres::Tactical,
        igdb::GameGenres::HackNSlash,
        igdb::GameGenres::Trivia,
        igdb::GameGenres::Pinball,
        igdb::GameGenres::Adventure,
        igdb::GameGenres::Indie,
        igdb::GameGenres::Arcade,
        igdb::GameGenres::VisualNovel,
        igdb::GameGenres::CardAndBoardGame,
        igdb::GameGenres::MOBA,
    ];

    #[derive(Serialize, Deserialize)]
    struct GenreQuery {
        id: u64,
        genres: Vec<u64>,
    }

    let genres_by_game: HashMap<u64, Vec<u64>> = igdb_wrapper
        .query::<Vec<GenreQuery>>(
            "games",
            "
            fields genres;
            where id = (7334, 119133, 125764);
            ",
        )
        .await
        .expect("Query failed.")
        .iter()
        .map(|q| (q.id, q.genres.clone()))
        .collect();

    let row_to_game: BiHashMap<usize, u64> = games
        .iter()
        .enumerate()
        .map(|(row, game)| (row, game.id))
        .collect::<BiMap<usize, u64>>();

    let col_to_genre: BiHashMap<usize, u64> = genres
        .iter()
        .enumerate()
        .map(|(col, genre)| (col, *genre as u64))
        .collect::<BiMap<usize, u64>>();

    // Construct feature matrix
    let mut feat_mat = CooMatrix::<f32>::new(games.len(), genres.len());
    for row in 0..games.len() {
        let game_id = row_to_game.get_by_left(&row).unwrap();
        let genres = &genres_by_game[game_id];

        for genre in genres {
            let genre_id = *col_to_genre.get_by_right(&genre).unwrap();
            feat_mat.push(row, genre_id, 1.0);
        }
    }
    let feat_mat = CscMatrix::from(&feat_mat);

    // Calcuate weighted feature matrix
    let rating_vec = DVector::from_iterator(games.len(), games.iter().map(|g| g.rating));
    let weighted_rating_mat = &feat_mat.transpose() * &rating_vec;

    // Calcuate user profile (normalized sum of weighted feat matrix)
    let mut user_profile = DVector::<f32>::zeros(genres.len());
    let weighted_sums = weighted_rating_mat.column_sum();
    let sum = weighted_sums.iter().sum::<f32>();
    for col in 0..weighted_sums.ncols() {
        user_profile[col] = weighted_sums[col] / sum;
    }

    // Multiply user profile matrix by candidate matrix. Result is a weighted candidate matrix.
    let mut genre_ids: HashSet<u64> = std::collections::HashSet::new();
    for genre_list in genres_by_game.values() {
        for genre_id in genre_list {
            genre_ids.insert(*genre_id);
        }
    }

    // Construct candidate matrix
    let where_genre_str = format!(
        "({})",
        genre_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let candidate_games = igdb_wrapper
        .query::<Vec<GenreQuery>>(
            "games",
            format!(
                "
                fields genres;
                where genres = {where_genre_str} & release_dates.date > 2010 & rating > 5;
                sort release_dates.date desc;
                limit 100;
                "
            )
            .as_str(),
        )
        .await
        .expect("Query failed.");

    let genres_by_game: HashMap<u64, Vec<u64>> = candidate_games
        .iter()
        .map(|q| (q.id, q.genres.clone()))
        .collect();

    let row_to_game: BiHashMap<usize, u64> = candidate_games
        .iter()
        .enumerate()
        .map(|(row, game)| (row, game.id))
        .collect::<BiMap<usize, u64>>();

    let mut candidate_feat_mat = CooMatrix::<f32>::new(candidate_games.len(), genres.len());
    for row in 0..games.len() {
        let game_id = row_to_game.get_by_left(&row).unwrap();
        let genres = &genres_by_game[game_id];

        for genre in genres {
            let genre_id = *col_to_genre.get_by_right(&genre).unwrap();
            candidate_feat_mat.push(row, genre_id, 1.0);
        }
    }
    let candidate_feat_mat = CscMatrix::from(&candidate_feat_mat);
    let weighted_candidate_mat = &candidate_feat_mat * &user_profile;

    // Sum weighted ratings and sort to rank
}
*/
