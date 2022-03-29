use crate::commands::{text_command_failure, text_command_message_options};
use serenity::framework::standard::macros::*;
use serenity::framework::standard::CommandResult;
use serenity::http::AttachmentType::Bytes;
use serenity::model::prelude::Message;
use serenity::prelude::Context;
use std::borrow::Cow;
use std::env;
use url_encoded_data::UrlEncodedData;

#[command]
async fn directions(ctx: &Context, msg: &Message) -> CommandResult {
    let typing = ctx.http.start_typing(msg.channel_id.0).unwrap();

    let options: String = text_command_message_options(msg).join(" ");

    if options.len() < 2 {
        text_command_failure(
            ctx,
            msg,
            "Must have at least a starting location and destination!",
        )
        .await?;
    } else {
        let directions: Vec<&str> = options.split(" to ").collect();

        let map_url = google_maps_url(directions[0], directions[1]);
        let static_map_url = static_map_url(directions[0], directions[1]);

        let response_bytes = reqwest::get(static_map_url).await?.bytes().await?.to_vec();

        let bytes = Bytes {
            data: Cow::from(response_bytes),
            filename: String::from("map.png"),
        };

        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.attachment("map.png")
                        .title(format!("{} to {}", directions[0], directions[1]))
                        .url(map_url)
                })
                .add_file(bytes)
            })
            .await?;
    }

    typing.stop();

    Ok(())
}

fn google_maps_url(origin: &str, destination: &str) -> String {
    let base_url = "https://www.google.com/maps/dir/?api=1";

    let url = UrlEncodedData::parse_str(base_url)
        .set_one("origin", origin)
        .set_one("destination", destination)
        .done();

    url.to_string_of_original_order()
}

fn static_map_url(origin: &str, destination: &str) -> String {
    let base_url = "https://maps.googleapis.com/maps/api/staticmap?size=512x512&maptype=roadmap\\";

    let token = env::var("GOOGLE_MAPS_TOKEN").unwrap();
    let locations = format!("{}|{}", origin, destination);

    let url = UrlEncodedData::parse_str(base_url)
        .set_one("key", &token)
        .set_one("markers", &locations)
        .done();

    url.to_string_of_original_order()
}
