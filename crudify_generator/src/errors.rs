use serde_json::Value;
use thiserror::Error;

// from https://www.reddit.com/r/rust/comments/gj8inf/comment/fqlmknt/
#[derive(Error, Debug)]
pub enum JsonConverterError<'a> {
    #[error("Could not convert value to object: {0}")]
    AsObjectError(&'a Value),
}

// impl Error for JsonConverterError<'_> {
//     fn source(&self) -> Option<&(dyn Error + 'static)> {
//         match *self {
//             JsonConverterError::AsObjectError { value: _ } => None,
//         }
//     }
// }

// impl Display for JsonConverterError<'_> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match *self {
//             Self::AsObjectError { ref value } => {
//                 write!(f, "Could not convert value to object: {}", value)
//             }
//         }
//     }
// }
