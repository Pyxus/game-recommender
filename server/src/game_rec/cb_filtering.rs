use super::igdb::{Genres, Game, IGDBWrapper, PlayerPerspective, Themes};
use nalgebra::{Const, DVector, Dyn, Matrix, VecStorage};
use nalgebra_sparse::{CooMatrix, CscMatrix};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use strum::{EnumCount, IntoEnumIterator};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RatedGame {
    pub game: Game,
    pub rating: f64,
}

struct FeatureSet {
    genres: HashSet<u64>,
    themes: HashSet<u64>,
    perspectives: HashSet<u64>,
}

pub async fn create_candidate_list(db: &IGDBWrapper, game_ids: &Vec<u64>) -> Vec<Game> {
    let games = db.find_games_from_ids(&game_ids).await;
    let similar_games = db.find_similar_games(&game_ids).await;
    let feature_set = create_feature_set(&games);
    let where_exlude_game_id = comma_sep(&similar_games, |game| game.id.to_string());
    let where_genre_str = comma_sep(&feature_set.genres, |g| g.to_string());
    let where_theme_str = comma_sep(&feature_set.themes, |g| g.to_string());
    let where_perspective_str = comma_sep(&feature_set.perspectives, |g| g.to_string());
    let query = format!(
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
    );
    let query_result = db
        .query::<Vec<Game>>("games", query.as_str())
        .await;

    match query_result {
        Ok(mut candidate_games) => {
            candidate_games.extend(similar_games);
            candidate_games
        },
        Err(_) => vec![],
    }
}

pub async fn create_feature_mat(games: &Vec<Game>) -> CscMatrix<f64> {
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

pub async fn calc_profile_mat(
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

fn comma_sep<T, F, C>(collection: C, f: F) -> String
where
    F: FnMut(&T) -> String,
    C: IntoIterator<Item = T>,
{
    let coll_vec: Vec<T> = collection.into_iter().collect();
    coll_vec.iter().map(f).collect::<Vec<String>>().join(", ")
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
