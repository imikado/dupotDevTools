use crate::infrastructure::ui::features::{self, RegisteredFeature};

/// The tool catalog, sorted by section then name.
pub struct FeatureLoaderService {
    features: Vec<RegisteredFeature>,
}

impl Default for FeatureLoaderService {
    fn default() -> Self {
        Self::new(features::registry())
    }
}

impl FeatureLoaderService {
    pub fn new(mut features: Vec<RegisteredFeature>) -> Self {
        features.sort_by_key(|f| (f.entity.section, f.entity.name));
        Self { features }
    }

    pub fn sections(&self) -> Vec<&'static str> {
        let mut sections: Vec<_> = self.features.iter().map(|f| f.entity.section).collect();
        sections.dedup();
        sections
    }

    pub fn all_features(&self) -> Vec<RegisteredFeature> {
        self.features.clone()
    }

    pub fn features_in_section(&self, section: &str) -> Vec<RegisteredFeature> {
        self.features
            .iter()
            .filter(|f| f.entity.section == section)
            .cloned()
            .collect()
    }
}
