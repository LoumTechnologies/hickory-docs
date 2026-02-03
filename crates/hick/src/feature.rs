//! Feature system for conditional code generation.
//!
//! Features are named, optional capabilities that can be enabled or disabled.
//! Features can depend on other features via the `requires` attribute.
//!
//! Example:
//! ```xml
//! <hick:feature name="auth" description="Authentication and authorization" />
//! <hick:feature name="billing" description="Stripe billing" />
//! <hick:feature name="premium" description="Premium tiers" requires="auth billing" />
//! ```
//!
//! When `premium` is enabled, `auth` and `billing` are automatically enabled.

use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// A feature definition.
#[derive(Debug, Clone)]
pub struct FeatureDef {
    /// Feature name (identifier).
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Names of features this feature requires.
    pub requires: Vec<String>,
}

impl FeatureDef {
    /// Create a new feature definition.
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            requires: Vec::new(),
        }
    }

    /// Add a required feature.
    pub fn with_requires(mut self, required: impl Into<String>) -> Self {
        self.requires.push(required.into());
        self
    }
}

/// Errors that can occur when working with features.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum FeatureError {
    #[error("unknown feature: '{0}'")]
    UnknownFeature(String),

    #[error("feature '{feature}' requires '{required}' which is not enabled")]
    MissingDependency { feature: String, required: String },

    #[error("circular dependency detected: {}", .0.join(" -> "))]
    CircularDependency(Vec<String>),

    #[error("feature '{feature}' requires unknown feature '{required}'")]
    UnknownDependency { feature: String, required: String },
}

/// Registry of all defined features.
#[derive(Debug, Clone, Default)]
pub struct FeatureRegistry {
    /// Feature definitions by name.
    features: HashMap<String, FeatureDef>,
}

impl FeatureRegistry {
    /// Create an empty feature registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a feature definition.
    pub fn register(&mut self, def: FeatureDef) {
        self.features.insert(def.name.clone(), def);
    }

    /// Get a feature by name.
    pub fn get(&self, name: &str) -> Option<&FeatureDef> {
        self.features.get(name)
    }

    /// Get all feature definitions.
    pub fn all(&self) -> impl Iterator<Item = &FeatureDef> {
        self.features.values()
    }

    /// Get all feature names.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.features.keys().map(|s| s.as_str())
    }

    /// Check if a feature is defined.
    pub fn is_defined(&self, name: &str) -> bool {
        self.features.contains_key(name)
    }

    /// Validate that all feature dependencies reference defined features
    /// and that there are no circular dependencies.
    pub fn validate(&self) -> Result<(), FeatureError> {
        // Check all dependencies reference defined features
        for def in self.features.values() {
            for required in &def.requires {
                if !self.features.contains_key(required) {
                    return Err(FeatureError::UnknownDependency {
                        feature: def.name.clone(),
                        required: required.clone(),
                    });
                }
            }
        }

        // Check for circular dependencies using DFS
        for name in self.features.keys() {
            self.check_circular_deps(name, &mut Vec::new(), &mut HashSet::new())?;
        }

        Ok(())
    }

    /// Depth-first search to detect circular dependencies.
    fn check_circular_deps(
        &self,
        current: &str,
        path: &mut Vec<String>,
        visited: &mut HashSet<String>,
    ) -> Result<(), FeatureError> {
        if path.contains(&current.to_string()) {
            // Found a cycle
            path.push(current.to_string());
            let cycle_start = path.iter().position(|p| p == current).unwrap();
            return Err(FeatureError::CircularDependency(
                path[cycle_start..].to_vec(),
            ));
        }

        if visited.contains(current) {
            // Already fully explored this node, no cycle through here
            return Ok(());
        }

        path.push(current.to_string());

        if let Some(def) = self.features.get(current) {
            for required in &def.requires {
                self.check_circular_deps(required, path, visited)?;
            }
        }

        path.pop();
        visited.insert(current.to_string());

        Ok(())
    }

    /// Get the transitive closure of all features required by the given feature.
    /// Includes the feature itself.
    pub fn transitive_requirements(&self, name: &str) -> HashSet<String> {
        let mut result = HashSet::new();
        self.collect_requirements(name, &mut result);
        result
    }

    fn collect_requirements(&self, name: &str, result: &mut HashSet<String>) {
        if result.contains(name) {
            return;
        }
        result.insert(name.to_string());

        if let Some(def) = self.features.get(name) {
            for required in &def.requires {
                self.collect_requirements(required, result);
            }
        }
    }
}

