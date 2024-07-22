use poise::serenity_prelude as serenity;
use poise::Context;
use ::serenity::all::CreateEmbed;

#[allow(dead_code)]
pub async fn paginate<U, E>(ctx: Context<'_, U, E>, pages: &[CreateEmbed]) -> Result<(), serenity::Error> {
    let ctx_id = ctx.id();
    let first_button_id = format!("{}first", ctx_id);
    let prev_button_id = format!("{}prev", ctx_id);
    let stop_button_id = format!("{}stop", ctx_id);
    let next_button_id = format!("{}next", ctx_id);
    let last_button_id = format!("{}last", ctx_id);

    let reply = {
        let components = serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new(&first_button_id).emoji('⏮'),
            serenity::CreateButton::new(&prev_button_id).emoji('◀'),
            serenity::CreateButton::new(&stop_button_id).emoji('⏹'),
            serenity::CreateButton::new(&next_button_id).emoji('▶'),
            serenity::CreateButton::new(&last_button_id).emoji('⏭'),
        ]);

        poise::CreateReply::default().embed(pages[0].clone()).components(vec![components])
    };

    ctx.send(reply).await?;

    let mut current_page = 0;
    while let Some(press) = serenity::collector::ComponentInteractionCollector::new(ctx)
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        .timeout(std::time::Duration::from_secs(60 * 5)) // wait 5min
        .await
    {
        if press.data.custom_id == first_button_id {
            current_page = 0;
        } else if press.data.custom_id == prev_button_id {
            current_page -= 1;
            if current_page < 0 {
                current_page = pages.len() - 1;
            }
        } else if press.data.custom_id == stop_button_id {
            break;
        } else if press.data.custom_id == next_button_id {
            current_page += 1;
            if current_page >= pages.len() {
                current_page = 0;
            }
        } else if press.data.custom_id == last_button_id {
            current_page = pages.len() - 1;
        } else {
            continue;
        }

        press.create_response(
                ctx.serenity_context(),
                serenity::CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::new().embed(pages[current_page].clone()),
                ),
            )
            .await?;
    }
    Ok(())
}
