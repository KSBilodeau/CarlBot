use serenity::prelude::Context;
use serenity::model::prelude::{Interaction, InteractionApplicationCommandCallbackDataFlags, User, GuildId, UserPublicFlags};
use serenity::model::prelude::ApplicationCommandInteractionDataOptionValue::User as AppIntUser;
use serenity::http::{Http};
use std::sync::Arc;
use serenity::builder::CreateEmbed;
use serde::{Serialize, Deserialize};
use serenity::model::id::UserId;

struct StringUser {
    nick: String,
    id: String,
    discriminator: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RawApplication {
    pub id: String,
    pub name: String,
    pub description: String,
    pub summary: String,
    hook: bool,
    pub bot_public: bool,
    bot_require_code_grant: bool,
    pub privacy_policy_url: String,
    verify_key: String,
    pub flags: u32,
}

impl StringUser {
    async fn from_interaction_user(user: &User, http: &Arc<Http>, guild_id: &Option<GuildId>) -> StringUser {
        let nick = user.nick_in(http, guild_id.unwrap().0).await;
        let nick = match nick {
            Some(val) => val,
            None => "".to_string()
        };

        StringUser {
            nick,
            id: format!("{:0>18}", user.id.0),
            discriminator: format!("#{:0>4}", user.discriminator),
        }
    }
}


pub async fn command(ctx: Context, interaction: Interaction) -> Result<(), serenity::Error> {
    if let Some(data) = &interaction.data {
        if let AppIntUser(user, _) = data.options[0].resolved.as_ref().unwrap() {
            let str_user = StringUser::from_interaction_user(user, &ctx.http, &interaction.guild_id).await;
            let icon = application_avatar(&ctx).await?;
            let status = status(&ctx, &interaction.guild_id.unwrap(), &user).await;
            let bot_flags;

            if user.bot {
                if let Ok(option) = public_flags(&user.id).await {
                    if let Some(application) = option {
                        bot_flags = Some(application.flags);
                    } else {
                        bot_flags = None;
                    }
                } else {
                    bot_flags = None;
                }
            } else {
                bot_flags = None;
            }


            interaction.create_interaction_response(&ctx.http, |r| {
                r.interaction_response_data(|d| {
                    d.embed(|e| {
                        // if user.bot {
                        //     embed_title_bot(e, &str_user.nick, user, bot_flags)
                        // } else {
                        //     embed_title_user(e, &str_user.nick, user);
                        // }

                        e.thumbnail(user.face())
                            .description("Want my number? <:catsnrk:815087234772369449>")
                            .field("Status", status, false)
                            .field("ID", str_user.id, true)
                            .field("Discriminator", str_user.discriminator, true)
                            .footer(|f| {
                                f.icon_url(icon)
                                    .text("Brought to you by Cuddle's Robotic Services")
                            })
                    })
                        .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                })
            }).await?
        }
    }

    Ok(())
}

async fn application_avatar(ctx: &Context) -> Result<String, serenity::Error> {
    let app_info = ctx.http.get_current_application_info().await?;

    Ok(avatar_url(app_info.id.0, &app_info.icon.unwrap()))
}

fn avatar_url(id: u64, img_hash: &str) -> String {
    format!("https://cdn.discordapp.com/avatars/{}/{}.webp?size=1024", id, img_hash)
}

async fn status(ctx: &Context, id: &GuildId, user: &User) -> String {
    let presences = &ctx.cache.guild_field(id, |f| {
        f.presences.clone()
    }).await;

    let default_status = String::from("(None Available)");
    let mut custom_status = String::new();

    let custom_status = match presences {
        Some(presences) => {
            if let Some(presence) =  presences.get(&user.id) {
                if presence.activities.len() > 0 {
                    let activity = &presence.activities[0].clone();

                    if let Some(emoji) = &activity.emoji {
                        custom_status.push_str(format!("<:{}:{}> ", emoji.name, emoji.id.unwrap()).as_str());
                    }

                    if let Some(text) = &activity.state {
                        custom_status.push_str(text);
                    }

                    custom_status
                } else {
                    default_status
                }
            } else {
                default_status
            }
        },
        None => default_status,
    };

    custom_status
}

fn embed_title_user(embed: &mut CreateEmbed, nick: &str, user: &User) {
    if nick.is_empty() {
        embed.title(format!("{}", user.name));
    } else {
        embed.title(format!("{} aka {}", user.name, nick));
    }

    embed.url(user.face());
}

fn embed_title_bot(embed: &mut CreateEmbed, nick: &str, user: &User, flags: Option<u32>) {
    let bot_verified_emoji = "<:bot1:844387373848461332><:bot2:844387418483720202>";
    let bot_emoji = "<:bot3:844441417841377282><:bot4:844441475537567799>";

    if let Some(flags) = flags {
        if nick.is_empty() {
            if ((flags >> 12) & (UserPublicFlags::VERIFIED_BOT.bits >> 13)) == (UserPublicFlags::VERIFIED_BOT.bits >> 13) {
                embed.title(format!("{} {}", user.name, bot_verified_emoji));
            } else {
                embed.title(format!("{} {}", user.name, bot_emoji));
            }
        } else {
            println!("B");
            if ((flags >> 12) & (UserPublicFlags::VERIFIED_BOT.bits >> 12)) == (UserPublicFlags::VERIFIED_BOT.bits >> 13) {
                embed.title(format!("{} aka {}", user.name, bot_verified_emoji));
            } else {
                embed.title(format!("{} aka {}", user.name, bot_emoji));
            }
        }
    } else {
        println!("H");
        if nick.is_empty() {
            embed.title(format!("{} {}", user.name, bot_emoji));
        } else {
            embed.title(format!("{} aka {}", user.name, bot_emoji));
        }
    }

    embed.url(user.face());
}

pub async fn public_flags(id: &UserId) -> Result<Option<RawApplication>, reqwest::Error> {
    let url = format!("https://discord.com/api/v9/applications/{}/rpc", id.0);
    let body = reqwest::get(url)
        .await?
        .text()
        .await?;

    let result = serde_json::from_str::<RawApplication>(body.as_str());

    if let Ok(v) =  result {
        return Ok(Some(v));
    } else if let Err(why) = result {
        eprintln!("{:#?}", why);
    }

    Ok(None)
}