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
    pub scoring: CampaignScoringDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignScoringDef {
    pub controlled_settlement_points: i32,
    pub town_points: i32,
    pub population_divisor: i32,
    pub connected_road_points: i32,
    pub integrated_settlement_points: i32,
    pub high_civic_bonus: i32,
    pub lost_settlement_penalty: i32,
    pub high_civic_threshold: i32,
    pub ambition_identity_threshold: i32,
    pub identity_road_minimum: i32,
    pub identity_wealth_minimum: i32,
    pub identity_food_minimum: i32,
    pub identity_civic_minimum: i32,
    pub ending_bands: Vec<EndingBandDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndingBandDef {
    pub id: String,
    pub label: String,
    pub minimum_score: i32,
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

        if self.scoring.population_divisor <= 0
            || self.scoring.high_civic_threshold < 0
            || self.scoring.ambition_identity_threshold < 0
            || self.scoring.identity_road_minimum < 0
            || self.scoring.identity_wealth_minimum < 0
            || self.scoring.identity_food_minimum < 0
            || self.scoring.identity_civic_minimum < 0
        {
            return Err("campaign scoring thresholds must be non-negative and usable".to_owned());
        }
        if self.scoring.ending_bands.is_empty() {
            return Err("campaign needs at least one ending band".to_owned());
        }
        for window in self.scoring.ending_bands.windows(2) {
            if window[0].minimum_score >= window[1].minimum_score {
                return Err("ending bands must be ordered by increasing score".to_owned());
            }
        }
        if self
            .scoring
            .ending_bands
            .iter()
            .any(|band| band.id.is_empty() || band.label.is_empty())
        {
            return Err("ending bands must have ids and labels".to_owned());
        }

        Ok(())
    }
}
