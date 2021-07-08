use std::num::ParseIntError;

use serde::Deserialize;
use serenity::builder::CreateEmbed;
use serenity::framework::standard::CommandResult;
use serenity::framework::standard::macros::command;
use serenity::model::channel::Message;
use serenity::model::channel::MessageType::InlineReply;
use serenity::model::id::GuildId;
use serenity::model::prelude::{InteractionApplicationCommandCallbackDataFlags, UserPublicFlags};
use serenity::model::prelude::application_command::{ApplicationCommandInteraction, ApplicationCommandInteractionDataOption, ApplicationCommandInteractionDataOptionValue};
use serenity::model::user::User;
use serenity::prelude::Context;

use crate::commands::{discord_get_pathway, avatar_url};
use crate::commands::user_lookup::UserRetrievalError::{InvalidCommandByReply, InvalidInteractionOption, InvalidUserId, Serenity, Reqwest, SerdeJson};

#[derive(Debug)]
pub enum UserRetrievalError {
    Serenity(serenity::Error),
    Reqwest(reqwest::Error),
    SerdeJson(serde_json::Error),
    InvalidCommandByReply,
    InvalidUserId,
    InvalidInteractionOption,
}

impl From<serenity::Error> for UserRetrievalError {
    fn from(err: serenity::Error) -> Self {
        Serenity(err)
    }
}

impl From<ParseIntError> for UserRetrievalError {
    fn from(_: ParseIntError) -> Self {
        InvalidUserId
    }
}

impl From<reqwest::Error> for UserRetrievalError {
    fn from(err: reqwest::Error) -> Self {
        Reqwest(err)
    }
}

impl From<serde_json::Error> for UserRetrievalError {
    fn from(err: serde_json::Error) -> Self {
        SerdeJson(err)
    }
}

#[derive(Deserialize, Debug)]
struct BotApplication {
    icon: String,
    name: String,
    flags: u64,
}

pub async fn user_from_message(ctx: &Context, msg: &Message) -> Result<User, UserRetrievalError> {
    if let InlineReply = msg.kind {
        Err(InvalidCommandByReply)
    }

    if msg.mentions.len() > 0 {
        Ok(msg.mentions.first().unwrap().clone())
    } else {
        let contents: Vec<&str> = msg.content.split_whitespace().take(3).collect();
        println!("{:#?}", contents);
        let user_id = contents[2].parse::<u64>()?;

        Ok(ctx.http.get_user(user_id).await?)
    }
}

pub async fn user_from_interaction_option(option: &ApplicationCommandInteractionDataOption) -> Result<&User, UserRetrievalError> {
    if let Some(value) = option.resolved.as_ref() {
        if let ApplicationCommandInteractionDataOptionValue::User(user, _) = value {
            Ok(&user)
        } else {
            Err(InvalidInteractionOption)
        }
    } else {
        Err(InvalidInteractionOption)
    }
}

pub async fn send_user_info_command_response(ctx: &Context, interaction: &ApplicationCommandInteraction, user: &User) -> Result<(), UserRetrievalError> {
    let nick_addon = nick_addon(&ctx, &interaction.guild_id, &user).await;
    let application: Option<BotApplication>;

    match bot_application(&user).await {
        Ok(app) => application = Some(app),
        Err(why) => {
            eprintln!("{:#?}", why);
            application = None;
        }
    }

    interaction.create_followup_message(&ctx.http, |f| {
        f.content(format!("```json\n{:#?}\n```", user))
            .create_embed(|e| {
                create_user_embed(e, &user, &nick_addon, application)
            })
            .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
    }).await?;

    Ok(())
}

#[command]
pub async fn user_info(ctx: &Context, msg: &Message) -> CommandResult {
    let user = user_from_message(ctx, msg).await;

    if let Ok(user) = user.as_ref() {
        let nick_addon = nick_addon(&ctx, &msg.guild_id, &user).await;
        let application: Option<BotApplication>;

        match bot_application(&user).await {
            Ok(app) => application = Some(app),
            Err(_) => application = None
        }

        msg.channel_id.send_message(&ctx.http, |m| {
            m.reference_message(msg)
                .allowed_mentions(|m| {
                    m.empty_parse()
                })
                .content(format!("```json\n{:#?}\n```", user))
                .embed(|e| {
                    create_user_embed(e, &user, &nick_addon, application)
                })
        }).await?;
    } else if let Err(err) = user {
        match err {
            UserRetrievalError::Serenity(err) => {
                println!("{:#?}", err);
                msg.reply(&ctx.http, "Command failed, ").await?;
            },
            UserRetrievalError::InvalidUserId => {
                msg.reply(&ctx.http, "Must pass a valid/existing user id!").await?;
            },
            UserRetrievalError::InvalidCommandByReply => {
                msg.reply(&ctx.http, "I'm sorry, but the user info command does not support replies due to ambiguity.").await?;
            }
            _ => {}
        }
    }

    Ok(())
}

async fn bot_application(user: &User) -> Result<BotApplication, UserRetrievalError>{
    let app_string = discord_get_pathway(&format!("/applications/{}/rpc", user.id.0)).await?;

    Ok(serde_json::from_str::<BotApplication>(&app_string)?)
}

async fn nick_addon(ctx: &Context, guild_id: &Option<GuildId>, user: &User) -> String {
    if guild_id.is_some() {
        if let Some(nick) = user.nick_in(&ctx.http, guild_id.unwrap()).await {
            return format!("aka {}", nick);
        }
    }

    String::new()
}

fn user_house(user: &User) -> &'static str {
    if let Some(flags) = user.public_flags.as_ref() {
        if flags.contains(UserPublicFlags::HOUSE_BRAVERY) {
            return "<:hypesquad_bravery:861391996567158784> Bravery";
        } else if flags.contains(UserPublicFlags::HOUSE_BRILLIANCE) {
            return "<:hypesquad_brilliance:861392654994112533> Brilliance";
        } else if flags.contains(UserPublicFlags::HOUSE_BALANCE) {
            return "<:hypesquad_balance:861392618397892628> Balance";
        }
    }

    "None"
}

fn create_user_embed<'a>(embed: &'a mut CreateEmbed, user: &User, nick_addon: &str, application: Option<BotApplication>) -> &'a mut CreateEmbed {
    if let Some(application) = application {
        embed.footer(|f| {
            f.icon_url(avatar_url(&user.id.0.to_string(), &application.icon, 1024))
                .text(format!("Brought you by {}!", application.name))
        });
    }

    embed.title(format!("{} {}", user.tag(), nick_addon))
        .url(user.face())
        .thumbnail(user.face())
        .field("Id:", user.id.0, false)
        .field("Hypesquad House:", user_house(&user), false)
        .field("Created:", format!("<t:{}:f>", user.created_at().timestamp()), false)
}