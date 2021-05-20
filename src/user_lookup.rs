use serenity::prelude::Context;
use serenity::model::prelude::{Interaction, User as SUser, ApplicationCommandInteractionDataOptionValue::User as AUser, InteractionApplicationCommandCallbackDataFlags};
use serde::{Serialize, Deserialize};
use std::env;
use serenity::model::id::GuildId;

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: String,
    username: String,
    avatar: String,
    discriminator: String,
    public_flags: u32,
}

pub async fn command(ctx: Context, interaction: Interaction) -> Result<(), serenity::Error> {
    let options = interaction.data.as_ref().unwrap().options.clone();

    if let AUser(s_user, _) = options[0].resolved.as_ref().unwrap() {
        let user = user(&s_user).await.unwrap();
        let nick = nickname(&ctx, &interaction.guild_id.as_ref().unwrap(), &s_user, &user).await;
        let status = status(&ctx, &interaction, s_user).await;
        let discriminator = format!("#{}", user.discriminator);

        interaction.create_interaction_response(&ctx.http, |r| {
            r.interaction_response_data(|d| {
                d.embed(|e| {
                    e.title(nick)
                        .thumbnail(s_user.face())
                        .description("Want my number? <:catsnrk:815087234772369449>")
                        .field("Status", status, false)
                        .field("ID", user.id, true)
                        .field("Discriminator", discriminator, true)
                        .url(s_user.face())
                })
                    .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
            })
        }).await?;
    }

    Ok(())
}

async fn status(ctx: &Context, interaction: &Interaction, user: &SUser) -> String {
    let presences = ctx.cache.guild_field(interaction.guild_id.unwrap(), |f| f.presences.clone()).await;
    let mut status = String::new();

    if let Some(presences) = presences {
        let presence = presences.get(&user.id);

        if let Some(presence) = presence {
            if presence.activities.len() > 0 {
                let activity = presence.activities[0].clone();

                if let Some(emoji) = activity.emoji {
                    if let Some(emoji_id) = emoji.id {
                        status.push_str(format!("<:{}:{}> ", emoji.name, emoji_id).as_str());
                    } else {
                        status.push_str(format!("{} ", emoji.name).as_str());
                    }

                    if let Some(state) = activity.state {
                        status.push_str(&state);
                    }
                } else {
                    if let Some(state) = activity.state {
                        status.push_str(&state);
                    } else {
                        status.push_str("(None Available)");
                    }
                }
            } else {
                status.push_str("(None Available)");
            }
        } else {
            status.push_str("(None Available)");
        }
    } else {
        status.push_str("(None Available)");
    }

    status
}

async fn nickname(ctx: &Context, id: &GuildId, s_user: &SUser, user: &User) -> String {
    let bot_verified_emoji = "<:bot1:844733440481820673><:bot2:844733440665845780>";
    let bot_emoji = "<:bot3:844702520646696991><:bot4:844702593002242068>";

    if let Some(nick) = s_user.nick_in(&ctx.http, id).await {
        if s_user.bot && !is_verified_bot(&user) {
            format!("{} aka {} {}", user.username, nick, bot_emoji)
        } else if s_user.bot && is_verified_bot(&user) {
            format!("{} aka {} {}", user.username, nick, bot_verified_emoji)
        } else {
            format!("{} aka {}", user.username, nick)
        }
    } else {
        if s_user.bot && !is_verified_bot(&user) {
            format!("{} {}", user.username, bot_emoji)
        } else if s_user.bot && is_verified_bot(&user) {
            format!("{} {}", user.username, bot_verified_emoji)
        } else {
            format!("{}", user.username)
        }
    }
}

async fn user(user: &SUser) -> Result<User, reqwest::Error> {
    let url = format!("https://discord.com/api/v9/users/{}", user.id.0);

    let token = env::var("AUTH").expect("Discord Token not in env_var!");

    let body = reqwest::Client::new()
        .get(url)
        .header("Authorization", token)
        .send()
        .await?
        .text()
        .await?;

    let result:User = serde_json::from_str(&body)
        .expect("User returned invalid response!");

    Ok(result)
}

fn is_verified_bot(user: &User) -> bool {
    if user.public_flags & (1 << 16) == (1 << 16){
        true
    } else {
        false
    }
}