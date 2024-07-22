use crate::{utils::market::Item, Context, Exception};
use serde::{Deserialize, Serialize};
use serenity::all::CreateEmbedAuthor;
use ::serenity::{all::CreateEmbed, futures};
use futures::{Stream, StreamExt};

#[derive(Serialize, Deserialize, Debug)]
struct DataCenter {
    name: String,
    region: String,
    worlds: Vec<u16>,
}

#[derive(Serialize, Deserialize, Debug)]
struct World {
    id: u16,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[allow(non_snake_case)]
struct MarketBoardItemResponse{
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
    on_mannequin: bool,
    retainer_city: i8,
    retainerID: Option<String>,
    retainerName: Option<String>,
    sellerID: Option<String>,
    total: u32,
    tax: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[allow(non_snake_case)]
struct Materia {
    slotID: i32,
    materiaID: i32,
}

async fn autocomplete_item_name_ja<'a>(
    ctx: Context<'_>,
    partial_name: &'a str,
) -> impl Stream<Item = String> +'a {
    let mut sql = sqlx::query_as::<_, Item>(r#"SELECT * FROM items WHERE LIKE ja = $1 OR id = $1"#)
        .bind(partial_name)
        .fetch_all(&ctx.data().db)
        .await
        .unwrap();
    sql.truncate(20);

    let candidates = sql.iter().map(|i| i.ja.to_string().clone()).collect::<Vec<String>>();

    futures::stream::iter(candidates)
        .filter(move |i| futures::future::ready(i.contains(partial_name)))
        .map(|i| i.to_string())
}

#[poise::command(slash_command)]
pub async fn market(
    ctx: Context<'_>,
    #[description = "The name of item to search"]
    #[autocomplete = "autocomplete_item_name_ja"]
    name: String,
    #[description = "The region to search"]
    #[choices("Japan", "Europe", "North-America", "Oceania", "China")]
    region: &str,
) -> Result<(), Exception> {
    /* println!("Deferred!");
    let items = ctx.data().loaded_data.clone();
    println!("Loaded!");
    let candidated_item = items.iter()
        .find(|i| i.ja == name || i.id.to_string() == name);*/

    let candidated_item = sqlx::query_as::<_, Item>(r#"SELECT * FROM items WHERE LIKE ja = $1 OR id = $1"#)
        .bind(name)
        .fetch_one(&ctx.data().db)
        .await;

    match candidated_item {
        Err(_e) => {
            let reply = {
                let embed = CreateEmbed::default()
                .author(CreateEmbedAuthor::new("エラー"))
                .description("アイテムが見つかりませんでした。")
                .color(0xDC143C);
                poise::CreateReply::default().embed(embed)
            };
            ctx.send(reply).await?;
            return Ok(())
        }
        Ok(item) => {
            ctx.defer_or_broadcast().await?;
            let client = reqwest::Client::new();
            let base_url = format!("https://universalis.app/api/v2/{}/{}", region, item.id);
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
                        .color(0xDC143C);
                    poise::CreateReply::default().embed(embed)
                };
                ctx.send(reply).await?;
                return Ok(())
            }
            let response: MarketBoardResponse = response_status.json().await.unwrap();

            let reply = {
                let embed = CreateEmbed::default()
                    .author(CreateEmbedAuthor::new("Market Information"))
                    .description(
                        format!(
                            "**{}**のマーケット情報を表示しています。",
                            item.ja,
                    ))
                    .color(0x0073d1);

                let mut fields: Vec<(String, String, bool)> = vec![];

                for sale in response.listings {
                    fields.push(
                        (
                            sale.worldName.unwrap(),
                            format!("**{}Gil**({})", sale.total, sale.pricePerUnit),
                            false
                        )
                    );
                };

                poise::CreateReply::default().embed(embed.fields(fields))
            };

            ctx.send(reply).await?;
            Ok(())
        }
    }
}
