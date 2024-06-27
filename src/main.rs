extern crate env_logger as logger;

use std::collections::HashSet;

use dotenvy::dotenv;
use env_logger;
use poise::serenity_prelude as serenity;

mod commands {
    pub mod about;
    pub mod news;
}

mod utils;

type Exception = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Exception>;

pub struct Data {}

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenv().unwrap();

    let mut owners = HashSet::new();
    owners.insert(serenity::UserId::new(631786733511376916));

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![commands::about::about()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let token = std::env::var("token").unwrap();
    let intents = serenity::GatewayIntents::non_privileged();

    let client = serenity::ClientBuilder::new(token, intents).framework(framework).await;

    client.unwrap().start().await.unwrap();
}
