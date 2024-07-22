extern crate env_logger as logger;

use std::collections::HashSet;

use dotenvy::dotenv;
use env_logger;
use poise::serenity_prelude as serenity;

mod commands {
    pub mod about;
    pub mod news;
    pub mod market;
    // pub mod weather;
}

mod utils {
    pub mod paginate;
    pub mod market;
}

type Exception = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Exception>;

pub struct Data {
    // loaded_data: Vec<market::Item>,
    db: sqlx::SqlitePool,
}

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "notice");
    env_logger::init();
    dotenv().unwrap();

    let mut owners = HashSet::new();
    owners.insert(serenity::UserId::new(631786733511376916));

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::about::about(),
                commands::news::news(),
                // commands::weather::weather()
                commands::market::market(),
                ],
            on_error: |error| {
                    Box::pin(async move {
                        println!("what the hell");
                        match error {
                            poise::FrameworkError::ArgumentParse { error, .. } => {
                                if let Some(error) = error.downcast_ref::<serenity::RoleParseError>() {
                                    println!("Found a RoleParseError: {:?}", error);
                                } else {
                                    println!("Not a RoleParseError :(");
                                }
                            }
                            other => poise::builtins::on_error(other).await.unwrap(),
                        }
                    })
                },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                // poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                // poise::builtins::register_in_guild(ctx, &framework.options().commands, serenity::GuildId::new(1140679501852966994)).await?; //一瞬間が近いよ
                poise::builtins::register_in_guild(ctx, &framework.options().commands, serenity::GuildId::new(1169613731269984376)).await?; // sasagusa folder

                let conn = sqlx::sqlite::SqlitePool::connect("sqlite://sqlite.db").await.unwrap();
                Ok(Data {
                    // loaded_data: load_items("src/resources/items.json"),
                    db: conn,
                })
            })
        })
        .build();

    let token = std::env::var("token").unwrap();
    let intents = serenity::GatewayIntents::non_privileged();

    let client = serenity::ClientBuilder::new(token, intents).framework(framework).await;

    client.unwrap().start().await.unwrap();
}
