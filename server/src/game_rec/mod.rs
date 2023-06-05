pub mod cb_filtering;

use crate::igdb::{Game, IGDBClient};
use crate::util::comma_sep;
use cb_filtering::RatedGame;
use std::collections::HashMap;

pub struct Recommender {
    igdb_client: IGDBClient,
}

impl Recommender {
    pub fn new(igdb_client: IGDBClient) -> Self {
        Recommender { igdb_client }
    }

    pub async fn get_recommended_games(&self, rating_by_id: &HashMap<u64, f64>) -> Vec<RatedGame> {
        let game_ids = rating_by_id.keys().cloned().collect::<Vec<u64>>();
        let input_games = self.igdb_client.find_games_from_ids(&game_ids).await;
        let rated_games = input_games
            .iter()
            .map(|game| RatedGame {
                game: game.clone(),
                rating: *rating_by_id.get(&game.id).expect("Failed to get game id"),
            })
            .collect();
        let candidate_games = self.create_candidate_list(&game_ids).await;
        let candidate_mat = cb_filtering::create_feature_mat(&candidate_games).await;

        let user_profile = cb_filtering::calc_profile_mat(&rated_games).await;
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
        recommended_games.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap());
        recommended_games
    }

    pub fn get_igdb(&self) -> &IGDBClient {
        &self.igdb_client
    }

    async fn create_candidate_list(&self, game_ids: &Vec<u64>) -> Vec<Game> {
        let games = self.igdb_client.find_games_from_ids(&game_ids).await;
        let similar_games = self.igdb_client.find_similar_games(&game_ids).await;
        let feature_set = cb_filtering::create_feature_set(&games);
        let where_exclude_game_id = comma_sep(&similar_games, |game| game.id.to_string());
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
                & id != ({where_exclude_game_id})
                & rating > 6;
            limit 500;
            "
        );
        let query_result = self
            .igdb_client
            .query::<Vec<Game>>("games", query.as_str())
            .await;

        match query_result {
            Ok(mut candidate_games) => {
                candidate_games.extend(similar_games);
                candidate_games
            }
            Err(_) => vec![],
        }
    }
}
