use std::env;
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

use serenity::async_trait;
use serenity::Client;
use serenity::client::bridge::gateway::GatewayIntents;
use serenity::framework::standard::CommandResult;
use serenity::framework::standard::macros::*;
use serenity::framework::StandardFramework;
use serenity::model::channel::Message;
use serenity::model::prelude::{Interaction, InteractionApplicationCommandCallbackDataFlags, InteractionResponseType, Ready};
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::Interaction::{ApplicationCommand, Ping};
use serenity::model::prelude::Interaction::MessageComponent;
use serenity::prelude::{Context, EventHandler};

use crate::commands::user_lookup::USER_INFO_COMMAND;

mod commands;

#[group]
#[commands(explode, user_info)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, _ready: Ready) {
        println!("Carl has connected!");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            ApplicationCommand(interaction) => {
                let command_data = &interaction.data;

                if let Err(why) = send_deferred_response(&ctx, &interaction).await {
                    eprintln!("{:#?}", why);
                }

                match command_data.name.as_str() {
                    "userinfo" => {
                        if command_data.options.len() > 0 {
                            if let Ok(user) = commands::user_lookup::user_from_interaction_option(&command_data.options[0]).await {
                                if let Err(why) = commands::user_lookup::send_user_info_command_response(&ctx, &interaction, &user).await {
                                    eprintln!("{:#?}", why);
                                }
                            }
                        } else {
                            if let Err(why) = commands::user_lookup::send_user_info_command_response(&ctx, &interaction, &interaction.user).await {
                                eprintln!("{:#?}", why);
                            }
                        }
                    },
                    _ => {}
                }
            },
            MessageComponent(_component) => {

            },
            Ping(_ping) => {

            },
        }
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Discord Token not in env_var!");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("CARL "))
        .group(&GENERAL_GROUP);

    let mut client = Client::builder(token)
        .application_id(820807331016081439)
        .event_handler(Handler)
        .framework(framework)
        .intents(GatewayIntents::non_privileged() | GatewayIntents::GUILD_MEMBERS | GatewayIntents::GUILD_PRESENCES)
        .await
        .expect("Error creating client!");

    if let Err(why) = client.start().await {
        println!("An error occurred while running the application: {:#?}", why)
    }
}

async fn send_deferred_response(ctx: &Context, interaction: &ApplicationCommandInteraction) -> serenity::Result<()> {
    interaction.create_interaction_response(&ctx.http, |r| {
        r.interaction_response_data(|d| {
            d.content("Carl is thinking")
                .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
        })
            .kind(InteractionResponseType::DeferredChannelMessageWithSource)
    }).await
}

const CARL_END_GIF: &str = "https://tenor.com/view/explosion-mushroom-cloud-atomic-bomb-bomb-boom-gif-4464831";

#[command]
#[owners_only]
async fn explode(ctx: &Context, msg: &Message) -> CommandResult {
    let reply = msg.reply(&ctx.http, CARL_END_GIF).await?;
    sleep(Duration::from_secs(10));
    reply.delete(&ctx.http).await?;
    msg.delete(&ctx.http).await?;

    exit(0);
}