/// A set of enabled features.
#[derive(Debug, Clone, Default)]
pub struct FeatureSet {
    /// Enabled feature names.
    enabled: HashSet<String>,
}

impl FeatureSet {
    /// Create an empty feature set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a feature set from a list of feature names.
    pub fn from_names(names: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            enabled: names.into_iter().map(|n| n.into()).collect(),
        }
    }

    /// Parse a comma-separated list of feature names.
    pub fn parse(s: &str) -> Self {
        let enabled = s
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        Self { enabled }
    }

    /// Enable a feature.
    pub fn enable(&mut self, name: impl Into<String>) {
        self.enabled.insert(name.into());
    }

    /// Disable a feature.
    pub fn disable(&mut self, name: &str) {
        self.enabled.remove(name);
    }

    /// Check if a feature is enabled.
    pub fn is_enabled(&self, name: &str) -> bool {
        self.enabled.contains(name)
    }

    /// Get all enabled feature names.
    pub fn enabled(&self) -> impl Iterator<Item = &str> {
        self.enabled.iter().map(|s| s.as_str())
    }

    /// Get the number of enabled features.
    pub fn len(&self) -> usize {
        self.enabled.len()
    }

    /// Check if no features are enabled.
    pub fn is_empty(&self) -> bool {
        self.enabled.is_empty()
    }

    /// Validate that all enabled features are defined and their dependencies are satisfied.
    /// Returns an expanded feature set that includes all transitive dependencies.
    pub fn validate_and_expand(
        &self,
        registry: &FeatureRegistry,
    ) -> Result<FeatureSet, FeatureError> {
        // First validate the registry itself
        registry.validate()?;

        // Check all enabled features are defined
        for name in &self.enabled {
            if !registry.is_defined(name) {
                return Err(FeatureError::UnknownFeature(name.clone()));
            }
        }

        // Expand to include all transitive dependencies
        let mut expanded = HashSet::new();
        for name in &self.enabled {
            let deps = registry.transitive_requirements(name);
            expanded.extend(deps);
        }

        Ok(FeatureSet { enabled: expanded })
    }

    /// Validate that all enabled features have their dependencies satisfied.
    /// Does NOT auto-expand dependencies.
    pub fn validate_dependencies(&self, registry: &FeatureRegistry) -> Result<(), FeatureError> {
        for name in &self.enabled {
            if let Some(def) = registry.get(name) {
                for required in &def.requires {
                    if !self.enabled.contains(required) {
                        return Err(FeatureError::MissingDependency {
                            feature: name.clone(),
                            required: required.clone(),
                        });
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // FeatureDef tests
    // -------------------------------------------------------------------------

    #[test]
    fn feature_def_builder() {
        let def = FeatureDef::new("auth", "Authentication")
            .with_requires("users")
            .with_requires("sessions");

        assert_eq!(def.name, "auth");
        assert_eq!(def.description, "Authentication");
        assert_eq!(def.requires, vec!["users", "sessions"]);
    }

    // -------------------------------------------------------------------------
    // FeatureRegistry tests
    // -------------------------------------------------------------------------

    #[test]
    fn registry_register_and_get() {
        let mut registry = FeatureRegistry::new();
        registry.register(FeatureDef::new("auth", "Authentication"));

        assert!(registry.is_defined("auth"));
        assert!(!registry.is_defined("billing"));

        let def = registry.get("auth").unwrap();
        assert_eq!(def.name, "auth");
    }

    #[test]
    fn registry_validate_unknown_dependency() {
        let mut registry = FeatureRegistry::new();
        registry.register(
            FeatureDef::new("premium", "Premium features").with_requires("billing"),
        );

        let err = registry.validate().unwrap_err();
        assert!(matches!(
            err,
            FeatureError::UnknownDependency { feature, required }
            if feature == "premium" && required == "billing"
        ));
    }

    #[test]
    fn registry_validate_circular_dependency() {
        let mut registry = FeatureRegistry::new();
        registry.register(FeatureDef::new("a", "Feature A").with_requires("b"));
        registry.register(FeatureDef::new("b", "Feature B").with_requires("c"));
        registry.register(FeatureDef::new("c", "Feature C").with_requires("a"));

        let err = registry.validate().unwrap_err();
        assert!(matches!(err, FeatureError::CircularDependency(_)));
    }

    #[test]
    fn registry_validate_self_dependency() {
        let mut registry = FeatureRegistry::new();
        registry.register(FeatureDef::new("a", "Feature A").with_requires("a"));

        let err = registry.validate().unwrap_err();
        assert!(matches!(err, FeatureError::CircularDependency(_)));
    }

    #[test]
    fn registry_validate_valid() {
        let mut registry = FeatureRegistry::new();
        registry.register(FeatureDef::new("auth", "Authentication"));
        registry.register(FeatureDef::new("billing", "Billing"));
        registry.register(
            FeatureDef::new("premium", "Premium")
                .with_requires("auth")
                .with_requires("billing"),
        );

        assert!(registry.validate().is_ok());
    }

    #[test]
    fn registry_transitive_requirements() {
        let mut registry = FeatureRegistry::new();
        registry.register(FeatureDef::new("users", "User management"));
        registry.register(FeatureDef::new("auth", "Authentication").with_requires("users"));
        registry.register(FeatureDef::new("billing", "Billing"));
        registry.register(
            FeatureDef::new("premium", "Premium")
                .with_requires("auth")
                .with_requires("billing"),
        );

        let reqs = registry.transitive_requirements("premium");
        assert!(reqs.contains("premium"));
        assert!(reqs.contains("auth"));
        assert!(reqs.contains("billing"));
        assert!(reqs.contains("users"));
        assert_eq!(reqs.len(), 4);
    }

    // -------------------------------------------------------------------------
    // FeatureSet tests
    // -------------------------------------------------------------------------

    #[test]
    fn feature_set_parse() {
        let set = FeatureSet::parse("auth, billing, premium");
        assert!(set.is_enabled("auth"));
        assert!(set.is_enabled("billing"));
        assert!(set.is_enabled("premium"));
        assert_eq!(set.len(), 3);
    }

    #[test]
    fn feature_set_parse_empty() {
        let set = FeatureSet::parse("");
        assert!(set.is_empty());
    }

    #[test]
    fn feature_set_from_names() {
        let set = FeatureSet::from_names(["auth", "billing"]);
        assert!(set.is_enabled("auth"));
        assert!(set.is_enabled("billing"));
        assert!(!set.is_enabled("premium"));
    }

    #[test]
    fn feature_set_enable_disable() {
        let mut set = FeatureSet::new();
        set.enable("auth");
        assert!(set.is_enabled("auth"));

        set.disable("auth");
        assert!(!set.is_enabled("auth"));
    }

    #[test]
    fn feature_set_validate_unknown_feature() {
        let registry = FeatureRegistry::new();
        let set = FeatureSet::from_names(["unknown"]);

        let err = set.validate_and_expand(&registry).unwrap_err();
        assert!(matches!(err, FeatureError::UnknownFeature(name) if name == "unknown"));
    }

    #[test]
    fn feature_set_validate_and_expand() {
        let mut registry = FeatureRegistry::new();
        registry.register(FeatureDef::new("users", "User management"));
        registry.register(FeatureDef::new("auth", "Authentication").with_requires("users"));
        registry.register(FeatureDef::new("billing", "Billing"));
        registry.register(
            FeatureDef::new("premium", "Premium")
                .with_requires("auth")
                .with_requires("billing"),
        );

        // Enable only "premium", should expand to include all deps
        let set = FeatureSet::from_names(["premium"]);
        let expanded = set.validate_and_expand(&registry).unwrap();

        assert!(expanded.is_enabled("premium"));
        assert!(expanded.is_enabled("auth"));
        assert!(expanded.is_enabled("billing"));
        assert!(expanded.is_enabled("users"));
        assert_eq!(expanded.len(), 4);
    }

    #[test]
    fn feature_set_validate_dependencies_missing() {
        let mut registry = FeatureRegistry::new();
        registry.register(FeatureDef::new("auth", "Authentication"));
        registry.register(FeatureDef::new("premium", "Premium").with_requires("auth"));

        // Enable premium but not auth
        let set = FeatureSet::from_names(["premium"]);
        let err = set.validate_dependencies(&registry).unwrap_err();

        assert!(matches!(
            err,
            FeatureError::MissingDependency { feature, required }
            if feature == "premium" && required == "auth"
        ));
    }

    #[test]
    fn feature_set_validate_dependencies_ok() {
        let mut registry = FeatureRegistry::new();
        registry.register(FeatureDef::new("auth", "Authentication"));
        registry.register(FeatureDef::new("premium", "Premium").with_requires("auth"));

        // Enable both premium and auth
        let set = FeatureSet::from_names(["premium", "auth"]);
        assert!(set.validate_dependencies(&registry).is_ok());
    }
}
