use serde::{Deserialize};
use serenity::model::prelude::{User, ApplicationCommandInteractionData, InteractionApplicationCommandCallbackDataFlags};
use serenity::model::interactions::ApplicationCommandInteractionDataOptionValue::User as AppValUser;
use serenity::model::interactions::Interaction;
use serenity::prelude::Context;
use crate::interactions::{UserRetrievalError, make_get_request};
use serenity::model::prelude::InteractionResponseType::DeferredChannelMessageWithSource;

pub struct UserInfoCommand<'a> {
    ctx: &'a Context,
    interaction: &'a Interaction
}

impl<'a> UserInfoCommand<'a> {
    pub fn new(ctx: &'a Context, interaction: &'a Interaction) -> UserInfoCommand<'a> {
        UserInfoCommand {
            ctx,
            interaction,
        }
    }

    pub async fn execute(&self, command_data: &ApplicationCommandInteractionData) -> Result<(), UserRetrievalError> {
        self.interaction.create_interaction_response(&self.ctx.http, |r| {
            r.kind(DeferredChannelMessageWithSource)
                .interaction_response_data(|d| {
                    d.content("Carl is thinking")
                })
        }).await?;

        let target_user;

        if command_data.options.len() > 0 {
            target_user = Some(TargetUser::from_interaction_option(&self, &command_data, 0).await);
        } else {
            target_user = Some(TargetUser::from_interaction_user(&self).await);
        }

        let target_user = target_user.unwrap();

        if let Ok(user) = &target_user {
            let mut bot_application = None;

            if user.bot {
                bot_application = Some(TargetUser::get_bot_application(user.user_data.id.parse().unwrap()).await?);
            }

            println!("followup");
            self.interaction.create_followup_message(&self.ctx.http, |f| {
                f.create_embed(|e| {
                    e.title(format!("{}#{}", &user.user_data.username, &user.user_data.discriminator))
                        .thumbnail(&user.avatar)
                        .url(&user.avatar)
                        .field("ID", &user.user_data.id, false);
                    if !user.status.is_empty() {
                        e.description(&user.status);
                    }
                    if user.bot {
                        e.field("Verified Bot", user.is_verified_bot, false)
                            .footer(|f| {
                                let app = bot_application.unwrap();
                                let url = format!("https://cdn.discordapp.com/avatars/{}/{}.webp?size=1024", app.id, app.icon);
                                println!("{}", url);
                                f.icon_url(url)
                                    .text(format!("Brought to you by {}!", app.name))
                            });
                    }
                    e.field("Account Creation Date", &user.creation_date, false)
                })
            }).await?;
        } else if let Err(why) = target_user {
            eprintln!("{:#?}", why);
        }

        Ok(())
    }
}

#[derive(Deserialize, Debug)]
struct PartialUser {
    id: String,
    username: String,
    discriminator: String,
    public_flags: u32,
}

impl PartialUser {
    pub async fn fetch_from_id(id: u64) -> Result<PartialUser, UserRetrievalError> {
        let pathway = format!("https://discord.com/api/v9/users/{}", id);
        Ok(serde_json::from_str::<PartialUser>(&make_get_request(&pathway).await?)?)
    }
}

#[derive(Deserialize, Debug)]
pub struct Application {
    id: String,
    name: String,
    icon: String,
}

#[derive(Debug)]
pub struct TargetUser {
    user_data: PartialUser,
    nick: String,
    status: String,
    avatar: String,
    creation_date: String,
    bot: bool,
    is_verified_bot: bool,
}

impl TargetUser {
    pub async fn from_interaction_option(command: &UserInfoCommand<'_>, data: &ApplicationCommandInteractionData, option_num: usize) -> Result<TargetUser, UserRetrievalError> {
        assert!(option_num < data.options.len());

        let mut target_user = None;

        if let AppValUser(user, _) = data.options[option_num].resolved.as_ref().unwrap() {
            target_user = Some(TargetUser::from_user(&command, &user).await?);
        }

        Ok(target_user.unwrap())
    }

    pub async fn from_interaction_user(command: &UserInfoCommand<'_>) -> Result<TargetUser, UserRetrievalError> {
        let target_user;

        if command.interaction.guild_id.is_some() {
            target_user = Some(&command.interaction.member.as_ref().unwrap().user);
        } else {
            target_user = Some(&command.interaction.user.as_ref().unwrap());
        }

        Ok(TargetUser::from_user(&command, &target_user.unwrap()).await?)
    }

    async fn from_user(command: &UserInfoCommand<'_>, user: &User) -> Result<TargetUser, UserRetrievalError> {
        let nick = TargetUser::nick(&command, &user).await;
        let status = TargetUser::status(&command, &user).await;

        let user_data = PartialUser::fetch_from_id(user.id.0).await?;

        let mut is_verified_bot = false;

        if user.bot {
            is_verified_bot = TargetUser::is_verified_bot(&user);
        }

        Ok(TargetUser {
            user_data,
            nick,
            status,
            avatar: user.face(),
            creation_date: user.created_at().to_string(),
            bot: user.bot,
            is_verified_bot,
        })
    }

    async fn nick(command: &UserInfoCommand<'_>, user: &User) -> String {
        if command.interaction.guild_id.is_some() {
            if let Some(nick) = user.nick_in(&command.ctx.http, command.interaction.guild_id.unwrap()).await {
                return nick;
            }
        }

        String::new()
    }

    async fn status(command: &UserInfoCommand<'_>, user: &User) -> String {
        let mut result = String::new();

        if command.interaction.guild_id.is_some() {
            let presence = command.ctx.cache
                .guild_field(command.interaction.guild_id.unwrap(), |f| {
                    if let Some(val) = f.presences.get(&user.id) {
                        Some(val.clone())
                    } else {
                        None
                    }
                }).await;

            if presence.is_some() {
                let presence = presence.unwrap();

                if presence.is_some() {
                    let presence = presence.unwrap();

                    if presence.activities.len() > 0 {
                        if let Some(emoji) = &presence.activities[0].emoji {
                            if emoji.id.is_some() {
                                result = format!("<:{}:{}> ", emoji.name, emoji.id.unwrap());
                            } else {
                                result = format!("{} ", emoji.name);
                            }
                        }
                        if let Some(status) = presence.activities[0].state.as_ref() {
                            result = format!("{}{}", result, status);
                        }
                    }
                }
            }
        }

        result
    }

    async fn get_bot_application(id: u64) -> Result<Application, UserRetrievalError> {
        let pathway = format!("https://discord.com/api/v9/applications/{}/rpc", id);

        Ok(serde_json::from_str::<Application>(&make_get_request(&pathway).await?)?)
    }

    fn is_verified_bot(user: &User) -> bool {
        user.public_flags.unwrap().bits & (1 << 16) == (1 << 16)
    }
}

impl Default for TargetUser {
    fn default() -> Self {
        TargetUser{
            user_data: PartialUser {
                id: Default::default(),
                username: Default::default(),
                discriminator: Default::default(),
                public_flags: 0
            },
            nick: Default::default(),
            status: Default::default(),
            avatar: Default::default(),
            creation_date: Default::default(),
            bot: false,
            is_verified_bot: false
        }
    }
}