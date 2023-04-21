mod game_rec;

use game_rec::cb_filtering::{Game, RatedGame};
use game_rec::Recommender;

#[tokio::main]
async fn main() {
    let mut rec = Recommender::new();
    rec.init().await;

    let rated_games = vec![RatedGame {
        rating: 1.0,
        game: Game {
            name: String::from("Bloodborne"),
            id: 7334,
            genres: vec![2],
            themes: vec![1],
            player_perspectives: vec![1],
        },
    }];
    let _ = rec.get_recommended_games(&rated_games).await;
}
