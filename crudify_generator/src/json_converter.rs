use indexmap::IndexMap;
use phf::phf_map;
use serde::Deserialize;
use serde_json::{Map, Value};

use crate::{errors::JsonConverterError, InternalModel, InternalModels};

#[derive(Deserialize, Debug)]
struct OA3Type {
    #[serde(rename = "type")]
    kind: String,
    format: Option<String>,
}

impl OA3Type {
    fn get_format_or_type(&self) -> String {
        match &self.format {
            Some(format) => format.to_string(),
            None => self.kind.to_string(),
        }
    }
}

pub fn convert_to_internal_model(j: &Value) -> Result<InternalModels, JsonConverterError> {
    let mut internal_models = Vec::new();
    for (key, value) in as_object(j)? {
        if value.is_object() {
            let properties = parse_properties(value)?;

            internal_models.push(InternalModel {
                name: key.to_string(),
                properties: Some(properties),
            })
        }
    }

    Ok(internal_models)
}

fn parse_properties(value: &Value) -> Result<IndexMap<String, String>, JsonConverterError> {
    let mut property_map: IndexMap<String, String> = IndexMap::new();
    let o = as_object(value)?;
    if let Some(properties) = o.get("properties") {
        for (property_key, property_value) in as_object_with_context(properties, value)? {
            if property_value.is_object() {
                let data_type = parse_data_type(property_value)?;
                property_map.insert(property_key.to_string(), data_type.to_string());
            } else {
                return Err(JsonConverterError::AsObjectError { value: property_value });
            }
        }
    }

    Ok(property_map)
}

fn parse_data_type(property_value: &Value) -> Result<String, JsonConverterError> {
    let parsed_object: Result<OA3Type, serde_json::Error> = serde_json::from_value(property_value.to_owned());
    match parsed_object {
        Err(_) => Err(JsonConverterError::AsObjectError { value: property_value }),
        Ok(property_object) => {
            let oa3_type = property_object.get_format_or_type();
            if let Some(data_type) = DATATYPE_TO_RUST_DATATYPE.get(&oa3_type) {
                Ok(data_type.to_string())
            } else {
                Ok("String".to_string())
            }
        }
    }
}

fn as_object(value: &Value) -> Result<&Map<String, Value>, JsonConverterError> {
    value.as_object().ok_or(JsonConverterError::AsObjectError { value })
}

fn as_object_with_context<'a>(value: &'a Value, ctx_value: &'a Value) -> Result<&Map<String, Value>, JsonConverterError> {
    value.as_object().ok_or(JsonConverterError::AsObjectError { value: ctx_value })
}

