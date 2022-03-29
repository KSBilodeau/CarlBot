use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;
use serenity::prelude::Context;
use std::env;
use std::thread::sleep;
use std::time::Duration;

pub mod directions;
pub mod user_composition;
pub mod user_lookup;

pub async fn discord_get_pathway(pathway: &str) -> reqwest::Result<String> {
    let pathway = format!("https://discord.com/api/v9/{}", pathway);
    let token = env::var("DISCORD_TOKEN").expect("Discord Token not in env_var!");

    Ok(reqwest::Client::new()
        .get(pathway)
        .header("Authorization", token)
        .send()
        .await?
        .text()
        .await?)
}

async fn text_command_failure(ctx: &Context, msg: &Message, err: &str) -> CommandResult {
    let reply = msg.reply(&ctx.http, err).await?;

    sleep(Duration::from_secs(10));

    msg.delete(&ctx.http).await?;
    reply.delete(&ctx.http).await?;

    Ok(())
}

fn avatar_url(id: &str, avatar_hash: &str, size: u64) -> String {
    format!(
        "https://cdn.discordapp.com/avatars/{}/{}.webp?size={}",
        id, avatar_hash, size
    )
}

fn text_command_message_options(msg: &Message) -> Vec<&str> {
    msg.content.split_whitespace().skip(3).collect()
}
