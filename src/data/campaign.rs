//! Campaign-level balance data for ambitions, projects, institutions, and endings.

use super::ResourceStock;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignBalance {
    pub campaign_turns: u32,
    pub difficulty_presets: Vec<DifficultyPresetDef>,
    pub ambitions: Vec<AmbitionDef>,
    pub projects: Vec<ProjectDef>,
    pub institutions: Vec<InstitutionDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyPresetDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub event_weight_modifier: i32,
    pub wilderness_modifier: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbitionDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub identity_tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDef {
    pub id: String,
    pub name: String,
    pub scope: String,
    pub description: String,
    pub cost: ResourceStock,
    #[serde(default)]
    pub action_cost: i32,
    #[serde(default)]
    pub effects: ProjectEffectDef,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectEffectDef {
    #[serde(default)]
    pub resource_delta: ResourceStock,
    #[serde(default)]
    pub prosperity_delta: i32,
    #[serde(default)]
    pub stability_delta: i32,
    #[serde(default)]
    pub loyalty_delta: i32,
    #[serde(default)]
    pub danger_delta: i32,
    #[serde(default)]
    pub wilderness_pressure_delta: i32,
    #[serde(default)]
    pub memory_tags: Vec<String>,
}

impl ProjectEffectDef {
    pub fn has_any_effect(&self) -> bool {
        self.resource_delta.food != 0
            || self.resource_delta.timber != 0
            || self.resource_delta.stone != 0
            || self.resource_delta.wealth != 0
            || self.prosperity_delta != 0
            || self.stability_delta != 0
            || self.loyalty_delta != 0
            || self.danger_delta != 0
            || self.wilderness_pressure_delta != 0
            || !self.memory_tags.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstitutionDef {
    pub id: String,
    pub name: String,
    pub description: String,
}

impl CampaignBalance {
    pub fn difficulty(&self, id: &str) -> Option<&DifficultyPresetDef> {
        self.difficulty_presets
            .iter()
            .find(|difficulty| difficulty.id == id)
    }

    pub fn ambition(&self, id: &str) -> Option<&AmbitionDef> {
        self.ambitions.iter().find(|ambition| ambition.id == id)
    }

    pub fn project(&self, id: &str) -> Option<&ProjectDef> {
        self.projects.iter().find(|project| project.id == id)
    }

    pub fn institution(&self, id: &str) -> Option<&InstitutionDef> {
        self.institutions
            .iter()
            .find(|institution| institution.id == id)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.campaign_turns != 80 {
            return Err("prototype campaign must last 80 seasonal turns".to_owned());
        }
        if self.ambitions.len() < 3 {
            return Err("campaign needs at least 3 ambitions".to_owned());
        }
        if self.projects.len() < 3 {
            return Err("campaign needs at least 3 proactive projects".to_owned());
        }
        if !self
            .projects
            .iter()
            .any(|project| project.scope == "settlement")
            || !self
                .projects
                .iter()
                .any(|project| project.scope == "regional")
        {
            return Err("campaign needs settlement and regional projects".to_owned());
        }
        for project in &self.projects {
            if project.action_cost < 0 {
                return Err(format!(
                    "project `{}` has a negative action cost",
                    project.id
                ));
            }
            if !project.effects.has_any_effect() {
                return Err(format!(
                    "project `{}` has no configured effects",
                    project.id
                ));
            }
        }
        if self.institutions.len() < 2 {
            return Err("campaign needs at least 2 institutions".to_owned());
        }

        Ok(())
    }
}
