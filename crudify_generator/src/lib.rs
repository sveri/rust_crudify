#![warn(clippy::unwrap_used)]

use indexmap::IndexMap;
use serde_json::{Value};

mod file_creator;
mod main_file_creator;
mod json_converter;
mod errors;

#[derive(Debug)]
pub struct InternalModel {
    pub name: String,
    pub properties: Option<IndexMap<String, String>>
}

pub type InternalModels = Vec<InternalModel>;

pub fn generate<'a>(user_id: &'a str, input_objects: &'a Value) -> Result<(), Box<dyn std::error::Error + 'a>> {
    let models = json_converter::convert_to_internal_model(input_objects)?;
    file_creator::write_all(user_id, &models)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn it_works() {
        let order_with_id = json!({"Order": {}, "OrderTwo": {}});
        assert!(generate("user_id", &order_with_id).is_ok())
    }
}
