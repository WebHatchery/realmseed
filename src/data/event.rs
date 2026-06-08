//! Event family and template data used by active issue chains.

use super::ResourceStock;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventStage {
    Opening,
    FollowUp,
    Resolution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActiveIssueState {
    Warning,
    Active,
    Escalating,
    Resolution,
    Dormant,
    Collapse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFamilyDef {
    pub id: String,
    pub family_type: String,
    pub trigger_kind: String,
    pub opening_template_id: String,
    pub followup_template_id: String,
    pub resolution_template_id: String,
    pub severity_levels: Vec<i32>,
    pub local_cooldown: u32,
    pub global_cooldown: u32,
    pub max_per_target: u32,
    pub memory_tags: Vec<String>,
    pub opening_chronicle_template_id: String,
    pub resolution_chronicle_template_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTemplateDef {
    pub id: String,
    pub family_id: String,
    pub stage: EventStage,
    pub title: String,
    pub narrative: String,
    pub cause: String,
    pub visible_consequences: String,
    pub hidden_consequences: String,
    pub allow_defer: bool,
    pub choices: Vec<EventChoiceDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventChoiceDef {
    pub id: String,
    pub label: String,
    pub visible_consequence: String,
    #[serde(default)]
    pub requirements: EventChoiceRequirements,
    #[serde(default)]
    pub blocked_by_tags: Vec<String>,
    pub effects: EventChoiceEffects,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventChoiceRequirements {
    #[serde(default)]
    pub resources: ResourceStock,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventChoiceEffects {
    #[serde(default)]
    pub resource_delta: ResourceStock,
    #[serde(default)]
    pub population_delta: i32,
    #[serde(default)]
    pub stability_delta: i32,
    #[serde(default)]
    pub loyalty_delta: i32,
    #[serde(default)]
    pub prosperity_delta: i32,
    #[serde(default)]
    pub danger_delta: i32,
    #[serde(default)]
    pub memory_tags: Vec<String>,
    pub issue_state: Option<ActiveIssueState>,
    #[serde(default)]
    pub severity_delta: i32,
    #[serde(default)]
    pub response_score: i32,
    #[serde(default)]
    pub close_issue: bool,
    pub chronicle_template_id: Option<String>,
}
