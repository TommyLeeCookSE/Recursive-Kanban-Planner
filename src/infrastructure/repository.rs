use crate::domain::error::DomainError;
use crate::domain::registry::CardRegistry;

/// A repository that serializes and deserializes the full Kanban board state to JSON.
pub struct JsonRepository;

impl JsonRepository {
    /// Serializes the `CardRegistry` to a pretty-printed JSON string.
    pub fn serialize_registry(registry: &CardRegistry) -> Result<String, DomainError> {
        serde_json::to_string_pretty(registry).map_err(|e| {
            DomainError::InvalidOperation(format!("Failed to serialize registry to JSON: {e}"))
        })
    }

    /// Deserializes a robust JSON string back into a `CardRegistry`.
    pub fn deserialize_registry(json: &str) -> Result<CardRegistry, DomainError> {
        serde_json::from_str(json).map_err(|e| {
            DomainError::InvalidOperation(format!(
                "Failed to deserialize registry from JSON: {e}"
            ))
        })
    }
}

/// A repository that saves and loads the JSON state to/from the browser's `localStorage`.
/// Note: This will only work in environments where `web_sys::window()` is fully available (e.g., WASM in the browser).
pub struct LocalStorageRepository;

impl LocalStorageRepository {
    const STORAGE_KEY: &'static str = "kanban_planner_state";

    /// Saves the registry to `localStorage`.
    pub fn save_to_local_storage(registry: &CardRegistry) -> Result<(), DomainError> {
        let json = JsonRepository::serialize_registry(registry)?;

        let window = web_sys::window().ok_or_else(|| {
            DomainError::InvalidOperation("Failed to access global window object".into())
        })?;

        let storage = window
            .local_storage()
            .map_err(|_| DomainError::InvalidOperation("Failed to access local_storage".into()))?
            .ok_or_else(|| {
                DomainError::InvalidOperation("local_storage is not available".into())
            })?;

        storage.set_item(Self::STORAGE_KEY, &json).map_err(|_| {
            DomainError::InvalidOperation("Failed to write to local_storage".into())
        })?;

        Ok(())
    }

    /// Loads the registry from `localStorage`. Returns `None` if no state was found.
    pub fn load_from_local_storage() -> Result<Option<CardRegistry>, DomainError> {
        let window = web_sys::window().ok_or_else(|| {
            DomainError::InvalidOperation("Failed to access global window object".into())
        })?;

        let storage = window
            .local_storage()
            .map_err(|_| DomainError::InvalidOperation("Failed to access local_storage".into()))?
            .ok_or_else(|| {
                DomainError::InvalidOperation("local_storage is not available".into())
            })?;

        let json_opt = storage.get_item(Self::STORAGE_KEY).map_err(|_| {
            DomainError::InvalidOperation("Failed to read from local_storage".into())
        })?;

        if let Some(json) = json_opt {
            let registry = JsonRepository::deserialize_registry(&json)?;
            Ok(Some(registry))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_integration() {
        // create -> serialize -> deserialize -> verify full structural equality
        let mut original = CardRegistry::new();
        let root_id = original.create_root_card("My Project".into()).unwrap();
        let bucket_id = original.add_bucket(root_id, "In Progress".into()).unwrap();
        original
            .create_child_card("My Task".into(), root_id, bucket_id)
            .unwrap();

        let json = JsonRepository::serialize_registry(&original).expect("Serialization failed");

        let deserialized =
            JsonRepository::deserialize_registry(&json).expect("Deserialization failed");

        assert_eq!(
            original, deserialized,
            "Deserialized registry does not match original"
        );
    }
}
