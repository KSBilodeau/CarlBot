use std::env;

pub mod user_lookup;

#[derive(Debug)]
pub enum UserRetrievalError {
    RequestError(reqwest::Error),
    JsonParsingError(serde_json::Error),
    SerenityError(serenity::Error),
}

impl From<reqwest::Error> for UserRetrievalError {
    fn from(error: reqwest::Error) -> Self {
        UserRetrievalError::RequestError(error)
    }
}

impl From<serde_json::Error> for UserRetrievalError {
    fn from(error: serde_json::Error) -> Self {
        UserRetrievalError::JsonParsingError(error)
    }
}

impl From<serenity::Error> for UserRetrievalError {
    fn from(error: serenity::Error) -> Self {
        UserRetrievalError::SerenityError(error)
    }
}

pub async fn make_get_request(pathway: &str) -> Result<String, UserRetrievalError> {
    let token = env::var("AUTH").expect("Discord Token not in env_var!");

    Ok(reqwest::Client::new()
        .get(pathway)
        .header("Authorization", token)
        .send()
        .await?
        .text()
        .await?)
}