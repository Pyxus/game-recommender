pub mod cb_filtering;
pub mod igdb;

use cb_filtering::Game;
use cb_filtering::RatedGame;
use dotenv::dotenv;
use igdb::IGDBWrapper;
use std::collections::HashMap;
use std::env;

use crate::game_rec::cb_filtering::find_games_from_ids;

struct Client {
    id: String,
    secret: String,
}

pub struct Recommender {
    db: IGDBWrapper,
}

impl Recommender {
    pub fn new() -> Self {
        let client = create_client();
        Recommender {
            db: IGDBWrapper::new(client.id, client.secret),
        }
    }

    pub async fn init(&mut self) {
        self.db.refresh_auth().await;
    }

    pub async fn search_game(&self, name: &String) -> Result<Vec<Game>, reqwest::Error> {
        let query = format!(
            r#"
            fields name, first_release_date;
            search "{name}";
            where version_parent = null & category = 0 & first_release_date != null;
        "#
        );
        let result = self.db
            .query::<Vec<Game>>("games",query.as_str())
            .await?;
        Ok(result)
    }

    pub async fn get_recommended_games(&self, rating_by_id: &HashMap<u64, f64>) -> Vec<RatedGame> {
        let game_ids = rating_by_id.keys().cloned().collect::<Vec<u64>>();
        let input_games = find_games_from_ids(&self.db, &game_ids).await;
        let rated_games = input_games
            .iter()
            .map(|game| RatedGame {
                game: game.clone(),
                rating: *rating_by_id.get(&game.id).expect("Failed to get game id"),
            })
            .collect();
        let candidate_games = cb_filtering::create_candidate_list(&self.db, &game_ids).await;
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
}

fn create_client() -> Client {
    dotenv().ok();
    Client {
        id: env::var("TWITCH_CLIENT_ID").expect("Failed to get twitch client id from env."),
        secret: env::var("TWITCH_CLIENT_SECRET").expect("Failed to get twitch client secret from env."),
    }
}
