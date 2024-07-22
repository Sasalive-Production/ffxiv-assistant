use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::FromRow)]
pub struct Item {
    pub id: u16,
    pub en: String,
    pub de: String,
    pub ja: String,
    pub fr: String,
}

#[allow(dead_code)]
pub fn load_items(path: &str) -> Vec<Item> {
    let file = std::fs::File::open(path).unwrap();
    let json: Vec<Item> = serde_json::from_reader(file).unwrap();

    return json
}