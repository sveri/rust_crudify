// json -> rust code -> file system

use std::{error::Error, fmt::Display};

use indexmap::IndexMap;
use log::{error, info, warn};
use phf::phf_map;
use serde::Deserialize;
use serde_json::{Map, Value};

use crate::{InternalModel, InternalModels};

// from https://www.reddit.com/r/rust/comments/gj8inf/comment/fqlmknt/
#[derive(Debug)]
pub enum JsonConverterError<'a> {
    // AsObjectError,
    AsObjectError { value: &'a Value },
}

impl Error for JsonConverterError<'_> {
    // fn source(&self) -> Option<&(dyn Error + 'static)> {

    // }
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            JsonConverterError::AsObjectError { ref value } => None,
        }
    }
}

impl Display for JsonConverterError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::AsObjectError { ref value } => {
                write!(f, "Could not convert value to object: {}", value)
            }
        }
    }
}

// https://github.com/OAI/OpenAPI-Specification/blob/main/versions/3.0.3.md#schema-object
static DATATYPE_FORMAT_TO_RUST_DATATYPE: phf::Map<&'static str, &'static str> = phf_map! {
    "int64" => "i64",
    "int32" => "i32",
    "date" => "chrono::Date",
    "date-time" => "chrono::DateTime",
    "password" => "String",
    "byte" => "u8",
    "boolean" => "bool",
    "float" => "f32",
    "double" => "f64"
};

// https://github.com/OAI/OpenAPI-Specification/blob/main/versions/3.0.3.md#schema-object
static DATATYPE_TO_RUST_DATATYPE: phf::Map<&'static str, &'static str> = phf_map! {
    "integer" => "i64",
    "string" => "String",
    "boolean" => "bool",
    "number" => "f64"
};

#[derive(Deserialize, Debug)]
struct OA3Type {
    #[serde(rename = "type")]
    kind: String,
    format: Option<String>,
}

pub fn convert_to_internal_model(j: &Value) -> Result<InternalModels, JsonConverterError> {
    let mut internal_models = Vec::new();
    for (key, value) in j.as_object().unwrap() {
        if value.is_object() {
            let properties = parse_properties(&value)?;

            internal_models.push(InternalModel {
                name: key.to_string(),
                properties: Some(properties),
            })
        }
    }

    Ok(internal_models)
}

fn as_object(value: &Value) -> Result<&Map<String, Value>, JsonConverterError> {
    value.as_object().ok_or(JsonConverterError::AsObjectError { value: value })
}

fn as_object_with_context<'a>(value: &'a Value, ctx_value: &'a Value) -> Result<&Map<String, Value>, JsonConverterError> {
    value.as_object().ok_or(JsonConverterError::AsObjectError { value: ctx_value })
}

fn parse_data_type_from_object(property_type: &Map<String, Value>, value: &Value) -> String {
    let mut data_type = "String";
    if let Some(format) = property_type.get("format") {
        if DATATYPE_FORMAT_TO_RUST_DATATYPE.contains_key(format.as_str().unwrap()) {
            data_type = DATATYPE_FORMAT_TO_RUST_DATATYPE.get(format.as_str().unwrap()).unwrap();
        } else if let Some(type_value) = property_type.get("type") {
            data_type = DATATYPE_TO_RUST_DATATYPE.get(type_value.as_str().unwrap()).unwrap();
        } else {
            warn!(
                "No type or format found in data type property, assuming string as default. Value: {:?}",
                value
            );
        }
    }

    data_type.to_string()
}

fn parse_data_type_from_value(property_value: OA3Type, value_context: &Value) -> String {
    let mut data_type = "String";
    if let Some(format) = DATATYPE_FORMAT_TO_RUST_DATATYPE.get(property_value.format.unwrap().as_str()) {
        data_type = format;
    } else if let Some(type_value) = DATATYPE_TO_RUST_DATATYPE.get(&property_value.kind) {
        data_type = type_value;
    }
    // if let Some(format) = property_type.get("format") {
    //     if DATATYPE_FORMAT_TO_RUST_DATATYPE.contains_key(format.as_str().unwrap()) {
    //         data_type = DATATYPE_FORMAT_TO_RUST_DATATYPE.get(format.as_str().unwrap()).unwrap();
    //     } else if let Some(type_value) = property_type.get("type") {
    //         data_type = DATATYPE_TO_RUST_DATATYPE.get(type_value.as_str().unwrap()).unwrap();
    //     } else {
    //         warn!(
    //             "No type or format found in data type property, assuming string as default. Value: {:?}",
    //             value_context
    //         );
    //     }
    // }

    data_type.to_string()
}

fn parse_properties(value: &Value) -> Result<IndexMap<String, String>, JsonConverterError> {
    let mut property_map: IndexMap<String, String> = IndexMap::new();
    let o = as_object(value)?;
    if let Some(properties) = o.get("properties") {
        for (property_key, property_value) in as_object_with_context(properties, value)? {
            if property_key == "id" && property_value.is_object() {
                let property_object: OA3Type = serde_json::from_value(property_value.to_owned()).unwrap();
                property_object.
                // let property_type = as_object_with_context(property_value, value)?;
                // let data_type = parse_data_type_from_object(property_type, value);

                property_map.insert(property_key.to_string(), data_type.to_string());
            } else {
                warn!("The id value must be an object, but was {:?}", property_value);
            }
        }
    }

    Ok(property_map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
    fn with_id_property() {
        let order_with_id = json!({"Order": {"type": "object", "properties": {"id": {"type": "integer", "format": "int64"}}}});
        let models = convert_to_internal_model(&order_with_id).unwrap();
        assert_eq!("Order", models.get(0).unwrap().name);
        assert_eq!(
            "i64".to_string(),
            models.get(0).unwrap().properties.as_ref().unwrap().get("id").unwrap().to_string()
        );
    }

    #[test]
    fn with_id_property_without_type_object() {
        let order_with_id = json!({"Order": {"type": "object", "properties": {"id": {}}}});
        let models = convert_to_internal_model(&order_with_id).unwrap();
        assert_eq!("Order", models.get(0).unwrap().name);
        assert_eq!(
            "String".to_string(),
            models.get(0).unwrap().properties.as_ref().unwrap().get("id").unwrap().to_string()
        );
    }

    #[test]
    fn with_wrong_id_property() {
        init();
        let order_with_id = json!({"Order": {"type": "object", "properties": {"id": "foobar"}}});
        let models = convert_to_internal_model(&order_with_id).unwrap();
        assert_eq!("Order", models.get(0).unwrap().name);
    }

    #[test]
    fn without_properties() {
        let two_order_objects = json!({"Order": {}, "OrderTwo": {}});
        let models = convert_to_internal_model(&two_order_objects).unwrap();
        assert_eq!("Order", models.get(0).unwrap().name);
        assert_eq!("OrderTwo", models.get(1).unwrap().name);
    }

    fn init() {
        let _ = env_logger::builder().is_test(true).filter_level(log::LevelFilter::Debug).try_init();
    }
}
