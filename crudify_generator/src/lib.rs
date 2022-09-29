mod file_creator;

pub struct InternalModel {
    pub name: String
}

pub type InternalModels = Vec<InternalModel>;

pub fn generate(user_id: &str, models: InternalModels) {
    file_creator::write_all(user_id, models);
    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let models = vec![InternalModel {
            name: "Order".to_string(),
        }];
        generate("user_id", models);
    }
}
