use std::str::FromStr;

use crate::{utils::market::{Item, WishList}, Context, Exception};
use ::serenity::{all::CreateEmbed, futures};
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use serenity::all::{ButtonStyle, CreateActionRow, CreateButton, CreateEmbedAuthor, CreateInteractionResponseMessage};

#[derive(Serialize, Deserialize)]
struct World {
    id: u16,
    name: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(non_snake_case)]
struct MarketBoardResponse {
    lastUploadTime: u64,
    listings: Vec<MarketBoardItemResponse>,
    currentAveragePrice: f32,
    regularSaleVelocity: f32,
    minPrice: u32,
    maxPrice: u32,
    unitsForSale: u32,
    unitsSold: u32,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct MarketBoardItemResponse {
    lastReviewTime: u64,
    pricePerUnit: u32,
    quantity: u32,
    stainID: u32,
    worldName: Option<String>,
    worldID: Option<u16>,
    creatorName: Option<String>,
    creatorID: Option<u16>,
    hq: bool,
    isCrafted: bool,
    listingID: Option<String>,
    materia: Option<Vec<Materia>>,
    onMannequin: bool,
    retainerCity: i8,
    retainerID: Option<String>,
    retainerName: Option<String>,
    sellerID: Option<String>,
    total: u32,
    tax: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(non_snake_case)]
struct Materia {
    slotID: i32,
    materiaID: i32,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct ItemAPIResponse {
    Icon: String,
    IconHD: String,
}

#[derive(strum::Display, strum::EnumString)]
enum DataCenter {
    Aether,
    Primal,
    Crystal,
    Dynamis,
    Elemental,
    Gaia,
    Mana,
    Meteor,
    Chaos,
    Light,
    Shadow,
    Materia,
    猫小胖,
    莫古力,
    豆豆柴,
    陆行鸟,
}

#[derive(strum::Display, strum::EnumString)]
enum DataCenterLocation {
    Japan,
    #[strum(serialize = "North-America")]
    NorthAmerica,
    Europe,
    Oceania,
    China,
}

impl DataCenter {
    pub fn location(&self) -> DataCenterLocation {
        match self {
            DataCenter::Aether | DataCenter::Primal | DataCenter::Crystal => DataCenterLocation::NorthAmerica,
            DataCenter::Dynamis | DataCenter::Elemental | DataCenter::Gaia | DataCenter::Mana | DataCenter::Meteor => DataCenterLocation::Japan,
            DataCenter::Chaos | DataCenter::Light | DataCenter::Shadow => DataCenterLocation::Europe,
            DataCenter::Materia => DataCenterLocation::Oceania,
            DataCenter::猫小胖 | DataCenter::莫古力 | DataCenter::豆豆柴 | DataCenter::陆行鸟 => DataCenterLocation::China,
        }
    }
}

async fn autocomplete_item_name_ja<'a>(ctx: Context<'_>, partial_name: &'a str) -> impl Stream<Item = String> + 'a {
    let sql = sqlx::query_as::<_, Item>(r#"SELECT * FROM items;"#).bind(partial_name).fetch_all(&ctx.data().db).await.unwrap();

    futures::stream::iter(sql).map(move |i| i.ja.to_string().clone()).filter(move |i| futures::future::ready(i.contains(partial_name))).take(25)
}

async fn autocomplete_server_name<'a>(ctx: Context<'_>, partial_name: &'a str) -> impl Stream<Item = String> + 'a {
    futures::stream::iter(ctx.data().servers.keys().cloned().collect::<Vec<String>>())
        .filter(move |i| futures::future::ready(i.contains(partial_name) && !i.contains("Korea")))
        .take(25)
}

#[poise::command(slash_command)]
pub async fn market(
    ctx: Context<'_>,
    #[description = "The name of item to search"]
    #[autocomplete = "autocomplete_item_name_ja"]
    name: String,
    #[description = "The region to search"]
    #[autocomplete = "autocomplete_server_name"]
    region: String,
) -> Result<(), Exception> {
    let candidated_item = sqlx::query_as::<_, Item>(r#"SELECT * FROM items WHERE ja = $1 OR id = $1"#).bind(name).fetch_one(&ctx.data().db).await;

    let location = DataCenter::from_str(&region).unwrap().location().to_string();

    match candidated_item {
        Err(_e) => {
            let reply = {
                let embed = CreateEmbed::default()
                    .author(CreateEmbedAuthor::new("エラー"))
                    .description("アイテムが見つかりませんでした。")
                    .color(ctx.data().color_error);
                poise::CreateReply::default().embed(embed)
            };
            ctx.send(reply).await?;
            return Ok(());
        }
        Ok(item) => {
            ctx.defer_or_broadcast().await?;
            let client = reqwest::Client::new();
            let base_url = format!("https://universalis.app/api/v2/{}/{}", location, item.id);
            let query_fields = [
                "lastUploadTime",
                "listings",
                "unitsForSale",
                "unitsSold",
                "currentAveragePrice",
                "regularSaleVelocity",
                "minPrice",
                "maxPrice",
            ];

            let query = [("listings", "8"), ("fields", &query_fields.join(","))];
            let response_status = client.get(base_url).query(&query).send().await.unwrap();
            if !(response_status.status().is_success()) {
                let reply = {
                    let embed = CreateEmbed::default()
                        .author(CreateEmbedAuthor::new("エラー"))
                        .description(format!("マーケット情報の取得に失敗しました。ステータスコード: {}", response_status.status()))
                        .color(ctx.data().color_error);
                    poise::CreateReply::default().embed(embed)
                };
                ctx.send(reply).await?;
                return Ok(());
            }
            let response: MarketBoardResponse = response_status.json().await.unwrap();
            // identify interaction by context id
            let context_id = ctx.id();
            let add_wishlist_id = format!("{}-add-wishlist", context_id);

            let item_info_url = format!("https://xivapi.com/Item/{}", item.id);
            let item_info_query = [("snake_case", "1"), ("columns", "Icon,IconHD")];
            let item_info: ItemAPIResponse = client.get(item_info_url).query(&item_info_query).send().await.unwrap().json().await.unwrap();

            let reply = {
                let embed = CreateEmbed::default()
                    .author(CreateEmbedAuthor::new("Market Information"))
                    .description(format!("**{}**のマーケット情報を表示しています。", item.ja,))
                    .color(ctx.data().color_info)
                    .thumbnail(item_info.IconHD);

                let mut fields: Vec<(String, String, bool)> = vec![];
                let worlds = ctx.data().servers.get(&region).unwrap();

                for sale in response.listings {
                    if let Some(world_name) = &sale.worldName {
                        if worlds.contains(&world_name) {
                            continue;
                        } else {
                            fields.push((
                                world_name.to_string(),
                                format!("**{}Gil**({}Gil/per * {})", sale.total, sale.pricePerUnit, sale.total / sale.pricePerUnit),
                                false,
                            ))
                        }
                    }
                }

                let components = CreateActionRow::Buttons(vec![CreateButton::new(&add_wishlist_id)
                    .emoji('📋')
                    .label("欲しいものリストに追加")
                    .style(ButtonStyle::Primary)]);

                poise::CreateReply::default().embed(embed.fields(fields)).components(vec![components])
            };

            ctx.send(reply).await?;

            while let Some(press) = serenity::collector::ComponentInteractionCollector::new(ctx)
                .filter(move |press| press.data.custom_id.starts_with(&context_id.to_string()))
                .timeout(std::time::Duration::from_secs(60 * 5))
                .await
            {
                if press.data.custom_id == add_wishlist_id {
                    let already_added = sqlx::query_as::<_, WishList>(r#"SELECT * FROM wishlist WHERE item_id = $1 AND user_id = $2;"#)
                        .bind(item.id)
                        .bind(i64::from(ctx.author().id))
                        .fetch_one(&ctx.data().db)
                        .await;

                    // Errが返ってきた時に追加するの、気持ち悪いね。
                    match already_added {
                        Ok(_) => {
                            let reply = {
                                let embed = CreateEmbed::default()
                                    .author(CreateEmbedAuthor::new("Wishlist"))
                                    .description(format!("**{}**はすでに欲しいものリストに追加されています。", item.ja))
                                    .color(ctx.data().color_info);
                                poise::CreateReply::default().embed(embed).ephemeral(true)
                            };
                            ctx.send(reply).await?;
                        }

                        Err(_) => {
                            sqlx::query(r#"INSERT INTO wishlist (item_id, user_id) VALUES ($1, $2, $3)"#)
                                .bind(item.id)
                                .bind(i64::from(ctx.author().id))
                                .execute(&ctx.data().db)
                                .await
                                .unwrap();

                            let reply = {
                                let embed = CreateEmbed::default()
                                    .author(CreateEmbedAuthor::new("Wishlist"))
                                    .description(format!("**{}**を欲しいものリストに追加しました。", item.ja))
                                    .color(ctx.data().color_success);
                                CreateInteractionResponseMessage::new().embed(embed).ephemeral(true)
                            };
                            press.create_response(ctx.http(), serenity::all::CreateInteractionResponse::Message(reply)).await?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
