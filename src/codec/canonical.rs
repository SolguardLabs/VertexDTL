use serde::Serialize;

use crate::{VertexError, VertexResult};

pub fn canonical_bytes<T: Serialize>(value: &T) -> VertexResult<Vec<u8>> {
    serde_json::to_vec(value).map_err(|error| VertexError::Serialization(error.to_string()))
}
