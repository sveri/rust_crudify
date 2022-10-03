#![warn(clippy::unwrap_used)]

use indexmap::IndexMap;
use serde_json::Value;

mod errors;
mod file_creator;
mod json_converter;
mod main_file_creator;

#[derive(Debug)]
pub struct InternalModel {
    pub name: String,
    pub properties: Option<IndexMap<String, String>>,
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
    fn lib_generate_one_object_with_two_properties() {
        let one_object_with_two_properties =
            json!({"Order": {"type": "object", "properties": {"id": {"type": "integer", "format": "int64"}, "name": {"type": "string"}}}});
        assert!(generate("user_id", &one_object_with_two_properties).is_ok())
    }
}
