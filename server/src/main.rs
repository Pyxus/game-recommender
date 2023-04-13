pub mod twitch;

use std::env;
use dotenv::dotenv;
use twitch::igdb;


#[tokio::main]
async fn main() {
    dotenv().ok();
    let client_id = env::var("TWITCH_CLIENT_ID").expect("Failed to get twitch client id.");
    let client_secret = env::var("TWITCH_CLIENT_SECRET").expect("Failed to get twitch client secret.");
    let mut igdb_wrapper = igdb::IGDBWrapper::new(client_id, client_secret);
    igdb_wrapper.refresh_auth().await;

    let response = igdb_wrapper.query("", "fields name;")
        .await
        .text()
        .await
        .expect("");

    println!("{}", response);
}