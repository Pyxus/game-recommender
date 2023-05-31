pub mod cb_filtering;
pub mod igdb;

use cb_filtering::Game;
use cb_filtering::RatedGame;
use dotenv::dotenv;
use igdb::IGDBWrapper;
use std::env;

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

    pub async fn search_game(&self, name: &String) -> Vec<Game> {
        self.db
            .query::<Vec<Game>>(
                "games",
                format!(
                r#"
                    fields name, first_release_date;
                    search "{name}";
                    where version_parent = null & category = 0 & first_release_date != null;
                "#
                )
                .as_str(),
            )
            .await
            .expect("Failed to query database.")
    }

    pub async fn get_recommended_games(&self, rated_games: &Vec<RatedGame>) -> Vec<RatedGame> {
        let games = rated_games
            .iter()
            .map(|rg| rg.game.clone())
            .collect::<Vec<_>>();
        let candidate_games = cb_filtering::create_candidate_list(&self.db, &games).await;
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
        id: env::var("TWITCH_CLIENT_ID").expect("Failed to get twitch client id."),
        secret: env::var("TWITCH_CLIENT_SECRET").expect("Failed to get twitch client secret."),
    }
}
