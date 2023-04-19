#![allow(dead_code, unused_imports)]
pub mod twitch;

use bimap::{BiHashMap, BiMap};
use dotenv::dotenv;
use nalgebra::{Const, DMatrix, DVector, Dyn, Matrix, RowDVector, VecStorage};
use nalgebra_sparse::{CooMatrix, CscMatrix};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fmt::Display;
use strum::{EnumCount, IntoEnumIterator};
use twitch::igdb::{Genres, IGDBWrapper, PlayerPerspective, Themes};
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
    name: String,
    id: u64,
    #[serde(default)]
    genres: Vec<u64>,
    #[serde(default)]
    themes: Vec<u64>,
    #[serde(default)]
    player_perspectives: Vec<u64>,
}

struct FeatureSet {
    genres: HashSet<u64>,
    themes: HashSet<u64>,
    perspectives: HashSet<u64>,
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ID: {}", self.id)
    }
}

#[tokio::main]
async fn main() {
    // Init DB Wrapper
    let client = create_client();
    let mut igdb_wrapper = IGDBWrapper::new(client.id, client.secret);
    igdb_wrapper.refresh_auth().await;

    // Generate input games for testing
    let rated_games = vec![
        RatedGame {
            game: Game {
                name: String::from("Bloodborne"),
                id: 7334,
                genres: vec![12, 31],
                themes: vec![1, 17, 19, 38],
                player_perspectives: vec![2],
            },
            rating: 2.0,
        },
        RatedGame {
            game: Game {
                name: String::from("Elden Ring"),
                id: 119133,
                genres: vec![12, 31],
                themes: vec![1, 17, 38],
                player_perspectives: vec![2],
            },
            rating: 1.0,
        },
        RatedGame {
            game: Game {
                name: String::from("Guilty Gear Strive"),
                id: 125764,
                genres: vec![4],
                themes: vec![1],
                player_perspectives: vec![4],
            },
            rating: 1.5,
        },
    ];

    // Generate candidate games using input game features
    let games = rated_games
        .iter()
        .map(|rg| rg.game.clone())
        .collect::<Vec<_>>();
    let candidate_features = create_feature_set(&games);
    let candidate_games = create_candidate_list(&igdb_wrapper, &games, &candidate_features).await;
    let candidate_mat = create_feature_mat(&candidate_games).await;

    let user_profile = calc_profile_mat(&rated_games).await;
    let recomendation_list = &candidate_mat * &user_profile;
    let mut recommended_games = Vec::from_iter(
        recomendation_list
            .column(0)
            .iter()
            .enumerate()
            .map(|(i, rating)| RatedGame {
                game: candidate_games[i].clone(),
                rating: *rating,
            }),
    );

    recommended_games.sort_by(|a, b| a.rating.partial_cmp(&b.rating).unwrap());
    recommended_games
        .iter()
        .for_each(|rg| println!("{} ({})", rg.game.name, rg.rating));
}

fn create_client() -> Client {
    dotenv().ok();
    Client {
        id: env::var("TWITCH_CLIENT_ID").expect("Failed to get twitch client id."),
        secret: env::var("TWITCH_CLIENT_SECRET").expect("Failed to get twitch client secret."),
    }
}

fn create_feature_set(games: &Vec<Game>) -> FeatureSet {
    let genres = games
        .iter()
        .flat_map(|game| game.genres.iter())
        .copied()
        .collect();

    let themes = games
        .iter()
        .flat_map(|game| game.themes.iter())
        .copied()
        .collect();

    let perspectives = games
        .iter()
        .flat_map(|game| game.player_perspectives.iter())
        .copied()
        .collect();

    return FeatureSet {
        genres,
        themes,
        perspectives,
    };
}

async fn create_candidate_list(
    db: &IGDBWrapper,
    input_games: &Vec<Game>,
    feature_set: &FeatureSet,
) -> Vec<Game> {
    #[derive(Serialize, Deserialize)]
    struct GameQuery {
        id: u64,
        similar_games: Vec<Game>,
    }
    let where_game_id_str = format!(
        "{}",
        input_games
            .iter()
            .map(|g| g.id.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let q = db
        .query::<Vec<GameQuery>>(
            "games", 
            format!(
            "
            fields similar_games.name, similar_games.genres, similar_games.themes, similar_games.player_perspectives;
            where id = ({where_game_id_str});
            limit 100; 
            "
            ).as_str()
        )
        .await
        .expect("Failed to query database.");

    let where_exlude_game_id = q
        .iter()
        .flat_map(|query| query.similar_games.iter())
        .map(|game| game.id.to_string())
        .collect::<Vec<String>>()
        .join(",");

    let where_genre_str = format!(
        "{}",
        feature_set
            .genres
            .iter()
            .map(|g| (*g as u64).to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let where_theme_str = format!(
        "{}",
        feature_set
            .themes
            .iter()
            .map(|g| (*g as u64).to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let where_perspective_str = format!(
        "{}",
        feature_set
            .perspectives
            .iter()
            .map(|g| (*g as u64).to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let similar_games = q
        .iter()
        .flat_map(|gq| gq.similar_games.clone())
        .collect::<Vec<Game>>();

    let mut candidate_games = db
        .query::<Vec<Game>>(
            "games",
            format!(
                "
            fields name, genres, themes, player_perspectives;
            where genres = 
                ({where_genre_str}) 
                & themes = ({where_theme_str}) 
                & player_perspectives = ({where_perspective_str})
                & id != ({where_exlude_game_id})
                & rating > 6;
            limit 500;
            "
            )
            .as_str(),
        )
        .await
        .expect("Failed to query database.");
    
    candidate_games.extend(similar_games);

    return candidate_games;
}

async fn create_feature_mat(games: &Vec<Game>) -> CscMatrix<f64> {
    let total_features = Genres::COUNT + Themes::COUNT + PlayerPerspective::COUNT;
    let mut feat_mat = CooMatrix::<f64>::zeros(games.len(), total_features);
    let mut col = 0;

    for (row, game) in games.iter().enumerate() {
        for genre in Genres::iter() {
            let genre_id = genre as u64;
            if game.genres.contains(&genre_id) {
                feat_mat.push(row, col, 1.0);
            }
            col += 1;
        }
        for theme in Themes::iter() {
            let theme_id = theme as u64;
            if game.themes.contains(&theme_id) {
                feat_mat.push(row, col, 1.0);
            }
            col += 1;
        }
        for perspective in PlayerPerspective::iter() {
            let perspective_id = perspective as u64;
            if game.player_perspectives.contains(&perspective_id) {
                feat_mat.push(row, col, 1.0);
            }
            col += 1;
        }
        col = 0;
    }

    return CscMatrix::from(&feat_mat);
}

async fn calc_profile_mat(
    rated_games: &Vec<RatedGame>,
) -> Matrix<f64, Dyn, Const<1>, VecStorage<f64, Dyn, Const<1>>> {
    let games = rated_games
        .iter()
        .map(|rg| rg.game.clone())
        .collect::<Vec<_>>();
    let user_rating_mat =
        DVector::from_iterator(rated_games.len(), rated_games.iter().map(|g| g.rating));
    let feat_mat = create_feature_mat(&games).await;
    let weighted_feat_mat = feat_mat.transpose() * user_rating_mat;
    let user_profile = &weighted_feat_mat / weighted_feat_mat.sum();

    return user_profile;
}
