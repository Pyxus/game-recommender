pub mod igdb;
pub mod cb_filtering;

use dotenv::dotenv;
use std::env;
use igdb::{IGDBWrapper};
use cb_filtering::{RatedGame};

struct Client {
    id: String,
    secret: String,
}

pub struct Recommender{
    db: IGDBWrapper
}

impl Recommender {
    pub fn new() -> Self {
        let client = create_client();
        Recommender {db: IGDBWrapper::new(client.id, client.secret) }
    }

    pub async fn init(&mut self){
        self.db.refresh_auth().await;
    }

    pub async fn get_recommended_games(&self, rated_games: &Vec<RatedGame>) -> Vec<RatedGame>{
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
        recommended_games.sort_by(|a, b| a.rating.partial_cmp(&b.rating).unwrap());
        recommended_games
            .iter()
            .for_each(|rg| println!("{} ({})", rg.game.name, rg.rating));

        return recommended_games;
    }
}

fn create_client() -> Client {
    dotenv().ok();
    Client {
        id: env::var("TWITCH_CLIENT_ID").expect("Failed to get twitch client id."),
        secret: env::var("TWITCH_CLIENT_SECRET").expect("Failed to get twitch client secret."),
    }
}
