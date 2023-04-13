pub mod igdb {
    use std::time::SystemTime;
    use serde::{Serialize, Deserialize};
    use reqwest::Response;

    #[derive(Serialize, Deserialize, Default)]
    pub struct Auth {
        pub access_token: String,
        pub expires_in: u64,
        pub token_type: String,
    }

    pub struct IGDBWrapper {
        auth: Auth,
        auth_refreshed_at: SystemTime,
        client: reqwest::Client,
        client_id: String,
        client_secret: String,
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
            let post = format!("https://id.twitch.tv/oauth2/token?client_id={client_id}&client_secret={client_secret}&grant_type=client_credentials");

            self.auth = self
                .client
                .post(post)
                .send()
                .await
                .expect("Post request failed.")
                .json::<Auth>()
                .await
                .expect("Failed to retreive Auth from JSON.");
            
            self.auth_refreshed_at = SystemTime::now();
        }

        pub async fn query(&self, end_point: &str, body: &str) -> Response {
            return self
                .client
                .post(format!("https://api.igdb.com/v4/games/{end_point}"))
                .header(
                    "Authorization",
                    format!("{} {}", self.auth.token_type, self.auth.access_token),
                )
                .header("Client-ID", &self.client_id)
                .body(body.to_owned())
                .send()
                .await
                .expect("Failed to query database.");
        }

        pub fn is_auth_valid(&self) -> bool {
            self.auth.expires_in > 0
                && SystemTime::now()
                    .elapsed()
                    .expect("Failed to get elapsed time")
                    .as_secs()
                    < self.auth.expires_in
        }
    }
}
