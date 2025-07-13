// src/models/card.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: Uuid,
    pub title: String,
    pub parent_id: Option<Uuid>,
    pub bucket: Option<String>,
    pub buckets: Vec<String>,
    pub children: Vec<Uuid>,
}

impl Card {
    pub fn new(title: String, parent_id: Option<Uuid>, bucket: Option<String>, buckets: Vec<String>, children: Vec<Uuid>) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            parent_id,
            bucket,
            buckets,
            children: Vec::new(),
        }
    }
}