use super::contract::FeatureContract;

#[derive(Clone)]
pub struct FeatureEntity {
    pub name: &'static str,
    pub description: &'static str,
    pub section: &'static str,
    pub icon: Option<&'static [u8]>,
    pub create: fn() -> Box<dyn FeatureContract>,
}
