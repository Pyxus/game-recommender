use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::time::SystemTime;
use strum_macros::{EnumCount, EnumIter};

#[repr(u64)]
#[derive(Debug, Copy, Clone, EnumIter, EnumCount)]
pub enum Genres {
    PointAndClick = 2,
    Fighting = 4,
    Shooter = 5,
    Music = 7,
    Platform = 8,
    Puzzle = 9,
    Racing = 10,
    RTS = 11,
    RPG = 12,
    Simulator = 13,
    Sport = 14,
    Strategy = 15,
    TBS = 16,
    Tactical = 24,
    HackNSlash = 25,
    Trivia = 26,
    Pinball = 30,
    Adventure = 31,
    Indie = 32,
    Arcade = 33,
    VisualNovel = 34,
    CardAndBoardGame = 35,
    MOBA = 36,
}

#[repr(u64)]
#[derive(Debug, Copy, Clone, EnumIter, EnumCount)]
pub enum Themes {
    Action = 1,
    Fantasy = 17,
    SciFi = 18,
    Horror = 19,
    Thriller = 20,
    Survival = 21,
    Historical = 22,
    Stealth = 23,
    Comedy = 27,
    Business = 28,
    Drama = 31,
    NonFiction = 32,
    Sandbox = 33,
    Educational = 34,
    Kids = 35,
    OpenWorld = 38,
    Warfare = 39,
    Party = 40,
    FourX = 41,
    Erotic = 42,
    Mystery = 43,
    Romance = 44,
}

#[repr(u64)]
#[derive(Debug, Copy, Clone, EnumIter, EnumCount)]
pub enum PlayerPerspective {
    FirstPerson = 1,
    ThirdPerson = 2,
    Isometric = 3,
    SideView = 4,
    Text = 5,
    Auditory = 6,
    VR = 7,
}

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

pub struct IGDBWrapper {
    auth: Auth,
    auth_refreshed_at: SystemTime,
    client: reqwest::Client,
    client_id: String,
    client_secret: String,
}

#[derive(Serialize, Deserialize, Default)]
struct Auth {
    access_token: String,
    expires_in: u64,
    token_type: String,
}

impl IGDBWrapper {
    pub fn new(client_id: String, client_secret: String) -> Self {
        IGDBWrapper {
            auth: Auth::default(),
            auth_refreshed_at: SystemTime::now(),
            client: reqwest::Client::new(),
            client_id,
            client_secret,
        }
    }

    pub async fn refresh_auth(&mut self) {
        let client_id = &self.client_id;
        let client_secret = &self.client_secret;
        let params = format!("client_id={client_id}&client_secret={client_secret}&grant_type=client_credentials");
        let post = format!("https://id.twitch.tv/oauth2/token?{params}");
        let post_result = self.client.post(post).send().await;

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
            .client
            .post(format!("https://api.igdb.com/v4/{end_point}"))
            .header(
                "Authorization",
                format!("{} {}", self.auth.token_type, self.auth.access_token),
            )
            .header("Client-ID", &self.client_id)
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
        let where_game_id_str = self.comma_sep(game_ids, |gid| (**gid).to_string());
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
        let where_ids = self.comma_sep(game_ids, |id| (**id).to_string());
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

    fn comma_sep<T, F, C>(&self, collection: C, f: F) -> String
    where
        F: FnMut(&T) -> String,
        C: IntoIterator<Item = T>,
    {
        let coll_vec: Vec<T> = collection.into_iter().collect();
        coll_vec.iter().map(f).collect::<Vec<String>>().join(", ")
    }
}
