use std::borrow::Borrow;
use crate::commands::text_command_failure;
use serenity::builder::CreateEmbed;
use serenity::framework::standard::{macros::*, CommandResult};
use serenity::model::prelude::{Message, User};
use serenity::prelude::Context;
use std::collections::HashMap;
use std::lazy::SyncLazy;

type Callback = for<'a> fn(&'a mut CreateEmbed, &User) -> &'a mut CreateEmbed;

pub static SPECIAL_USERS: SyncLazy<HashMap<u64, Callback>> = SyncLazy::new(|| {
    HashMap::from([
        (688130941582966946, kenna as Callback),
        (348275601934778368, keegan as Callback),
        (838926666822582292, sofia as Callback),
        (326877252811751426, ale as Callback),
        (485605533227679756, moosh as Callback),
        (688187112335474753, emmy as Callback),
    ])
});

#[command]
async fn user_composition(ctx: &Context, msg: &Message) -> CommandResult {
    let options: Vec<&str> = msg.content.split_whitespace().collect();

    if let Ok(id) = options[2].parse::<u64>() {
        if let Ok(user) = ctx.http.get_user(id).await {
            if SPECIAL_USERS.borrow().contains_key(&user.id.0) {
                msg.channel_id
                    .send_message(&ctx.http, |m| {
                        m.embed(|e| SPECIAL_USERS.borrow()[&user.id.0](e, &user))
                    })
                    .await?;
            } else {
                msg.channel_id
                    .send_message(&ctx.http, |m| m.embed(|e| default_embed(e, &user)))
                    .await?;
            }
        } else {
            text_command_failure(ctx, msg, "A user with that id does not exist!").await?;
        }
    } else {
        text_command_failure(
            ctx,
            msg,
            "This command only takes user ids, please try again!",
        )
        .await?;
    }

    Ok(())
}

fn default_embed<'a>(embed: &'a mut CreateEmbed, user: &User) -> &'a mut CreateEmbed {
    embed.title(user.tag())
        .thumbnail(user.face())
        .description("We dont have any records on your composition, but you are certifiably cool! <:fingerguns:810343397311250455> ")
}

fn kenna<'a>(embed: &'a mut CreateEmbed, user: &User) -> &'a mut CreateEmbed {
    embed.title(user.tag()).thumbnail(user.face())
}

fn keegan<'a>(embed: &'a mut CreateEmbed, user: &User) -> &'a mut CreateEmbed {
    embed.title(user.tag()).thumbnail(user.face())
}

fn sofia<'a>(embed: &'a mut CreateEmbed, user: &User) -> &'a mut CreateEmbed {
    embed.title(user.tag()).thumbnail(user.face())
}

fn ale<'a>(embed: &'a mut CreateEmbed, user: &User) -> &'a mut CreateEmbed {
    embed.title(user.tag()).thumbnail(user.face())
}

fn moosh<'a>(embed: &'a mut CreateEmbed, user: &User) -> &'a mut CreateEmbed {
    embed.title(user.tag()).thumbnail(user.face())
}

fn emmy<'a>(embed: &'a mut CreateEmbed, user: &User) -> &'a mut CreateEmbed {
    embed.title(user.tag())
        .description("According to our fantastically meticulous records, the user's composition is as follows:")
        .thumbnail("https://cdn.discordapp.com/emojis/845122742378168360.png?v=1")
        .field("Memeness:", "20000000000%", false)
}
