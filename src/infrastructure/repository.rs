//! Persistence and serialization logic for the Kanban Planner.
//!
//! This module provides the repository patterns for storing the registry as JSON
//! in browser `localStorage` or other future backends.
//!
//! For a conceptual overview of the repository pattern in this project, see
//! `docs/rust-for-python-devs.md`.

use crate::domain::error::DomainError;
use crate::domain::registry::CardRegistry;
use crate::infrastructure::logging::record_diagnostic;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{Level, error, info, warn};

const CURRENT_SCHEMA_VERSION: u8 = 2;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct PersistedRegistry {
    schema_version: u8,
    registry: CardRegistry,
}

/// A repository for serializing and deserializing the card registry to and from JSON.
///
/// # Examples
///
/// ```rust
/// use kanban_planner::domain::registry::CardRegistry;
/// use kanban_planner::infrastructure::repository::JsonRepository;
///
/// let registry = CardRegistry::new();
/// let json = JsonRepository::serialize_registry(&registry).unwrap();
/// let deserialized = JsonRepository::deserialize_registry(&json).unwrap();
/// assert_eq!(registry, deserialized);
/// ```
pub struct JsonRepository;

impl JsonRepository {
    /// Serializes the given registry into a pretty-printed JSON string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use kanban_planner::domain::registry::CardRegistry;
    /// use kanban_planner::infrastructure::repository::JsonRepository;
    ///
    /// let registry = CardRegistry::new();
    /// let json = JsonRepository::serialize_registry(&registry).unwrap();
    /// assert!(json.contains("schema_version"));
    /// ```
    pub fn serialize_registry(registry: &CardRegistry) -> Result<String, DomainError> {
        let workspace_child_count = registry.workspace_child_count();
        info!(workspace_child_count, "Serializing registry to JSON");

        // We wrap the registry reference in our persistence envelope to avoid a redundant clone
        // of the entire registry before serialization.
        #[derive(Serialize)]
        struct PersistedRegistryRef<'a> {
            schema_version: u8,
            registry: &'a CardRegistry,
        }

        let persisted = PersistedRegistryRef {
            schema_version: CURRENT_SCHEMA_VERSION,
            registry,
        };

        // Use compact serialization for internal persistence to improve performance
        // when the registry contains many cards.
        serde_json::to_string(&persisted).map_err(|e| {
            let error =
                DomainError::InvalidOperation(format!("Failed to serialize registry to JSON: {e}"));
            error!(
                workspace_child_count,
                error = %error,
                "Registry serialization failed"
            );
            record_diagnostic(
                Level::ERROR,
                "persistence",
                format!("Registry serialization failed: {error}"),
            );
            error
        })
    }

    /// Deserializes a registry from a JSON string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use kanban_planner::domain::registry::CardRegistry;
    /// use kanban_planner::infrastructure::repository::JsonRepository;
    ///
    /// let registry = CardRegistry::new();
    /// let json = JsonRepository::serialize_registry(&registry).unwrap();
    /// let deserialized = JsonRepository::deserialize_registry(&json).unwrap();
    /// assert_eq!(registry, deserialized);
    /// ```
    pub fn deserialize_registry(json: &str) -> Result<CardRegistry, DomainError> {
        let payload_bytes = json.len();
        info!(payload_bytes, "Deserializing registry from JSON");

        let value: Value = match serde_json::from_str(json) {
            Ok(value) => value,
            Err(error_value) => {
                return Err(deserialization_error(
                    payload_bytes,
                    format!("Failed to deserialize registry from JSON: {error_value}"),
                ));
            }
        };

        if contains_legacy_bucket_model(&value) {
            return Err(deserialization_error(
                payload_bytes,
                "Older bucket-based data is incompatible with the card-only model.".to_string(),
            ));
        }

        let registry = match value.get("schema_version") {
            Some(schema_version_value) => {
                let Some(schema_version) =
                    schema_version_value.as_u64().map(|version| version as u8)
                else {
                    return Err(deserialization_error(
                        payload_bytes,
                        "Persisted schema version must be a positive integer".to_string(),
                    ));
                };

                if schema_version != CURRENT_SCHEMA_VERSION {
                    return Err(deserialization_error(
                        payload_bytes,
                        format!("Unsupported persisted schema version {schema_version}"),
                    ));
                }

                match serde_json::from_value::<PersistedRegistry>(value) {
                    Ok(persisted) => persisted.registry,
                    Err(error_value) => {
                        return Err(deserialization_error(
                            payload_bytes,
                            format!("Failed to decode persisted registry payload: {error_value}"),
                        ));
                    }
                }
            }
            None => {
                return Err(deserialization_error(
                    payload_bytes,
                    "Persisted registry is missing the schema envelope".to_string(),
                ));
            }
        };

        if let Err(error_value) = registry.validate() {
            return Err(deserialization_error(
                payload_bytes,
                format!("Registry validation failed after deserialization: {error_value}"),
            ));
        }

        Ok(registry)
    }
}