// https://github.com/OAI/OpenAPI-Specification/blob/main/versions/3.0.3.md#schema-object
static DATATYPE_TO_RUST_DATATYPE: phf::Map<&'static str, &'static str> = phf_map! {
    "integer" => "i64",
    "string" => "String",
    "boolean" => "bool",
    "number" => "f64",
    "int64" => "i64",
    "int32" => "i32",
    "date" => "chrono::Date",
    "date-time" => "chrono::DateTime",
    "password" => "String",
    "byte" => "u8",
    "float" => "f32",
    "double" => "f64"
};

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn with_name_property_in_a_second_object() {
        let order_with_id =
            json!({"Order": {"type": "object", "properties": {"id": {"type": "integer", "format": "int64"}, "name": {"type": "string"}}},
            "OrderTwo": {"type": "object", "properties": {"id": {"type": "integer", "format": "int64"}, "isFoo": {"type": "boolean"}}}});
        let models = convert_to_internal_model(&order_with_id).unwrap();
        assert_eq!("Order", models.get(0).unwrap().name);
        assert_eq!(
            "i64".to_string(),
            models.get(0).unwrap().properties.as_ref().unwrap().get("id").unwrap().to_string()
        );
        assert_eq!(
            "String".to_string(),
            models.get(0).unwrap().properties.as_ref().unwrap().get("name").unwrap().to_string()
        );
        assert_eq!("OrderTwo", models.get(1).unwrap().name);
        assert_eq!(
            "i64".to_string(),
            models.get(1).unwrap().properties.as_ref().unwrap().get("id").unwrap().to_string()
        );
        assert_eq!(
            "bool".to_string(),
            models.get(1).unwrap().properties.as_ref().unwrap().get("isFoo").unwrap().to_string()
        );
    }
    #[test]
    fn with_name_property() {
        let order_with_id =
            json!({"Order": {"type": "object", "properties": {"id": {"type": "integer", "format": "int64"}, "name": {"type": "string"}}}});
        let models = convert_to_internal_model(&order_with_id).unwrap();
        assert_eq!("Order", models.get(0).unwrap().name);
        assert_eq!(
            "i64".to_string(),
            models.get(0).unwrap().properties.as_ref().unwrap().get("id").unwrap().to_string()
        );
        assert_eq!(
            "String".to_string(),
            models.get(0).unwrap().properties.as_ref().unwrap().get("name").unwrap().to_string()
        );
    }

    #[test]
    fn with_wrong_id_property_must_err() {
        let order_with_id = json!({"Order": {"type": "object", "properties": {"id": "foobar"}}});
        let models = convert_to_internal_model(&order_with_id);
        assert!(models.is_err());
        assert_eq!(
            JsonConverterError::AsObjectError { value: &json!("foobar") }.to_string(),
            models.unwrap_err().to_string()
        );
    }

    #[test]
    fn non_object_value_must_err() {
        let order_with_id = json!({"Order": {"type": "object", "properties": []}});
        let models = convert_to_internal_model(&order_with_id);
        assert!(models.is_err());
        assert_eq!(
            JsonConverterError::AsObjectError {
                value: &json!({"type": "object", "properties": []})
            }
            .to_string(),
            models.unwrap_err().to_string()
        );
    }
    #[test]
    fn with_id_property_without_type_object() {
        let order_with_id = json!({"Order": {"type": "object", "properties": {"id": {}}}});
        let models = convert_to_internal_model(&order_with_id);
        assert!(models.is_err());
        assert_eq!(
            JsonConverterError::AsObjectError { value: &json!({}) }.to_string(),
            models.unwrap_err().to_string()
        );
    }

    #[test]
    fn with_id_property_type_and_format() {
        let order_with_id = json!({"Order": {"type": "object", "properties": {"id": {"type": "integer", "format": "int64"}}}});
        let models = convert_to_internal_model(&order_with_id).unwrap();
        assert_eq!("Order", models.get(0).unwrap().name);
        assert_eq!(
            "i64".to_string(),
            models.get(0).unwrap().properties.as_ref().unwrap().get("id").unwrap().to_string()
        );
    }

    #[test]
    fn with_with_extra_properties() {
        let order_with_id =
            json!({"Order": {"type": "object", "properties": {"id": {"type": "integer", "format": "int64", "example": "3"}}}});
        let models = convert_to_internal_model(&order_with_id).unwrap();
        assert_eq!("Order", models.get(0).unwrap().name);
        assert_eq!(
            "i64".to_string(),
            models.get(0).unwrap().properties.as_ref().unwrap().get("id").unwrap().to_string()
        );
    }

    #[test]
    fn with_id_property_and_type() {
        let order_with_id = json!({"Order": {"type": "object", "properties": {"id": {"type": "integer"}}}});
        let models = convert_to_internal_model(&order_with_id).unwrap();
        assert_eq!("Order", models.get(0).unwrap().name);
        assert_eq!(
            "i64".to_string(),
            models.get(0).unwrap().properties.as_ref().unwrap().get("id").unwrap().to_string()
        );
    }

    #[test]
    fn without_properties() {
        let two_order_objects = json!({"Order": {}, "OrderTwo": {}});
        let models = convert_to_internal_model(&two_order_objects).unwrap();
        assert_eq!("Order", models.get(0).unwrap().name);
        assert_eq!("OrderTwo", models.get(1).unwrap().name);
    }
}
