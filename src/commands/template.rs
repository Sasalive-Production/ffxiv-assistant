use crate::{Context, Exception};
use poise::serenity_prelude as serenity;

#[poise::command(slash_command)]
pub async fn command_name(ctx: Context<'_>) -> Result<(), Exception> {
    // write your code here
    Ok(())
}

