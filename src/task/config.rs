use super::DEFAULT_WEIGHT;
use crate::config::DisabledOptions;
use derive_builder::Builder;
use getset::Getters;
use serde::{Deserialize, Serialize};
use std::{cell::OnceCell, marker::PhantomData, ops::AddAssign};

#[inline]
fn get_default_slug<S: AsRef<str>>(task: S) -> String {
    slug::slugify(task.as_ref())
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder, Getters)]
#[serde(rename_all = "kebab-case")]
#[builder(name = "TaskBuilder")]
#[getset(get = "pub")]
pub struct TaskConfig {
    #[builder(setter(custom = true))]
    #[getset(skip)]
    #[serde(with = "crate::serializers::once_cell")]
    slug: OnceCell<String>,
    pub task: String,
    #[builder(default)]
    #[serde(default, skip_serializing_if = "std::option::Option::is_none")]
    pub description: Option<String>,
    #[builder(default)]
    #[serde(default, skip_serializing_if = "std::option::Option::is_none")]
    pub max_occurrences: Option<u32>,
    #[builder(default)]
    #[serde(default, skip_serializing_if = "std::option::Option::is_none")]
    pub min_frequency: Option<u32>,
    #[builder(default = "DEFAULT_WEIGHT")]
    pub weight: f64,
    #[builder(default)]
    #[serde(default, skip_serializing_if = "DisabledOptions::is_enabled")]
    pub disabled: DisabledOptions,
    #[serde(default, skip_serializing_if = "std::vec::Vec::is_empty")]
    pub tags: Vec<String>,
}

impl TaskBuilder {
    #[inline]
    fn default_slug(&self) -> String {
        get_default_slug(self.task.as_ref().unwrap())
    }

    pub fn slug(&mut self, value: String) -> &mut Self {
        self.slug = if value.is_empty() {
            None
        } else {
            let cell = OnceCell::new();
            cell.set(value).ok();
            Some(cell)
        };
        self
    }

    pub fn tag<S: AsRef<str>>(&mut self, tag: S) -> &mut Self {
        let tag = String::from(tag.as_ref());
        self.tags = Some(if let Some(mut tags) = self.tags.take() {
            tags.push(tag);
            tags
        } else {
            vec![tag]
        });
        self
    }
}

impl TaskConfig {
    pub fn slug(&self) -> &str {
        self.slug.get_or_init(|| get_default_slug(&self.task))
    }

    pub fn enable(&mut self) {
        self.disabled = DisabledOptions::Enabled;
    }

    pub fn disable(&mut self) {
        self.disabled = DisabledOptions::Disabled;
    }

    /// Takes the values from the `other` argument, and overrides the values in this struct as long
    /// as the value in the other struct is not the default value. **Note**: the `slug` property is
    /// never overwritten.
    pub(crate) fn merge(&mut self, other: Self) {
        if !other.task.is_empty() {
            self.task = other.task;
        }
        if let Some(description) = other.description
            && !description.is_empty()
        {
            self.description = Some(description);
        }
        if other.weight != DEFAULT_WEIGHT {
            self.weight = other.weight;
        }
        if other.min_frequency.unwrap_or(0) != 0 {
            self.min_frequency = other.min_frequency;
        }
        if other.max_occurrences.unwrap_or(0) != 0 {
            self.max_occurrences = other.max_occurrences;
        }
        if other.disabled != DisabledOptions::Enabled {
            self.disabled = other.disabled;
        }
        for tag in other.tags.into_iter() {
            if !self.tags.contains(&tag) {
                self.tags.push(tag);
            }
        }
    }

    pub fn update(&mut self, other: TaskBuilder) {
        if let Some(task) = other.task {
            self.task = task;
        }
        if let Some(description) = other.description {
            self.description = description;
        }
        if let Some(weight) = other.weight {
            self.weight = weight;
        }
        if let Some(min_frequency) = other.min_frequency {
            self.min_frequency = min_frequency;
        }
        if let Some(max_occurrences) = other.max_occurrences {
            self.max_occurrences = max_occurrences;
        }
        if let Some(disabled) = other.disabled {
            self.disabled = disabled;
        }
        if let Some(tags) = other.tags {
            for tag in tags.into_iter() {
                if !self.tags.contains(&tag) {
                    self.tags.push(tag);
                }
            }
        }
    }
}

impl AddAssign for TaskConfig {
    fn add_assign(&mut self, other: Self) {
        self.merge(other);
    }
}

impl AddAssign<TaskBuilder> for TaskConfig {
    fn add_assign(&mut self, other: TaskBuilder) {
        self.update(other);
    }
}
