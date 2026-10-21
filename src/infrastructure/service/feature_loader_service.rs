use crate::domain::entity::FeatureEntity;
use crate::infrastructure::ui::features;

pub struct FeatureLoaderService {
    features: Vec<FeatureEntity>,
}

impl FeatureLoaderService {
    pub fn new() -> Self {
        let mut features = features::registry();
        features.sort_by(|a, b| (a.section, a.name).cmp(&(b.section, b.name)));
        Self { features }
    }

    pub fn sections(&self) -> Vec<&'static str> {
        let mut sections: Vec<_> = self.features.iter().map(|f| f.section).collect();
        sections.dedup();
        sections
    }

    pub fn all_features(&self) -> Vec<FeatureEntity> {
        self.features.clone()
    }

    pub fn features_in_section(&self, section: &str) -> Vec<FeatureEntity> {
        self.features
            .iter()
            .filter(|f| f.section == section)
            .cloned()
            .collect()
    }
}
