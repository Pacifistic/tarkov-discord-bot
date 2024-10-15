use std::error::Error;

use graphql_client::{GraphQLQuery, Response};
use reqwest::Client;
use strsim::levenshtein;

const TARKOV_API_URL: &str = "https://api.tarkov.dev/graphql";

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.json",
    query_path = "graphql/item_list.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
struct ItemsList;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.json",
    query_path = "graphql/item_details.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
struct ItemDetails;

#[derive(Debug)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub short_name: String,
    pub normalized_name: String,
}

#[derive(Debug)]
pub struct ItemInfo {
    pub base_price: i64,
    pub low_price: i64,
    pub avg_price: i64,
    pub link: String,
    pub icon_link: String,
    pub name: String,
    pub vendor_price: i64,
    pub vendor: String,
}

pub async fn retrieve_items() -> Result<Vec<Item>, Box<dyn Error>> {
    let client = Client::new();
    let request = ItemsList::build_query(items_list::Variables {});
    let res = client.post(TARKOV_API_URL).json(&request).send().await?;

    let response_body: Response<items_list::ResponseData> = res.json().await?;
    let response_data = response_body
        .data
        .expect("There should be data in the response");
    let mut items: Vec<Item> = vec![];
    for item in response_data.items {
        let item_attributes = item.expect("There should be item attributes");
        let id = item_attributes.id;
        let name = item_attributes.name.expect("there should be a name");
        let short_name = item_attributes
            .short_name
            .expect("there should be a short name");
        let normalized_name = item_attributes
            .normalized_name
            .expect("there should be a normalized name");
        items.push(Item {
            id,
            name,
            short_name,
            normalized_name,
        });
    }
    Ok(items)
}

pub fn find_closest_object<'a>(search: &str, objects: &'a [Item]) -> Option<&'a Item> {
    objects
        .iter()
        .map(|obj| {
            let distance = [
                levenshtein(search, &obj.id),
                levenshtein(search, &obj.name),
                levenshtein(search, &obj.short_name),
                levenshtein(search, &obj.normalized_name),
            ]
            .iter()
            .cloned()
            .min() // Find the minimum distance
            .unwrap_or(usize::MAX); // Handle cases where all fields are empty

            (distance, obj) // Return a tuple of distance and object reference
        })
        .min_by_key(|&(distance, _)| distance) // Find the object with the smallest distance
        .map(|(_, obj)| obj) // Extract the object reference
}

pub async fn retrieve_item_info(id: &str) -> Result<ItemInfo, Box<dyn Error>> {
    let client = Client::new();
    let request = ItemDetails::build_query(item_details::Variables { id: id.to_string() });
    let res = client.post(TARKOV_API_URL).json(&request).send().await?;

    let response_body: Response<item_details::ResponseData> = res.json().await?;
    let response_data = response_body
        .data
        .expect("There should be data in the response");

    let item = response_data
        .item
        .expect("There should be an item in the response");

    let mut vendor = "".to_string();
    let mut vendor_price = 0;

    for sell_for in item
        .sell_for
        .expect("There should be at least one item in sell_for")
    {
        let price = sell_for.price_rub.expect("There should be a price");
        if sell_for.vendor.name != "Flea Market" && price > vendor_price {
            vendor_price = price;
            vendor = sell_for.vendor.name;
        }
    }

    Ok(ItemInfo {
        base_price: item.base_price,
        low_price: item.last_low_price.unwrap_or(0),
        avg_price: item.avg24h_price.unwrap_or(0),
        link: item.link.expect("There should be an item link"),
        icon_link: item.icon_link.expect("There should be an icon link"),
        name: item.name.expect("there should be a name"),
        vendor_price,
        vendor,
    })
}
