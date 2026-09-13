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
    pub wilderness_seasonal_delta: i32,
    pub trail_warden_pressure_reduction: i32,
    pub fortification_pressure_reduction: i32,
    pub road_pressure_reduction: i32,
    pub rival_actions: RivalActionBalanceDef,
    pub independent_interactions: IndependentInteractionBalanceDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RivalActionBalanceDef {
    pub goal_max_age: u32,
    pub action_cooldown: u32,
    pub repeat_goal_bonus: i32,
    pub opportunist_bonus: i32,
    pub expand_border_pressure_delta: i32,
    pub influence_rival_pressure_delta: i32,
    pub influence_loyalty_delta: i32,
    pub trade_rival_pressure_delta: i32,
    pub trade_trust_delta: i32,
    pub fortify_confidence_delta: i32,
    pub recover_losses_delta: i32,
    pub recover_confidence_delta: i32,
    pub confront_hostility_delta: i32,
    pub appease_hostility_delta: i32,
    pub raid: RivalRaidBalanceDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RivalRaidBalanceDef {
    pub connected_defence_bonus: i32,
    pub isolated_defence_penalty: i32,
    pub hostility_divisor: i32,
    pub stability_divisor: i32,
    pub success_margin: i32,
    pub food_loss: i32,
    pub wealth_loss: i32,
    pub stability_loss: i32,
    pub loyalty_loss: i32,
    pub rival_pressure_delta: i32,
    pub hostility_delta: i32,
    pub resistance_defence_delta: i32,
    pub resistance_recent_losses_delta: i32,
    pub resistance_fear_delta: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndependentInteractionBalanceDef {
    pub starting_trust: i32,
    pub starting_autonomy: i32,
    pub starting_rival_pressure: i32,
    pub seasonal_pressure_delta: i32,
    pub seasonal_trade_trust_delta: i32,
    pub trade_min_trust: i32,
    pub trade_trust_delta: i32,
    pub integration_min_trust: i32,
    pub integration_max_autonomy: i32,
    pub integration_max_rival_pressure: i32,
    pub integration_progress_delta: i32,
    pub integration_autonomy_delta: i32,
    pub request_pressure_threshold: i32,
    pub request_autonomy_delta: i32,
    pub protection_pressure_threshold: i32,
    pub resistance_autonomy_threshold: i32,
    pub resistance_pressure_threshold: i32,
    pub resistance_progress_delta: i32,
    pub defection_pressure_threshold: i32,
    pub defection_trust_threshold: i32,
}

impl FactionBalance {
    pub fn validate(&self) -> Result<(), String> {
        if self.rival.id.is_empty() || self.rival.name.is_empty() {
            return Err("rival faction must have an id and name".to_owned());
        }
        if self.wilderness_base_pressure < 0 {
            return Err("wilderness base pressure cannot be negative".to_owned());
        }
        if self.wilderness_seasonal_delta < 0 {
            return Err("wilderness seasonal pressure delta cannot be negative".to_owned());
        }
        if self
            .rival
            .controlled_locations
            .iter()
            .any(|location| location.is_empty())
        {
            return Err("rival controlled locations cannot be empty".to_owned());
        }
        if self.rival_actions.goal_max_age == 0 || self.rival_actions.action_cooldown == 0 {
            return Err("rival action timing must be positive".to_owned());
        }
        if self.rival_actions.raid.hostility_divisor <= 0
            || self.rival_actions.raid.stability_divisor <= 0
            || self.rival_actions.raid.success_margin < 0
        {
            return Err("rival raid divisors and margin are invalid".to_owned());
        }
        let independent = &self.independent_interactions;
        if independent.starting_trust < 0
            || independent.starting_trust > 100
            || independent.starting_autonomy < 0
            || independent.starting_autonomy > 100
            || independent.starting_rival_pressure < 0
            || independent.starting_rival_pressure > 100
            || independent.integration_progress_delta <= 0
            || independent.integration_max_autonomy > 100
            || independent.integration_max_rival_pressure > 100
        {
            return Err("independent interaction thresholds are invalid".to_owned());
        }

        Ok(())
    }
}
