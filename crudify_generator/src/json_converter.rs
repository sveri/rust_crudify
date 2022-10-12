use std::{fmt::Display, u8};

use indexmap::IndexMap;
use phf::phf_map;
use serde::Deserialize;
use serde_json::{Map, Value};

use crate::{errors::JsonConverterError, errors::JsonConverterError::AsObjectError, InternalModel, InternalModels};

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

fn parse_properties(value: &Value) -> Result<IndexMap<String, RustDataType>, JsonConverterError> {
    let mut property_map: IndexMap<String, RustDataType> = IndexMap::new();
    let o = as_object(value)?;
    if let Some(properties) = o.get("properties") {
        for (property_key, property_value) in as_object_with_context(properties, value)? {
            if property_value.is_object() {
                let data_type = parse_data_type(property_value)?;
                property_map.insert(property_key.to_string(), data_type);
            } else {
                return Err(AsObjectError(property_value));
            }
        }
    }

    Ok(property_map)
}

fn parse_data_type(property_value: &Value) -> Result<RustDataType, JsonConverterError> {
    let parsed_object: Result<OA3Type, serde_json::Error> = serde_json::from_value(property_value.to_owned());
    match parsed_object {
        Err(_) => Err(AsObjectError(property_value)),
        Ok(property_object) => {
            let oa3_type = property_object.get_format_or_type();
            if let Some(data_type) = DATATYPE_TO_RUST_DATATYPE.get(&oa3_type) {
                Ok(*data_type)
            } else {
                Ok(RustDataType::String)
            }
        }
    }
}

fn as_object(value: &Value) -> Result<&Map<String, Value>, JsonConverterError> {
    value.as_object().ok_or(AsObjectError(value))
}

fn as_object_with_context<'a>(value: &'a Value, ctx_value: &'a Value) -> Result<&Map<String, Value>, JsonConverterError> {
    value.as_object().ok_or(AsObjectError(ctx_value))
}

#[derive(Copy, Clone, Debug)]
pub enum RustDataType {
    U8,
    I32,
    I64,
    String,
    Bool,
    F32,
    F64,
    Date,
    DateTime
}

impl Display for RustDataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RustDataType::U8 => write!(f, "u8"),
            RustDataType::I32 => write!(f, "i32"),
            RustDataType::I64 => write!(f, "i64"),
            RustDataType::String => write!(f, "String"),
            RustDataType::Bool => write!(f, "bool"),
            RustDataType::F32 => write!(f, "f32"),
            RustDataType::F64 => write!(f, "f64"),
            RustDataType::Date => write!(f, "chrono::Date"),
            RustDataType::DateTime => write!(f, "chrono::DateTime"),
        }
    }
}

// https://github.com/OAI/OpenAPI-Specification/blob/main/versions/3.0.3.md#schema-object
static DATATYPE_TO_RUST_DATATYPE: phf::Map<&'static str, RustDataType> = phf_map! {
    "integer" => RustDataType::I64,
    "string" => RustDataType::String,
    "boolean" => RustDataType::Bool,
    "number" => RustDataType::F64,
    "int64" => RustDataType::I64,
    "int32" => RustDataType::I32,
    "date" => RustDataType::Date,
    "date-time" => RustDataType::DateTime,
    "password" => RustDataType::String,
    "byte" => RustDataType::U8,
    "float" => RustDataType::F32,
    "double" => RustDataType::F64
};

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn with_name_property_in_a_second_object() {
        let order_with_id = json!({"Order": {"type": "object", "properties": {"id": {"type": "integer", "format": "int64"}, "name": {"type": "string"}}},
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
            models
                .get(1)
                .unwrap()
                .properties
                .as_ref()
                .unwrap()
                .get("isFoo")
                .unwrap()
                .to_string()
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
        assert_eq!(AsObjectError(&json!("foobar")).to_string(), models.unwrap_err().to_string());
    }

    #[test]
    fn non_object_value_must_err() {
        let order_with_id = json!({"Order": {"type": "object", "properties": []}});
        let models = convert_to_internal_model(&order_with_id);
        assert!(models.is_err());
        assert_eq!(
            AsObjectError(&json!({"type": "object", "properties": []})).to_string(),
            models.unwrap_err().to_string()
        );
    }
    #[test]
    fn with_id_property_without_type_object() {
        let order_with_id = json!({"Order": {"type": "object", "properties": {"id": {}}}});
        let models = convert_to_internal_model(&order_with_id);
        assert!(models.is_err());
        assert_eq!(AsObjectError(&json!({})).to_string(), models.unwrap_err().to_string());
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
