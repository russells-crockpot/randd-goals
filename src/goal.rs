use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[serde(rename_all = "kebab-case")]
pub struct Goal {
    #[builder(default = "self.default_slug()")]
    pub slug: String,
    pub goal: String,
    #[builder(default = "1.0")]
    pub weight: f64,
    #[builder(default = "false")]
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub disabled: bool,
    #[serde(skip_serializing_if = "std::vec::Vec::is_empty")]
    pub tags: Vec<String>,
}

impl GoalBuilder {
    fn default_slug(&self) -> String {
        slug::slugify(self.goal.as_ref().unwrap())
    }
}
