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
        if self.institutions.len() < 2 {
            return Err("campaign needs at least 2 institutions".to_owned());
        }

        Ok(())
    }
}
