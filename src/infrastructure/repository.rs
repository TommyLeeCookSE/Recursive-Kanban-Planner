use crate::domain::error::DomainError;
use crate::domain::registry::CardRegistry;
use crate::infrastructure::logging::record_diagnostic;
use tracing::{Level, error, info};

/// A repository that serializes and deserializes the full Kanban board state to JSON.
pub struct JsonRepository;

impl JsonRepository {
    /// Serializes the `CardRegistry` to a pretty-printed JSON string.
    pub fn serialize_registry(registry: &CardRegistry) -> Result<String, DomainError> {
        let root_count = registry.get_root_cards().len();
        info!(root_count, "Serializing registry to JSON");

        serde_json::to_string_pretty(registry).map_err(|e| {
            let error =
                DomainError::InvalidOperation(format!("Failed to serialize registry to JSON: {e}"));
            error!(root_count, error = %error, "Registry serialization failed");
            record_diagnostic(
                Level::ERROR,
                "persistence",
                format!("Registry serialization failed: {error}"),
            );
            error
        })
    }

    /// Deserializes a robust JSON string back into a `CardRegistry`.
    pub fn deserialize_registry(json: &str) -> Result<CardRegistry, DomainError> {
        let payload_bytes = json.len();
        info!(payload_bytes, "Deserializing registry from JSON");

        serde_json::from_str(json).map_err(|e| {
            let error = DomainError::InvalidOperation(format!(
                "Failed to deserialize registry from JSON: {e}"
            ));
            error!(payload_bytes, error = %error, "Registry deserialization failed");
            record_diagnostic(
                Level::ERROR,
                "persistence",
                format!("Registry deserialization failed: {error}"),
            );
            error
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
        info!(
            storage_key = Self::STORAGE_KEY,
            root_count = registry.get_root_cards().len(),
            "Saving registry to localStorage"
        );
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

        info!(
            storage_key = Self::STORAGE_KEY,
            "Saved registry to localStorage"
        );
        Ok(())
    }

    /// Loads the registry from `localStorage`. Returns `None` if no state was found.
    pub fn load_from_local_storage() -> Result<Option<CardRegistry>, DomainError> {
        info!(
            storage_key = Self::STORAGE_KEY,
            "Loading registry from localStorage"
        );
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
            info!(
                storage_key = Self::STORAGE_KEY,
                root_count = registry.get_root_cards().len(),
                "Loaded registry from localStorage"
            );
            Ok(Some(registry))
        } else {
            info!(
                storage_key = Self::STORAGE_KEY,
                "No persisted registry found"
            );
            Ok(None)
        }
    }
}

/// A small platform-aware persistence facade used by the interface layer.
pub struct AppPersistence;

impl AppPersistence {
    /// Loads the registry for the current platform.
    #[cfg(target_arch = "wasm32")]
    pub fn load_registry() -> Result<Option<CardRegistry>, DomainError> {
        info!(
            platform = "web",
            "Delegating persistence load to browser storage"
        );
        LocalStorageRepository::load_from_local_storage()
    }

    /// Loads the registry for the current platform.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_registry() -> Result<Option<CardRegistry>, DomainError> {
        let error = DomainError::InvalidOperation(
            "Persistence is not yet supported on this platform".into(),
        );
        error!(platform = crate::infrastructure::logging::target_name(), error = %error, "Persistence load unsupported on this platform");
        record_diagnostic(
            Level::ERROR,
            "persistence",
            format!("Persistence load unsupported: {error}"),
        );
        Err(error)
    }

    /// Saves the registry for the current platform.
    #[cfg(target_arch = "wasm32")]
    pub fn save_registry(registry: &CardRegistry) -> Result<(), DomainError> {
        info!(
            platform = "web",
            root_count = registry.get_root_cards().len(),
            "Delegating persistence save to browser storage"
        );
        LocalStorageRepository::save_to_local_storage(registry)
    }

    /// Saves the registry for the current platform.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save_registry(_registry: &CardRegistry) -> Result<(), DomainError> {
        let error = DomainError::InvalidOperation(
            "Persistence is not yet supported on this platform".into(),
        );
        error!(platform = crate::infrastructure::logging::target_name(), error = %error, "Persistence save unsupported on this platform");
        record_diagnostic(
            Level::ERROR,
            "persistence",
            format!("Persistence save unsupported: {error}"),
        );
        Err(error)
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

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_app_persistence_non_web_load_is_explicitly_unsupported() {
        assert!(matches!(
            AppPersistence::load_registry(),
            Err(DomainError::InvalidOperation(_))
        ));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_app_persistence_non_web_save_is_explicitly_unsupported() {
        let registry = CardRegistry::new();
        assert!(matches!(
            AppPersistence::save_registry(&registry),
            Err(DomainError::InvalidOperation(_))
        ));
    }
}
