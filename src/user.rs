use serde::Deserialize;
use std::iter::Map;
use serenity::prelude::TypeMapKey;
use std::collections::HashMap;
use crate::interactions::discord_get_request;

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

#[derive(Deserialize, Debug)]
pub struct RequestUser {
    id: String,
    username: String,
    avatar: String,
    discriminator: String,
    public_flags: u64,
}

impl RequestUser {
    pub async fn fetch(id: &str) -> Result<RequestUser, UserRetrievalError> {
        let pathway = format!("users/{}", id);

        Ok(serde_json::from_str::<RequestUser>(&discord_get_request(&pathway).await?)?)
    }
}

#[derive(Deserialize, Debug)]
struct BotApplication {
    id: String,
    name: String,
    description: String,
    summary: String,
    hook: bool,
    bot_public: bool,
    privacy_policy_url: String,
    flags: u64,
}

struct RequestBotUser {
    user: RequestUser,
    application: BotApplication,
    application_avatar: String,
}

impl RequestBotUser {
    // pub async fn fetch(id: &str) -> Result<RequestBotUser, UserRetrievalError> {
    //
    // }

    async fn fetch_bot_application(id: &str) -> Result<BotApplication, UserRetrievalError> {
        let pathway = format!("applications/{}/rpc", id);

        Ok(serde_json::from_str::<BotApplication>(&discord_get_request(&pathway).await?)?)
    }
}

pub struct CompleteUser {
    tag: String,
    nick_addon: String,
    status: String,
    id: String,
    created_at: String,
}

pub struct CachedUsers;

impl TypeMapKey for CachedUsers {
    type Value = HashMap<u64, CompleteUser>;
}

fn avatar_url(id: String, avatar_hash: String, size: u64) -> String {
    format!("https://cdn.discordapp.com/avatars/{}/{}.webp?size={}", id, avatar_hash, size)
}