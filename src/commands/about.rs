use crate::{Context, Exception};
use ::serenity::all::CreateEmbedAuthor;
use poise::serenity_prelude as serenity;

#[poise::command(slash_command)]
pub async fn about(ctx: Context<'_>) -> Result<(), Exception> {
    let msg = {
        let embed = serenity::CreateEmbed::default().author(CreateEmbedAuthor::new("About")).description("FF14の各種情報を提供します。").color(0x0073d1);

        poise::CreateReply::default().embed(embed)
    };

    ctx.send(msg).await?;

    Ok(())
}
