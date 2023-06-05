pub mod enums;

#[allow(unused_imports)]
use strum_macros::{EnumCount, EnumIter};
use dotenv::dotenv;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::env;
use std::time::SystemTime;
use crate::util::{comma_sep};

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct Game {
    pub id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub genres: Vec<u64>,
    #[serde(default)]
    pub themes: Vec<u64>,
    #[serde(default)]
    pub player_perspectives: Vec<u64>,
    #[serde(default)]
    pub first_release_date: i64,
    #[serde(default)]
    pub similar_games: Vec<Game>,
}

pub struct TwitchClient {
    id: String,
    secret: String,
}

impl TwitchClient {
    pub fn from_dotenv(id_key: String, secret_key: String) -> TwitchClient {
        dotenv().ok();
        TwitchClient {
            id: env::var(id_key).expect("Failed to get twitch client id from env."),
            secret: env::var(secret_key).expect("Failed to get twitch client secret from env."),
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
struct Auth {
    access_token: String,
    expires_in: u64,
    token_type: String,
}

pub struct IGDBClient {
    auth: Auth,
    auth_refreshed_at: SystemTime,
    reqwest_client: reqwest::Client,
    twitch_client: TwitchClient,
}

impl IGDBClient {
    pub fn new(twitch_client: TwitchClient) -> Self {
        IGDBClient {
            auth: Auth::default(),
            auth_refreshed_at: SystemTime::now(),
            reqwest_client: reqwest::Client::new(),
            twitch_client
        }
    }

    pub async fn refresh_auth(&mut self) {
        let client_id = &self.twitch_client.id;
        let client_secret = &self.twitch_client.secret;
        let params = format!(
            "client_id={client_id}&client_secret={client_secret}&grant_type=client_credentials"
        );
        let post = format!("https://id.twitch.tv/oauth2/token?{params}");
        let post_result = self.reqwest_client.post(post).send().await;

        match post_result {
            Ok(response) => match response.json::<Auth>().await {
                Ok(auth) => {
                    self.auth = auth;
                    self.auth_refreshed_at = SystemTime::now();
                }
                Err(_) => eprintln!("Error refreshing auth. Failed to parse JSON response."),
            },
            Err(_) => eprintln!("Error refreshing auth. Failed to receive post response."),
        }
    }

    pub async fn query<T: DeserializeOwned>(
        &self,
        end_point: &str,
        body: &str,
    ) -> Result<T, reqwest::Error> {
        let response_result = self
            .reqwest_client
            .post(format!("https://api.igdb.com/v4/{end_point}"))
            .header(
                "Authorization",
                format!("{} {}", self.auth.token_type, self.auth.access_token),
            )
            .header("Client-ID", &self.twitch_client.id)
            .body(body.to_owned())
            .send()
            .await;

        match response_result {
            Ok(response) => response.json::<T>().await.and_then(|data| Ok(data)),
            Err(error) => {
                eprint!("Failed to query database: {:?}", error);
                if !self.is_auth_valid() {
                    eprint!("Auth is no longer valid!");
                }
                Err(error)
            }
        }
    }

    pub async fn find_similar_games(&self, game_ids: &Vec<u64>) -> Vec<Game> {
        let where_game_id_str = comma_sep(game_ids, |gid| (**gid).to_string());
        let query =format!(
            "
            fields similar_games.name, similar_games.genres, similar_games.themes, similar_games.player_perspectives;
            where id = ({where_game_id_str});
            limit 100; 
            "
            );
        let query_result = self.query::<Vec<Game>>("games", query.as_str()).await;
        match query_result {
            Ok(game_queries) => {
                let mut games: Vec<Game> = Vec::new();
                for query in game_queries {
                    for game in query.similar_games {
                        if games.iter().find(|g| g.id == game.id).is_none() {
                            games.push(game);
                        }
                    }
                }
                games
            }
            Err(_) => vec![],
        }
    }

    pub async fn find_games_from_ids(&self, game_ids: &Vec<u64>) -> Vec<Game> {
        let where_ids = comma_sep(game_ids, |id| (**id).to_string());
        let query = format!(
            "
            fields name, genres, themes, player_perspectives, first_release_date;
            where id = ({where_ids});
            limit 500;
            "
        );
        let query_result = self.query::<Vec<Game>>("games", query.as_str()).await;

        match query_result {
            Ok(games) => games,
            Err(_) => vec![],
        }
    }

    pub fn is_auth_valid(&self) -> bool {
        match self.auth_refreshed_at.elapsed() {
            Ok(duration) => duration.as_secs() < self.auth.expires_in,
            Err(_) => false,
        }
    }

    pub async fn search_game(&self, name: &String) -> Result<Vec<Game>, reqwest::Error> {
        let main_game = 0;
        let query = format!(
            r#"
            fields name, first_release_date;
            search "{name}";
            where version_parent = null & category = {main_game} & first_release_date != null;
            limit 20;
            "#
        );
        let result = self.query::<Vec<Game>>("games", query.as_str()).await?;
        Ok(result)
    }
}
