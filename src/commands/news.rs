use crate::{Context, Exception};
use ::serenity::all::CreateEmbedAuthor;
use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct News {
    id: String,
    url: String,
    title: String,
    image: String,
    description: String,
}

#[poise::command(slash_command)]
pub async fn news(ctx: Context<'_>) -> Result<(), Exception> {
    let req_url = "https://lodestonenews.com/news/topics?locale=jp";
    let url = "https://jp.finalfantasyxiv.com/lodestone/";
    let client = reqwest::Client::new();
    let res = client.get(req_url).send().await;
    match res {
        Ok(res) => {
            if res.status() == 200 {
                let res_text = res.text().await?;
                let news: Vec<News> = serde_json::from_str(&res_text).unwrap();
                let msg = poise::CreateReply::default().embed(
                    serenity::CreateEmbed::new().author(CreateEmbedAuthor::new("FF14 Lodestone News").url("")).fields(news.iter().take(10).map(|n| {
                        (
                            n.title.clone(),
                            format!("[{}]({})", format!("{}...", n.description.chars().take(20).collect::<String>()), n.url),
                            false,
                        )
                    })),
                );

                ctx.send(msg).await?;
            } else {
                let msg = poise::CreateReply::default().embed(
                    serenity::CreateEmbed::new()
                        .author(CreateEmbedAuthor::new("FF14 Lodestone News").url(url))
                        .description(format!("エラーが発生しました。ステータスコード: {}", res.status())),
                );
                ctx.send(msg).await?;
            }
        }
        Err(e) => {
            let msg = poise::CreateReply::default().embed(
                serenity::CreateEmbed::new()
                    .author(CreateEmbedAuthor::new("FF14 Lodestone News").url(url))
                    .description(format!("エラーが発生しました。\n```\n{}\n```", e)),
            );
            ctx.send(msg).await?;
        }
    };
    Ok(())
}
