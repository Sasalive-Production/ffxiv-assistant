use crate::{Context, Exception};
use ::serenity::all::CreateEmbedAuthor;
use poise::serenity_prelude as serenity;
use reqwest::Response;
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
pub async fn maintenance(ctx: Context<'_>) -> Result<(), Exception> {
    let url = "https://lodestonenews.com/news/topics?locale=jp";

    let client = reqwest::Client::new();
    let res: Response = client.get(url).send().await?;

    if res.status() == 200 {
        let news: Vec<News> = serde_json::from_str(&res.text().await?).unwrap();
        let embed = serenity::CreateEmbed::new()
            .author(CreateEmbedAuthor::new("FF14 Lodestone News").url(url))
            .fields({ news.iter().map(|n| (n.title.clone(), format!("[{}]({})", n.description, n.url), true)) });

        let msg = poise::CreateReply::default().embed(embed);

        ctx.send(msg).await?;
    } else {
        let embed = serenity::CreateEmbed::new()
            .author(CreateEmbedAuthor::new("FF14 Lodestone News").url(url))
            .description(format!("エラーが発生しました。ステータスコード: {}", res.status()));

        let msg = poise::CreateReply::default().embed(embed.clone());
        ctx.send(msg).await?;
    }

    Ok(())
}
