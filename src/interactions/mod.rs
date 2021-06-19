use std::env;
use crate::user::UserRetrievalError;

pub mod user_lookup;

pub async fn discord_get_request(pathway: &str) -> Result<String, UserRetrievalError> {
    let pathway = format!("https://discord.com/api/v9/{}", pathway);
    let token = env::var("AUTH").expect("Discord Token not in env_var!");

    Ok(reqwest::Client::new()
        .get(pathway)
        .header("Authorization", token)
        .send()
        .await?
        .text()
        .await?)
}