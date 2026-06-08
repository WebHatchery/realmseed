//! Road, route, and regional project balance data.

use super::ResourceStock;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteLevel {
    None,
    Path,
    Road,
    StoneRoad,
}

impl RouteLevel {
    pub fn from_author_level(level: u8) -> Self {
        match level {
            0 => Self::None,
            1 => Self::Path,
            2 => Self::Road,
            _ => Self::StoneRoad,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::None => "No road",
            Self::Path => "Path",
            Self::Road => "Road",
            Self::StoneRoad => "Stone Road",
        }
    }

    pub fn is_built(self) -> bool {
        self != Self::None
    }

    pub fn supply_cost(self) -> Option<i32> {
        match self {
            Self::None => None,
            Self::Path => Some(3),
            Self::Road => Some(2),
            Self::StoneRoad => Some(1),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadActionDef {
    pub kind: String,
    pub from: RouteLevel,
    pub to: RouteLevel,
    pub action_cost: i32,
    pub cost: ResourceStock,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionalProjectDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub action_cost: i32,
    pub cost: ResourceStock,
    pub unmanaged_strain_reduction: i32,
    pub danger_reduction: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadBalance {
    pub source_site_id: String,
    pub regional_project_id: String,
    pub road_event_issue_ids: Vec<String>,
    pub actions: Vec<RoadActionDef>,
    pub regional_projects: Vec<RegionalProjectDef>,
}

impl RoadBalance {
    pub fn action_for_level(&self, level: RouteLevel) -> Option<&RoadActionDef> {
        self.actions
            .iter()
            .find(|action| action.from == level && action.kind != "repair_route")
    }

    pub fn repair_action(&self) -> Option<&RoadActionDef> {
        self.actions
            .iter()
            .find(|action| action.kind == "repair_route")
    }

    pub fn regional_project(&self, id: &str) -> Option<&RegionalProjectDef> {
        self.regional_projects
            .iter()
            .find(|project| project.id == id)
    }

    pub fn validate(&self) -> Result<(), String> {
        for level in [RouteLevel::None, RouteLevel::Path, RouteLevel::Road] {
            if self.action_for_level(level).is_none() {
                return Err(format!("missing road action for {}", level.label()));
            }
        }
        if self.repair_action().is_none() {
            return Err("missing route repair action".to_owned());
        }
        if self.regional_project(&self.regional_project_id).is_none() {
            return Err("configured regional road project is not defined".to_owned());
        }
        if self.road_event_issue_ids.len() < 3 {
            return Err("road balance needs at least 3 road event ids".to_owned());
        }

        Ok(())
    }
}
