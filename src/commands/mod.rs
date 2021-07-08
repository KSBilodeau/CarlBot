use std::env;

pub mod user_lookup;

pub async fn discord_get_pathway(pathway: &str) -> reqwest::Result<String> {
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

fn avatar_url(id: &str, avatar_hash: &str, size: u64) -> String {
    format!("https://cdn.discordapp.com/avatars/{}/{}.webp?size={}", id, avatar_hash, size)
}