

use std::sync::Arc;

use serenity::{prelude::Context, model::prelude::ChannelId};

pub async fn run(ctx: Arc<Context>) {
    let _noe = ChannelId(772092284153757719).send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title("Asyncly doing stuff")
            .field("name", "value", false)
        })
    }).await;
    if let Err(e) = _noe { // TODO: log
        println!("Error: {:?}", e);
    }
}