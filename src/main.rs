#![feature(async_closure)]
#![feature(never_type)]
#![feature(exhaustive_patterns)]
#![feature(once_cell)]

#![warn(clippy::pedantic)]

use std::env;
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

use serenity::async_trait;
use serenity::client::bridge::gateway::GatewayIntents;
use serenity::framework::standard::macros::*;
use serenity::framework::standard::CommandResult;
use serenity::framework::StandardFramework;
use serenity::model::channel::Message;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::Interaction::ApplicationCommand;
use serenity::model::prelude::{
    Interaction, InteractionApplicationCommandCallbackDataFlags, InteractionResponseType, Ready,
};
use serenity::prelude::{Context, EventHandler};
use serenity::Client;

use crate::commands::directions::DIRECTIONS_COMMAND;
use crate::commands::user_composition::USER_COMPOSITION_COMMAND;

mod commands;

#[group]
#[commands(explode, user_composition, directions)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, new_message: Message) {
        if new_message.channel_id.0 == 952722147099422730 {
            if let Err(err) = new_message.delete(ctx.http).await {
                println!("{:#?}", err);
            }
        }
    }

    async fn ready(&self, _ctx: Context, _ready: Ready) {
        println!("Carl has connected!");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let ApplicationCommand(interaction) = interaction {
            let command_data = &interaction.data;

            if let Err(why) = send_deferred_response(&ctx, &interaction).await {
                eprintln!("{:#?}", why);
            }

            if command_data.name == "userinfo" {
                if !command_data.options.is_empty() {
                    let Ok(user) = commands::user_lookup::user_from_interaction_option(
                        &command_data.options[0],
                    );

                    if let Err(why) = commands::user_lookup::send_user_info_command_response(
                        &ctx,
                        &interaction,
                        user,
                    )
                    .await
                    {
                        eprintln!("{:#?}", why);
                    }
                } else if let Err(why) = commands::user_lookup::send_user_info_command_response(
                    &ctx,
                    &interaction,
                    &interaction.user,
                )
                .await
                {
                    eprintln!("{:#?}", why);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Discord Token not in env_var!");
    let app_id: u64 = env::var("APPLICATION_ID")
        .expect("Application ID not in env_var!")
        .parse()
        .unwrap();

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("CARL "))
        .group(&GENERAL_GROUP);

    let mut client = Client::builder(token)
        .application_id(app_id)
        .event_handler(Handler)
        .framework(framework)
        .intents(
            GatewayIntents::non_privileged()
                | GatewayIntents::GUILD_MEMBERS
                | GatewayIntents::GUILD_PRESENCES,
        )
        .await
        .expect("Error creating client!");

    if let Err(why) = client.start().await {
        println!(
            "An error occurred while running the application: {:#?}",
            why
        )
    }
}

async fn send_deferred_response(
    ctx: &Context,
    interaction: &ApplicationCommandInteraction,
) -> serenity::Result<()> {
    interaction
        .create_interaction_response(&ctx.http, |r| {
            r.interaction_response_data(|d| {
                d.content("Carl is thinking")
                    .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
            })
            .kind(InteractionResponseType::DeferredChannelMessageWithSource)
        })
        .await
}

const CARL_END_GIF: &str =
    "https://tenor.com/view/explosion-mushroom-cloud-atomic-bomb-bomb-boom-gif-4464831";

#[command]
#[owners_only]
async fn explode(ctx: &Context, msg: &Message) -> CommandResult {
    let reply = msg.reply(&ctx.http, CARL_END_GIF).await?;
    sleep(Duration::from_secs(10));
    reply.delete(&ctx.http).await?;
    msg.delete(&ctx.http).await?;

    exit(0);
}
