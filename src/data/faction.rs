//! Faction, independent settlement, and wilderness balance data.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactionGoal {
    Expand,
    Fortify,
    Raid,
    Trade,
    Influence,
    Recover,
    Confront,
    Appease,
}

impl FactionGoal {
    pub fn label(self) -> &'static str {
        match self {
            Self::Expand => "Expand",
            Self::Fortify => "Fortify",
            Self::Raid => "Raid",
            Self::Trade => "Trade",
            Self::Influence => "Influence",
            Self::Recover => "Recover",
            Self::Confront => "Confront",
            Self::Appease => "Appease",
        }
    }

    pub fn all() -> [Self; 8] {
        [
            Self::Expand,
            Self::Fortify,
            Self::Raid,
            Self::Trade,
            Self::Influence,
            Self::Recover,
            Self::Confront,
            Self::Appease,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionDef {
    pub id: String,
    pub name: String,
    pub personality: String,
    pub description: String,
    pub confidence: i32,
    pub fear: i32,
    pub hostility: i32,
    pub border_pressure: i32,
    pub controlled_locations: Vec<String>,
    pub memory_tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionBalance {
    pub rival: FactionDef,
    pub wilderness_base_pressure: i32,
    pub trail_warden_pressure_reduction: i32,
    pub fortification_pressure_reduction: i32,
    pub road_pressure_reduction: i32,
}

impl FactionBalance {
    pub fn validate(&self) -> Result<(), String> {
        if self.rival.id.is_empty() || self.rival.name.is_empty() {
            return Err("rival faction must have an id and name".to_owned());
        }
        if self.wilderness_base_pressure < 0 {
            return Err("wilderness base pressure cannot be negative".to_owned());
        }

        Ok(())
    }
}
