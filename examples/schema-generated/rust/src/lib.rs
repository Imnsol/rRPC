use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Node {
    pub id: uuid::Uuid,
    pub title: String,
    pub position: [f64; 4],
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct HyperEdge {
    pub id: String,
    pub nodes: Vec<uuid::Uuid>,
    pub label: Option<String>,
}

#[cfg(test)]
mod tests { use super::*; use serde_json; use uuid;

    #[test]
    fn roundtrip_dummy() {
        // generation test left intentionally minimal for prototype
    }
}