/// A repository for storing the registry in the browser's `localStorage`.
///
/// # Examples
///
/// ```ignore
/// use kanban_planner::domain::registry::CardRegistry;
/// use kanban_planner::infrastructure::repository::LocalStorageRepository;
///
/// let registry = CardRegistry::new();
/// LocalStorageRepository::save_to_local_storage(&registry).unwrap();
/// let loaded = LocalStorageRepository::load_from_local_storage().unwrap().unwrap();
/// assert_eq!(registry, loaded);
/// ```
pub struct LocalStorageRepository;

impl LocalStorageRepository {
    const STORAGE_KEY: &'static str = "kanban_planner_state";

    /// Saves the registry to the browser's `localStorage`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use kanban_planner::domain::registry::CardRegistry;
    /// use kanban_planner::infrastructure::repository::LocalStorageRepository;
    ///
    /// let registry = CardRegistry::new();
    /// LocalStorageRepository::save_to_local_storage(&registry).unwrap();
    /// ```
    pub fn save_to_local_storage(registry: &CardRegistry) -> Result<(), DomainError> {
        info!(
            storage_key = Self::STORAGE_KEY,
            workspace_child_count = registry.workspace_child_count(),
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

    /// Loads the registry from the browser's `localStorage`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use kanban_planner::infrastructure::repository::LocalStorageRepository;
    ///
    /// if let Some(registry) = LocalStorageRepository::load_from_local_storage().unwrap() {
    ///     println!("Loaded registry with {} children", registry.workspace_child_count());
    /// }
    /// ```
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
                workspace_child_count = registry.workspace_child_count(),
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

    /// Clears the registry from the browser's `localStorage`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use kanban_planner::infrastructure::repository::LocalStorageRepository;
    ///
    /// LocalStorageRepository::clear_local_storage().unwrap();
    /// ```
    pub fn clear_local_storage() -> Result<(), DomainError> {
        info!(
            storage_key = Self::STORAGE_KEY,
            "Clearing registry from localStorage"
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

        storage
            .remove_item(Self::STORAGE_KEY)
            .map_err(|_| DomainError::InvalidOperation("Failed to clear local_storage".into()))?;

        info!(
            storage_key = Self::STORAGE_KEY,
            "Cleared registry from localStorage"
        );
        Ok(())
    }
}

/// A facade for cross-platform persistence.
///
/// # Examples
///
/// ```ignore
/// use kanban_planner::domain::registry::CardRegistry;
/// use kanban_planner::infrastructure::repository::AppPersistence;
///
/// let registry = CardRegistry::new();
/// AppPersistence::save_registry(&registry).unwrap();
/// let loaded = AppPersistence::load_registry().unwrap().unwrap();
/// assert_eq!(registry, loaded);
/// ```
pub struct AppPersistence;

impl AppPersistence {
    /// Loads the registry from the current platform's persistent storage.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use kanban_planner::infrastructure::repository::AppPersistence;
    ///
    /// if let Some(registry) = AppPersistence::load_registry().unwrap() {
    ///     println!("Loaded registry");
    /// }
    /// ```
    #[cfg(target_arch = "wasm32")]
    pub fn load_registry() -> Result<Option<CardRegistry>, DomainError> {
        info!(
            platform = "web",
            "Delegating persistence load to browser storage"
        );
        LocalStorageRepository::load_from_local_storage()
    }

    /// Loads the registry from the current platform's persistent storage.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use kanban_planner::infrastructure::repository::AppPersistence;
    ///
    /// if let Some(registry) = AppPersistence::load_registry().unwrap() {
    ///     println!("Loaded registry");
    /// }
    /// ```
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

    /// Saves the registry to the current platform's persistent storage.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use kanban_planner::domain::registry::CardRegistry;
    /// use kanban_planner::infrastructure::repository::AppPersistence;
    ///
    /// let registry = CardRegistry::new();
    /// AppPersistence::save_registry(&registry).unwrap();
    /// ```
    #[cfg(target_arch = "wasm32")]
    pub fn save_registry(registry: &CardRegistry) -> Result<(), DomainError> {
        info!(
            platform = "web",
            workspace_child_count = registry.workspace_child_count(),
            "Delegating persistence save to browser storage"
        );
        LocalStorageRepository::save_to_local_storage(registry)
    }

    /// Clears the registry from the current platform's persistent storage.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use kanban_planner::infrastructure::repository::AppPersistence;
    ///
    /// AppPersistence::clear_registry().unwrap();
    /// ```
    #[cfg(target_arch = "wasm32")]
    pub fn clear_registry() -> Result<(), DomainError> {
        info!(
            platform = "web",
            "Delegating persistence clear to browser storage"
        );
        LocalStorageRepository::clear_local_storage()
    }

    /// Saves the registry to the current platform's persistent storage.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use kanban_planner::domain::registry::CardRegistry;
    /// use kanban_planner::infrastructure::repository::AppPersistence;
    ///
    /// let registry = CardRegistry::new();
    /// AppPersistence::save_registry(&registry).unwrap();
    /// ```
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

    /// Clears the registry from the current platform's persistent storage.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use kanban_planner::infrastructure::repository::AppPersistence;
    ///
    /// AppPersistence::clear_registry().unwrap();
    /// ```
    #[cfg(not(target_arch = "wasm32"))]
    pub fn clear_registry() -> Result<(), DomainError> {
        let error = DomainError::InvalidOperation(
            "Persistence is not yet supported on this platform".into(),
        );
        error!(platform = crate::infrastructure::logging::target_name(), error = %error, "Persistence clear unsupported on this platform");
        record_diagnostic(
            Level::ERROR,
            "persistence",
            format!("Persistence clear unsupported: {error}"),
        );
        Err(error)
    }
}

fn contains_legacy_bucket_model(value: &Value) -> bool {
    match value {
        Value::Object(map) => {
            map.contains_key("bucket_id")
                || map.contains_key("buckets")
                || map.values().any(contains_legacy_bucket_model)
        }
        Value::Array(values) => values.iter().any(contains_legacy_bucket_model),
        _ => false,
    }
}

fn deserialization_error(payload_bytes: usize, reason: String) -> DomainError {
    let error_value = DomainError::IncompatibleLegacyData(reason.clone());
    warn!(payload_bytes, error = %error_value, "Rejecting incompatible persisted state");
    record_diagnostic(
        Level::WARN,
        "persistence",
        format!("Persisted state was rejected: {reason}"),
    );
    error_value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_integration() {
        let mut original = CardRegistry::new();
        let workspace_id = original.workspace_card_id().unwrap();
        let project_id = original
            .create_child_card("My Project".into(), workspace_id)
            .unwrap();
        original
            .create_child_card("My Task".into(), project_id)
            .unwrap();

        let json = JsonRepository::serialize_registry(&original).expect("Serialization failed");
        let deserialized =
            JsonRepository::deserialize_registry(&json).expect("Deserialization failed");

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_deserialize_registry_rejects_blank_titles() {
        let mut original = CardRegistry::new();
        let workspace_id = original.workspace_card_id().unwrap();
        original
            .create_child_card("My Project".into(), workspace_id)
            .unwrap();

        let json = JsonRepository::serialize_registry(&original).expect("Serialization failed");
        let tampered = json.replacen("\"title\": \"My Project\"", "\"title\": \"   \"", 1);

        assert!(matches!(
            JsonRepository::deserialize_registry(&tampered),
            Err(DomainError::IncompatibleLegacyData(_))
        ));
    }

    #[test]
    fn test_deserialize_registry_rejects_legacy_bucket_payload() {
        let legacy_json = r#"{
  "store": {
    "01ARZ3NDEKTSV4RRFFQ69G5FAV": {
      "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
      "title": "Legacy",
      "parent_id": null,
      "children_ids": [],
      "bucket_id": null,
      "buckets": []
    }
  }
}"#;

        assert!(matches!(
            JsonRepository::deserialize_registry(legacy_json),
            Err(DomainError::IncompatibleLegacyData(_))
        ));
    }

    #[test]
    fn test_deserialize_registry_rejects_invalid_json_payload() {
        assert!(matches!(
            JsonRepository::deserialize_registry("{ definitely not json }"),
            Err(DomainError::IncompatibleLegacyData(_))
        ));
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

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_app_persistence_non_web_clear_is_explicitly_unsupported() {
        assert!(matches!(
            AppPersistence::clear_registry(),
            Err(DomainError::InvalidOperation(_))
        ));
    }
}
