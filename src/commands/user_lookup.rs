use serde::Deserialize;
use serenity::builder::CreateEmbed;
use serenity::model::prelude::application_command::{
    ApplicationCommandInteraction, ApplicationCommandInteractionDataOption,
    ApplicationCommandInteractionDataOptionValue,
};
use serenity::model::prelude::{
    InteractionApplicationCommandCallbackDataFlags, Member, UserPublicFlags,
};
use serenity::model::user::User;
use serenity::prelude::Context;

use crate::commands::{avatar_url, discord_get_pathway};

#[derive(Debug, Default, Deserialize)]
struct BotApplication {
    icon: Option<String>,
    description: Option<String>,
    name: Option<String>,
}

pub fn user_from_interaction_option(
    option: &ApplicationCommandInteractionDataOption,
) -> anyhow::Result<&User, !> {
    if let Some(value) = option.resolved.as_ref() {
        if let ApplicationCommandInteractionDataOptionValue::User(user, _) = value {
            Ok(user)
        } else {
            unreachable!("This interaction should always have a user")
        }
    } else {
        unreachable!("This interaction should always have a value")
    }
}

pub async fn send_user_info_command_response(
    ctx: &Context,
    interaction: &ApplicationCommandInteraction,
    user: &User,
) -> anyhow::Result<()> {
    let mut application: Option<BotApplication> = None;
    let mut member: Option<Member> = None;

    if let Ok(Some(app)) = bot_application(user).await {
        application = Some(app);
    }

    if let Some(guild_id) = interaction.guild_id {
        if let Ok(guild_member) = ctx.http.get_member(guild_id.0, &user.id.0).await {
            member = Some(guild_member);
        }
    }

    interaction
        .create_followup_message(&ctx.http, |f| {
            f.content(format!("```json\n{:#?}\n```", user))
                .create_embed(|e| create_user_embed(e, user, member, application))
                .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
        })
        .await?;

    Ok(())
}

async fn bot_application(user: &User) -> anyhow::Result<Option<BotApplication>> {
    if user.bot {
        let app_string = discord_get_pathway(&format!("/applications/{}/rpc", user.id.0)).await?;

        Ok(Some(serde_json::from_str::<BotApplication>(&app_string)?))
    } else {
        Ok(None)
    }
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

fn create_user_embed<'a>(
    embed: &'a mut CreateEmbed,
    user: &User,
    member: Option<Member>,
    application: Option<BotApplication>,
) -> &'a mut CreateEmbed {
    if let Some(application) = &application {
        embed.footer(|f| {
            if let Some(icon) = &application.icon {
                f.icon_url(avatar_url(&user.id.0.to_string(), icon, 1024));
            } else {
                f.icon_url(user.face());
            }

            if let Some(name) = &application.name {
                f.text(format!("Brought to you by {}!", name));
            } else {
                f.text(format!("Brought to you by {}!", user.name));
            }

            f
        });
    }

    if user
        .public_flags
        .unwrap()
        .contains(UserPublicFlags::VERIFIED_BOT)
    {
        embed.title(format!("{} <:verified:952780472570740746>", user.tag()));
    } else {
        embed.title(user.tag());
    }

    if let Some(member) = member {
        if let Some(nick) = member.nick {
            if user
                .public_flags
                .unwrap()
                .contains(UserPublicFlags::VERIFIED_BOT)
            {
                embed.title(format!(
                    "{} aka {} <:verified:952780472570740746>",
                    user.tag(),
                    nick
                ));
            }
        }

        embed
            .description(
                application
                    .unwrap_or_default()
                    .description
                    .unwrap_or_default(),
            )
            .url(user.face())
            .thumbnail(user.face())
            .field("Id:", &user.id.0, false)
            .field("Hypesquad House:", user_house(user), false)
            .field(
                "Joined:",
                format!("<t:{}:f>", member.joined_at.unwrap().timestamp()),
                false,
            )
            .field(
                "Created:",
                format!("<t:{}:f>", user.created_at().timestamp()),
                false,
            );
    } else {
        embed
            .url(user.face())
            .thumbnail(user.face())
            .field("Id:", user.id.0, false)
            .field("Hypesquad House:", user_house(user), false)
            .field(
                "Created:",
                format!("<t:{}:f>", user.created_at().timestamp()),
                false,
            );
    }

    embed
}
