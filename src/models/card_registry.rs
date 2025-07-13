use std::collections::HashMap;
use uuid::Uuid;

use super::card::Card;

pub struct CardRegistry {
    cards: HashMap<Uuid, Card>,
}


impl CardRegistry {
    pub fn new() -> Self {
        Self {
            cards: HashMap::new(),
        }
    }

    pub fn add_card(&mut self, card: Card) -> Result<(), String> {
        let id = card.id;

        if self.cards.contains_key(&id) {
            return Err("Card with this ID already exists.".to_string());
        }

        if let Some(parent_id) = card.parent_id {
            if let Some(parent_card) = self.cards.get_mut(&parent_id) {
                parent_card.children.push(card.id);
            } else {
                return Err("Parent card not found.".to_string());
            }
        }

        self.cards.insert(id, card);
        Ok(())
    }
}