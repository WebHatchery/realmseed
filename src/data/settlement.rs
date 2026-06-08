//! Settlement balance data shared by runtime economy and UI.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SettlementTier {
    Camp,
    Village,
    Town,
    City,
}

impl SettlementTier {
    pub fn label(self) -> &'static str {
        match self {
            Self::Camp => "Camp",
            Self::Village => "Village",
            Self::Town => "Town",
            Self::City => "City",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ResourceStock {
    #[serde(default)]
    pub food: i32,
    #[serde(default)]
    pub timber: i32,
    #[serde(default)]
    pub stone: i32,
    #[serde(default)]
    pub wealth: i32,
}

impl ResourceStock {
    pub fn add(&mut self, other: Self) {
        self.food += other.food;
        self.timber += other.timber;
        self.stone += other.stone;
        self.wealth += other.wealth;
    }

    pub fn subtract(&mut self, other: Self) {
        self.food -= other.food;
        self.timber -= other.timber;
        self.stone -= other.stone;
        self.wealth -= other.wealth;
    }

    pub fn scaled(self, modifier: f32) -> Self {
        Self {
            food: (self.food as f32 * modifier).round() as i32,
            timber: (self.timber as f32 * modifier).round() as i32,
            stone: (self.stone as f32 * modifier).round() as i32,
            wealth: (self.wealth as f32 * modifier).round() as i32,
        }
    }

    pub fn deficit_text(self, cost: Self) -> Option<String> {
        let mut deficits = Vec::new();
        if self.food < cost.food {
            deficits.push(format!("{} food", cost.food - self.food));
        }
        if self.timber < cost.timber {
            deficits.push(format!("{} timber", cost.timber - self.timber));
        }
        if self.stone < cost.stone {
            deficits.push(format!("{} stone", cost.stone - self.stone));
        }
        if self.wealth < cost.wealth {
            deficits.push(format!("{} wealth", cost.wealth - self.wealth));
        }

        if deficits.is_empty() {
            None
        } else {
            Some(format!("Needs {}", deficits.join(", ")))
        }
    }

    pub fn cost_text(self) -> String {
        let mut parts = Vec::new();
        if self.food > 0 {
            parts.push(format!("{} food", self.food));
        }
        if self.timber > 0 {
            parts.push(format!("{} timber", self.timber));
        }
        if self.stone > 0 {
            parts.push(format!("{} stone", self.stone));
        }
        if self.wealth > 0 {
            parts.push(format!("{} wealth", self.wealth));
        }

        if parts.is_empty() {
            "no resources".to_owned()
        } else {
            parts.join(", ")
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementStartDef {
    pub tier: SettlementTier,
    pub population: i32,
    pub resources: ResourceStock,
    pub prosperity: i32,
    pub stability: i32,
    pub defence: i32,
    pub loyalty: i32,
    pub danger: i32,
    pub focus_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoundingBalanceDef {
    pub action_cost: i32,
    pub source_site_id: String,
    pub population_transfer: i32,
    pub min_source_population: i32,
    pub cost: ResourceStock,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamineBalanceDef {
    pub issue_id: String,
    pub stability_loss: i32,
    pub loyalty_loss: i32,
    pub prosperity_loss: i32,
    pub population_loss_percent: i32,
    pub collapse_after_seasons: i32,
    pub collapse_population_below: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierBalanceDef {
    pub tier: SettlementTier,
    pub production_modifier: f32,
    pub consumption_modifier: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementFocusDef {
    pub id: String,
    pub name: String,
    pub output: ResourceStock,
    pub prosperity_delta: i32,
    pub stability_delta: i32,
    pub loyalty_delta: i32,
    pub defence_delta: i32,
    pub danger_delta: i32,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementUpgradeDef {
    pub from: SettlementTier,
    pub to: SettlementTier,
    pub action_cost: i32,
    pub min_population: i32,
    pub min_prosperity: i32,
    pub min_stability: i32,
    pub cost: ResourceStock,
    pub requires_capital_network: bool,
    pub requires_stone_road_or_port: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementBalance {
    pub council_actions_per_season: i32,
    pub focus_change_action_cost: i32,
    pub starting_migrant_pool: i32,
    pub food_consumed_per_population: f32,
    pub starting_capital: SettlementStartDef,
    pub founded_camp: SettlementStartDef,
    pub founding: FoundingBalanceDef,
    pub famine: FamineBalanceDef,
    pub tiers: Vec<TierBalanceDef>,
    pub focuses: Vec<SettlementFocusDef>,
    pub upgrades: Vec<SettlementUpgradeDef>,
}

impl SettlementBalance {
    pub fn tier(&self, tier: SettlementTier) -> Option<&TierBalanceDef> {
        self.tiers.iter().find(|balance| balance.tier == tier)
    }

    pub fn focus(&self, focus_id: &str) -> Option<&SettlementFocusDef> {
        self.focuses.iter().find(|focus| focus.id == focus_id)
    }

    pub fn upgrade_from(&self, tier: SettlementTier) -> Option<&SettlementUpgradeDef> {
        self.upgrades.iter().find(|upgrade| upgrade.from == tier)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.council_actions_per_season < 1 {
            return Err("settlement balance must grant at least 1 council action".to_owned());
        }
        if self.focuses.len() < 3 {
            return Err("settlement balance needs at least 3 focuses".to_owned());
        }
        for tier in [
            SettlementTier::Camp,
            SettlementTier::Village,
            SettlementTier::Town,
            SettlementTier::City,
        ] {
            if self.tier(tier).is_none() {
                return Err(format!("missing tier balance for {}", tier.label()));
            }
        }
        if self.focus(&self.starting_capital.focus_id).is_none() {
            return Err("starting capital focus is not defined".to_owned());
        }
        if self.focus(&self.founded_camp.focus_id).is_none() {
            return Err("founded camp focus is not defined".to_owned());
        }
        if self.upgrade_from(SettlementTier::Camp).is_none()
            || self.upgrade_from(SettlementTier::Village).is_none()
            || self.upgrade_from(SettlementTier::Town).is_none()
        {
            return Err("camp, village, and town upgrade gates must be defined".to_owned());
        }

        Ok(())
    }
}
