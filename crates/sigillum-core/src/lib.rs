//! Deterministic orchestration primitives shared by every Sigillum interface.

#![forbid(unsafe_code)]

/// The workspace package version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Static identity information exposed to adapters.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProductInfo {
    /// Human-readable product name.
    pub name: &'static str,
    /// Semantic version of the core package.
    pub version: &'static str,
}

impl ProductInfo {
    /// Returns identity information for the running build.
    #[must_use]
    pub const fn current() -> Self {
        Self {
            name: "Sigillum",
            version: VERSION,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ProductInfo, VERSION};

    #[test]
    fn current_product_info_uses_package_version() {
        let info = ProductInfo::current();

        assert_eq!(info.name, "Sigillum");
        assert_eq!(info.version, VERSION);
    }

    #[test]
    fn version_is_valid_semver_triplet() {
        let segments = VERSION.split('.').collect::<Vec<_>>();

        assert_eq!(segments.len(), 3);
        assert!(segments.iter().all(|segment| segment.parse::<u64>().is_ok()));
    }
}

