use crate::{utils::{self, weather::calculate_weather}, Context, Exception};
use poise::serenity_prelude as serenity;
use crate::utils::weather;

#[poise::command(slash_command)]
pub async fn weather(ctx: Context<'_>) -> Result<(), Exception> {
    // write your code here
    Ok(())
}
