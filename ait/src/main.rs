use crate::hyperview::api_constants::ASSET_TYPES;

mod hyperview;

fn main() {
    ASSET_TYPES.iter().for_each(|t| println!("{t}"));
}
