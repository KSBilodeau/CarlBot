mod user_lookup;

use std::env;
use serenity::Client;
use serenity::async_trait;
use serenity::prelude::{EventHandler, Context};
use serenity::model::prelude::{Ready};
use serenity::model::interactions::Interaction;
use serenity::framework::StandardFramework;
use serenity::client::bridge::gateway::GatewayIntents;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Some(interaction_data) = interaction.data.as_ref() {
            match interaction_data.name.as_str() {
                "userlookup" => {
                    if let Err(why) = user_lookup::command(ctx, interaction).await {
                        println!("{:#?}", why);
                    }
                }
                _ => {}
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