use crate::InternalModel;

pub fn create_get_all_entities(model: &InternalModel) -> String {
    let mut sql: String = "SELECT ".to_string();

    let fields: String = match &model.properties {
        None => "*".to_string(),
        Some(properties) => {
            properties.keys().map(|p| p.as_ref()).collect::<Vec<_>>().join(", ")
        }
    };

    sql.push_str(&fields);
    sql.push_str(&format!(" FROM {}", &model.name.to_lowercase()));

    sql
}

pub fn create_create_entitiy(model: &InternalModel) -> String {
    let mut sql: String = "INSERT INTO ".to_string();

    let fields: String = match &model.properties {
        None => "*".to_string(),
        Some(properties) => {
            properties.keys().map(|p| p.as_ref()).collect::<Vec<_>>().join(", ")
        }
    };

    sql.push_str(&fields);
    sql.push_str(&format!(" FROM {}", &model.name.to_lowercase()));

    sql
}

#[cfg(test)]
mod tests {
    use indexmap::{indexmap, IndexMap};

    use super::*;

    impl InternalModel {
        fn new(name: String) -> InternalModel {
            InternalModel {
                name,
                properties: None,
            }
        }

        fn new_with_props(name: String, props: IndexMap<String, String>) -> InternalModel {
            InternalModel {
                name,
                properties: Some(props),
            }
        }
    }

    #[test]
    fn test_create_entity_with_multiple_properties() {
        let props = indexmap! {"id".to_string() => "i64".to_string(), "name".to_string() => "Sting".to_string()};
        let expected = "INSERT INTO (id, name) VALUES (?, ?)";
        assert_eq!(expected, create_create_entitiy(&InternalModel::new_with_props("Order".to_string(), props)));
    }

    #[test]
    fn test_get_entities_with_multiple_properties() {
        let props = indexmap! {"id".to_string() => "i64".to_string(), "name".to_string() => "Sting".to_string()};
        let expected = "SELECT id, name FROM order";
        assert_eq!(expected, create_get_all_entities(&InternalModel::new_with_props("Order".to_string(), props)));
    }

    #[test]
    fn test_get_entities_with_property() {
        let props = indexmap! {"id".to_string() => "i64".to_string()};
        let expected = "SELECT id FROM order";
        assert_eq!(expected, create_get_all_entities(&InternalModel::new_with_props("Order".to_string(), props)));
    }

    #[test]
    fn test_get_entities_without_properties() {
        let expected = "SELECT * FROM order";
        assert_eq!(expected, create_get_all_entities(&InternalModel::new("Order".to_string())));
    }
}
