use std::env;

use serenity::async_trait;
use serenity::Client;
use serenity::client::bridge::gateway::GatewayIntents;
use serenity::framework::StandardFramework;
use serenity::model::interactions::Interaction;
use serenity::model::interactions::InteractionData::{ApplicationCommand, MessageComponent};
use serenity::model::prelude::Ready;
use serenity::prelude::{Context, EventHandler};

use interactions::user_lookup;

mod interactions;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Some(interaction_data) = interaction.data.as_ref() {
            if let ApplicationCommand(command_data) = interaction_data {
                match command_data.name.as_str() {
                    "userinfo" => {
                        if let Err(why) = user_lookup::UserInfoCommand::new(&ctx, &interaction)
                            .execute(&command_data).await {
                            eprintln!("{:#?}", why);
                        }
                    },
                    _ => {}
                }
            } else if let MessageComponent(_component_data) = interaction_data {

            }
        } else {
            println!("No data was sent!");
        }
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Discord Token not in env_var!");

    let framework = StandardFramework::new();

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
