use std::error::Error;

use tarkov_discord_bot::requests::{find_closest_object, retrieve_item_info, retrieve_items};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let items = retrieve_items().await?;

    let search = "mayo";
    let item = find_closest_object(search, &items).unwrap();
    let item_info = retrieve_item_info(&item.id).await?;

    println!("{:#?}", item_info);
    Ok(())
}
