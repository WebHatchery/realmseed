//! Settlement runtime state, player actions, and seasonal economy updates.

use super::{GameSession, RouteCondition, Season};
use crate::data::{
    GameData, ResourceStock, RouteLevel, SettlementFocusDef, SettlementStartDef, SettlementTier,
    SiteCategory,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum SettlementStatus {
    #[default]
    Active,
    Lost,
}

impl SettlementStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Active => "Active",
            Self::Lost => "Lost",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementRuntimeState {
    pub id: String,
    pub name: String,
    pub location_id: String,
    pub owner_faction: String,
    pub tier: SettlementTier,
    pub population: i32,
    pub stored: ResourceStock,
    pub prosperity: i32,
    pub stability: i32,
    pub defence: i32,
    pub loyalty: i32,
    pub danger: i32,
    pub focus_id: String,
    pub traits: Vec<String>,
    pub memory_tags: Vec<String>,
    pub active_issue_ids: Vec<String>,
    #[serde(default)]
    pub autonomy_pressure: i32,
    #[serde(default)]
    pub rival_pressure: i32,
    pub founded_year: u32,
    pub founded_season: Season,
    #[serde(default)]
    pub status: SettlementStatus,
    #[serde(default)]
    pub famine_seasons: i32,
}

impl SettlementRuntimeState {
    pub fn from_start(
        id: String,
        name: String,
        location_id: String,
        owner_faction: String,
        start: &SettlementStartDef,
        clock_year: u32,
        clock_season: Season,
    ) -> Self {
        Self {
            id,
            name,
            location_id,
            owner_faction,
            tier: start.tier,
            population: start.population,
            stored: start.resources,
            prosperity: clamp_stat(start.prosperity),
            stability: clamp_stat(start.stability),
            defence: clamp_stat(start.defence),
            loyalty: clamp_stat(start.loyalty),
            danger: clamp_stat(start.danger),
            focus_id: start.focus_id.clone(),
            traits: Vec::new(),
            memory_tags: Vec::new(),
            active_issue_ids: Vec::new(),
            autonomy_pressure: 0,
            rival_pressure: 0,
            founded_year: clock_year,
            founded_season: clock_season,
            status: SettlementStatus::Active,
            famine_seasons: 0,
        }
    }

    pub fn focus<'a>(&self, data: &'a GameData) -> Option<&'a SettlementFocusDef> {
        data.settlement_balance.focus(&self.focus_id)
    }

    pub fn is_active(&self) -> bool {
        self.status == SettlementStatus::Active
    }
}

#[derive(Debug, Clone)]
pub struct SettlementActionStatus {
    pub enabled: bool,
    pub reason: String,
}

impl SettlementActionStatus {
    pub fn enabled(reason: impl Into<String>) -> Self {
        Self {
            enabled: true,
            reason: reason.into(),
        }
    }

    pub fn disabled(reason: impl Into<String>) -> Self {
        Self {
            enabled: false,
            reason: reason.into(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SeasonAdvanceReport {
    pub food_shortages: usize,
    pub settlements_lost: usize,
    pub produced: ResourceStock,
    pub food_consumed: i32,
    pub population_delta: i32,
    pub isolated_settlements: usize,
    pub road_warnings: usize,
    pub unmanaged_strain: i32,
    pub events_triggered: usize,
    pub issues_escalated: usize,
    pub rival_actions: usize,
    pub independent_requests: usize,
    pub wilderness_changes: usize,
    pub campaign_finished: bool,
    pub first_food_shortage_site_id: Option<String>,
    pub first_lost_site_id: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct SettlementSeasonOutcome {
    started_famine: bool,
    food_shortage: bool,
    lost: bool,
    produced: ResourceStock,
    food_consumed: i32,
    population_delta: i32,
}

impl GameSession {
    pub fn create_starting_settlements(data: &GameData) -> Vec<SettlementRuntimeState> {
        let Some(capital) = data.site(&data.settlement_balance.founding.source_site_id) else {
            return Vec::new();
        };

        let mut settlement = SettlementRuntimeState::from_start(
            format!("settlement_{}", capital.id),
            capital.name.clone(),
            capital.id.clone(),
            "player".to_owned(),
            &data.settlement_balance.starting_capital,
            data.config.starting_year,
            Season::from_config(&data.config.starting_season),
        );
        settlement.traits = capital.traits.clone();
        settlement.memory_tags.push("charter_seat".to_owned());
        vec![settlement]
    }

    pub fn settlement_at_site(&self, site_id: &str) -> Option<&SettlementRuntimeState> {
        self.settlements
            .iter()
            .find(|settlement| settlement.location_id == site_id)
    }

    pub fn selected_settlement(&self) -> Option<&SettlementRuntimeState> {
        self.settlement_at_site(&self.selected_site_id)
    }

    pub fn founding_status(&self, data: &GameData) -> SettlementActionStatus {
        let Some(site) = self.selected_site(data) else {
            return SettlementActionStatus::disabled(data.text("state.settlement_select"));
        };

        if !self.is_known(&site.id) {
            return SettlementActionStatus::disabled(data.text("state.settlement_scout"));
        }
        if site.category != SiteCategory::Settlement {
            return SettlementActionStatus::disabled(data.text("state.settlement_category"));
        }
        if self.settlement_at_site(&site.id).is_some() {
            return SettlementActionStatus::disabled(data.text("state.settlement_exists"));
        }
        if site.owner.as_deref().is_some() {
            return SettlementActionStatus::disabled(data.text("state.settlement_claimed"));
        }

        let founding = &data.settlement_balance.founding;
        if self.council_actions_remaining < founding.action_cost {
            let actions = founding.action_cost.to_string();
            return SettlementActionStatus::disabled(
                data.text_with("state.needs_council_action", &[("{actions}", &actions)]),
            );
        }

        let Some(source) = self.active_settlement_at_site(&founding.source_site_id) else {
            return SettlementActionStatus::disabled(data.text("state.capital_unavailable"));
        };
        if let Some(reason) = source.stored.deficit_text_with(founding.cost, data) {
            return SettlementActionStatus::disabled(reason);
        }

        let source_can_send =
            source.population - founding.population_transfer >= founding.min_source_population;
        let migrants_can_send = self.migrant_pool >= founding.population_transfer;
        if !source_can_send && !migrants_can_send {
            let population = founding.population_transfer.to_string();
            return SettlementActionStatus::disabled(data.text_with(
                "state.settlement_settlers",
                &[("{population}", &population)],
            ));
        }

        let actions = founding.action_cost.to_string();
        let cost = founding.cost.cost_text_with(data);
        let population = founding.population_transfer.to_string();
        SettlementActionStatus::enabled(data.text_with(
            "state.settlement_found_cost",
            &[
                ("{actions}", &actions),
                ("{cost}", &cost),
                ("{population}", &population),
            ],
        ))
    }

    pub fn found_selected_camp(&mut self, data: &GameData) -> Result<String, String> {
        let status = self.founding_status(data);
        if !status.enabled {
            return Err(status.reason);
        }

        let site_id = self.selected_site_id.clone();
        let site = data
            .site(&site_id)
            .ok_or_else(|| data.text("state.settlement_missing"))?;
        let founding = &data.settlement_balance.founding;
        let source_index = self
            .settlements
            .iter()
            .position(|settlement| {
                settlement.location_id == founding.source_site_id && settlement.is_active()
            })
            .ok_or_else(|| data.text("state.capital_unavailable"))?;

        let source_can_send = self.settlements[source_index].population
            - founding.population_transfer
            >= founding.min_source_population;
        {
            let source = &mut self.settlements[source_index];
            source.stored.subtract(founding.cost);
            if source_can_send {
                source.population -= founding.population_transfer;
                add_unique_tag(&mut source.memory_tags, "sent_frontier_families");
            }
        }
        if !source_can_send {
            self.migrant_pool -= founding.population_transfer;
        }
        self.council_actions_remaining -= founding.action_cost;

        let mut settlement = SettlementRuntimeState::from_start(
            format!("settlement_{}", site.id),
            site.name.clone(),
            site.id.clone(),
            "player".to_owned(),
            &data.settlement_balance.founded_camp,
            self.clock.year,
            self.clock.season,
        );
        settlement.traits = site.traits.clone();
        add_unique_tag(&mut settlement.memory_tags, "frontier_founding");
        self.settlements.push(settlement);
        self.add_chronicle_entry(data, "settlement_founded", Some(&site_id));

        Ok(data.text_with("state.settlement_founded", &[("{site}", &site.name)]))
    }

    pub fn selected_upgrade_label(&self, data: &GameData) -> String {
        let Some(settlement) = self.selected_settlement() else {
            return data.text("state.upgrade_label");
        };
        data.settlement_balance
            .upgrade_from(settlement.tier)
            .map(|upgrade| {
                let tier = upgrade.to.label().to_owned();
                data.text_with("state.upgrade_to", &[("{tier}", &tier)])
            })
            .unwrap_or_else(|| data.text("state.upgrade_label"))
    }

    pub fn upgrade_status(&self, data: &GameData) -> SettlementActionStatus {
        let Some(settlement) = self.selected_settlement() else {
            return SettlementActionStatus::disabled(data.text("state.upgrade_selected"));
        };
        if !settlement.is_active() {
            return SettlementActionStatus::disabled(data.text("state.upgrade_lost"));
        }

        let Some(upgrade) = data.settlement_balance.upgrade_from(settlement.tier) else {
            return SettlementActionStatus::disabled(data.text("state.upgrade_ceiling"));
        };
        if self.council_actions_remaining < upgrade.action_cost {
            let actions = upgrade.action_cost.to_string();
            return SettlementActionStatus::disabled(
                data.text_with("state.needs_council_action", &[("{actions}", &actions)]),
            );
        }
        if settlement
            .active_issue_ids
            .contains(&data.settlement_balance.famine.issue_id)
        {
            return SettlementActionStatus::disabled(data.text("state.upgrade_famine"));
        }
        if settlement.population < upgrade.min_population {
            let minimum = upgrade.min_population.to_string();
            return SettlementActionStatus::disabled(
                data.text_with("state.upgrade_population", &[("{minimum}", &minimum)]),
            );
        }
        if settlement.prosperity < upgrade.min_prosperity {
            let minimum = upgrade.min_prosperity.to_string();
            return SettlementActionStatus::disabled(
                data.text_with("state.upgrade_prosperity", &[("{minimum}", &minimum)]),
            );
        }
        if settlement.stability < upgrade.min_stability {
            let minimum = upgrade.min_stability.to_string();
            return SettlementActionStatus::disabled(
                data.text_with("state.upgrade_stability", &[("{minimum}", &minimum)]),
            );
        }
        if let Some(reason) = settlement.stored.deficit_text_with(upgrade.cost, data) {
            return SettlementActionStatus::disabled(reason);
        }
        if upgrade.requires_capital_network
            && !self.has_capital_network_access(data, &settlement.location_id)
        {
            return SettlementActionStatus::disabled(data.text("state.upgrade_network"));
        }
        if upgrade.requires_stone_road_or_port
            && !self.has_stone_road_or_port_access(data, &settlement.location_id)
        {
            return SettlementActionStatus::disabled(data.text("state.upgrade_stone_network"));
        }

        let actions = upgrade.action_cost.to_string();
        let cost = upgrade.cost.cost_text_with(data);
        SettlementActionStatus::enabled(data.text_with(
            "state.upgrade_ready",
            &[("{actions}", &actions), ("{cost}", &cost)],
        ))
    }

    pub fn upgrade_selected_settlement(&mut self, data: &GameData) -> Result<String, String> {
        let status = self.upgrade_status(data);
        if !status.enabled {
            return Err(status.reason);
        }

        let site_id = self.selected_site_id.clone();
        let settlement_index = self
            .settlements
            .iter()
            .position(|settlement| settlement.location_id == site_id)
            .ok_or_else(|| data.text("state.upgrade_selected"))?;
        let upgrade = data
            .settlement_balance
            .upgrade_from(self.settlements[settlement_index].tier)
            .ok_or_else(|| data.text("state.upgrade_no_available"))?
            .clone();
        let settlement = &mut self.settlements[settlement_index];
        settlement.stored.subtract(upgrade.cost);
        settlement.tier = upgrade.to;
        settlement.prosperity = clamp_stat(settlement.prosperity + 5);
        settlement.stability = clamp_stat(settlement.stability + 4);
        settlement.defence = clamp_stat(settlement.defence + 5);
        add_unique_tag(
            &mut settlement.memory_tags,
            &format!("upgraded_to_{}", upgrade.to.label().to_lowercase()),
        );
        let settlement_name = settlement.name.clone();
        self.council_actions_remaining -= upgrade.action_cost;
        self.add_chronicle_entry(data, "settlement_upgraded", Some(&site_id));

        let tier = upgrade.to.label().to_owned();
        Ok(data.text_with(
            "state.upgrade_completed",
            &[("{settlement}", &settlement_name), ("{tier}", &tier)],
        ))
    }

    pub fn focus_change_status(&self, data: &GameData, focus_id: &str) -> SettlementActionStatus {
        let Some(settlement) = self.selected_settlement() else {
            return SettlementActionStatus::disabled(data.text("state.focus_selected"));
        };
        if !settlement.is_active() {
            return SettlementActionStatus::disabled(data.text("state.focus_lost"));
        }
        let Some(focus) = data.settlement_balance.focus(focus_id) else {
            return SettlementActionStatus::disabled(data.text("state.focus_unknown"));
        };
        if settlement.focus_id == focus.id {
            return SettlementActionStatus::disabled(data.text("state.focus_active"));
        }
        let action_cost = data.settlement_balance.focus_change_action_cost;
        if self.council_actions_remaining < action_cost {
            let actions = action_cost.to_string();
            return SettlementActionStatus::disabled(
                data.text_with("state.needs_council_action", &[("{actions}", &actions)]),
            );
        }

        if action_cost > 0 {
            let actions = action_cost.to_string();
            SettlementActionStatus::enabled(data.text_with(
                "state.focus_cost_action",
                &[("{actions}", &actions), ("{notes}", &focus.notes)],
            ))
        } else {
            SettlementActionStatus::enabled(focus.notes.clone())
        }
    }

    pub fn set_selected_settlement_focus(
        &mut self,
        data: &GameData,
        focus_id: &str,
    ) -> Result<String, String> {
        let status = self.focus_change_status(data, focus_id);
        if !status.enabled {
            return Err(status.reason);
        }
        let focus_name = data
            .settlement_balance
            .focus(focus_id)
            .map(|focus| focus.name.clone())
            .ok_or_else(|| data.text("state.focus_unknown"))?;
        let site_id = self.selected_site_id.clone();
        let action_cost = data.settlement_balance.focus_change_action_cost;
        let settlement = self
            .settlements
            .iter_mut()
            .find(|settlement| settlement.location_id == site_id)
            .ok_or_else(|| data.text("state.focus_selected"))?;
        settlement.focus_id = focus_id.to_owned();
        self.council_actions_remaining -= action_cost;

        Ok(data.text_with(
            "state.focus_changed",
            &[("{settlement}", &settlement.name), ("{focus}", &focus_name)],
        ))
    }

    pub fn advance_settlement_economy(&mut self, data: &GameData) -> SeasonAdvanceReport {
        let mut report = SeasonAdvanceReport::default();
        let mut chronicle_events: Vec<(&str, String)> = Vec::new();

        for settlement in &mut self.settlements {
            let outcome = apply_season_to_settlement(settlement, data);
            if outcome.food_shortage {
                report.food_shortages += 1;
                if report.first_food_shortage_site_id.is_none() {
                    report.first_food_shortage_site_id = Some(settlement.location_id.clone());
                }
            }
            if outcome.started_famine {
                chronicle_events.push(("settlement_starvation", settlement.location_id.clone()));
            }
            if outcome.lost {
                report.settlements_lost += 1;
                if report.first_lost_site_id.is_none() {
                    report.first_lost_site_id = Some(settlement.location_id.clone());
                }
                chronicle_events.push(("settlement_lost", settlement.location_id.clone()));
            }
            report.produced.add(outcome.produced);
            report.food_consumed += outcome.food_consumed;
            report.population_delta += outcome.population_delta;
        }

        self.council_actions_remaining = data.settlement_balance.council_actions_per_season;
        for (template_id, site_id) in chronicle_events {
            self.add_chronicle_entry(data, template_id, Some(&site_id));
        }

        report
    }

    fn active_settlement_at_site(&self, site_id: &str) -> Option<&SettlementRuntimeState> {
        self.settlements
            .iter()
            .find(|settlement| settlement.location_id == site_id && settlement.is_active())
    }

    fn has_capital_network_access(&self, data: &GameData, site_id: &str) -> bool {
        self.is_site_in_capital_network(data, site_id)
    }

    fn has_stone_road_or_port_access(&self, data: &GameData, site_id: &str) -> bool {
        let has_stone_road = self.routes.iter().any(|route| {
            route.connects(site_id)
                && route.level == RouteLevel::StoneRoad
                && route.condition != RouteCondition::Blocked
        });
        let has_port_trait = data
            .site(site_id)
            .map(|site| {
                site.traits
                    .iter()
                    .any(|site_trait| site_trait.to_lowercase().contains("port"))
            })
            .unwrap_or(false);

        has_stone_road || has_port_trait
    }
}

fn apply_season_to_settlement(
    settlement: &mut SettlementRuntimeState,
    data: &GameData,
) -> SettlementSeasonOutcome {
    if !settlement.is_active() {
        return SettlementSeasonOutcome::default();
    }

    let mut outcome = SettlementSeasonOutcome::default();
    let starting_population = settlement.population;
    let production = production_for(settlement, data);
    outcome.produced = production;
    settlement.stored.add(production);

    let consumption = food_consumption_for(settlement, data);
    outcome.food_consumed = consumption;
    settlement.stored.food -= consumption;
    let had_shortage = settlement.stored.food < 0;
    if had_shortage {
        settlement.stored.food = 0;
        outcome.food_shortage = true;
        outcome.started_famine = settlement.famine_seasons == 0;
        settlement.famine_seasons += 1;
        add_unique_tag(&mut settlement.memory_tags, "hungry");
        add_unique_issue(
            &mut settlement.active_issue_ids,
            &data.settlement_balance.famine.issue_id,
        );
        apply_famine(settlement, data);
    } else {
        recover_from_food_security(settlement, data);
    }

    apply_focus_drift(settlement, data);
    if settlement.famine_seasons == 0 && settlement.stability > 65 {
        settlement.loyalty = clamp_stat(settlement.loyalty + 2);
    }

    if should_lose_to_famine(settlement, data) {
        settlement.status = SettlementStatus::Lost;
        settlement.population = 0;
        settlement.stored = ResourceStock::default();
        settlement.active_issue_ids.clear();
        add_unique_issue(&mut settlement.active_issue_ids, "abandoned");
        add_unique_tag(&mut settlement.memory_tags, "lost_to_famine");
        outcome.lost = true;
    }

    outcome.population_delta = settlement.population - starting_population;
    outcome
}

fn production_for(settlement: &SettlementRuntimeState, data: &GameData) -> ResourceStock {
    let Some(focus) = settlement.focus(data) else {
        return ResourceStock::default();
    };
    let tier_modifier = data
        .settlement_balance
        .tier(settlement.tier)
        .map(|tier| tier.production_modifier)
        .unwrap_or(1.0);
    let mut output = focus.output.scaled(tier_modifier);

    if let Some(site) = data.site(&settlement.location_id) {
        if let Some(terrain) = data.terrain_at(site.position.x, site.position.y) {
            match focus.id.as_str() {
                "farming" => output.food += ((terrain.fertility - 50) / 5).clamp(-8, 12),
                "logging" => output.timber += ((terrain.timber - 50) / 5).clamp(-8, 12),
                "quarrying" => output.stone += ((terrain.stone - 50) / 5).clamp(-8, 12),
                "trade"
                    if data
                        .roads_for_site(&settlement.location_id)
                        .any(|road| road.level > 0) =>
                {
                    output.wealth += 8;
                }
                _ => {}
            }
        }
    }

    output.food = output.food.max(0);
    output.timber = output.timber.max(0);
    output.stone = output.stone.max(0);
    output.wealth = output.wealth.max(0);
    output
}

fn food_consumption_for(settlement: &SettlementRuntimeState, data: &GameData) -> i32 {
    let consumption_modifier = data
        .settlement_balance
        .tier(settlement.tier)
        .map(|tier| tier.consumption_modifier)
        .unwrap_or(1.0);

    (settlement.population as f32
        * data.settlement_balance.food_consumed_per_population
        * consumption_modifier)
        .round()
        .max(1.0) as i32
}

fn apply_famine(settlement: &mut SettlementRuntimeState, data: &GameData) {
    let famine = &data.settlement_balance.famine;
    settlement.stability = clamp_stat(settlement.stability - famine.stability_loss);
    settlement.loyalty = clamp_stat(settlement.loyalty - famine.loyalty_loss);
    settlement.prosperity = clamp_stat(settlement.prosperity - famine.prosperity_loss);
    let population_loss_percent = famine.population_loss_percent + settlement.famine_seasons - 1;
    let population_loss = ((settlement.population * population_loss_percent) / 100).max(1);
    settlement.population = (settlement.population - population_loss).max(0);
}

fn recover_from_food_security(settlement: &mut SettlementRuntimeState, data: &GameData) {
    if settlement.famine_seasons > 0 {
        settlement.famine_seasons -= 1;
        if settlement.famine_seasons == 0 {
            remove_issue(
                &mut settlement.active_issue_ids,
                &data.settlement_balance.famine.issue_id,
            );
        }
    }
    if settlement.stability > 50 {
        let natural_growth = ((settlement.population as f32) * 0.005).round() as i32;
        settlement.population += natural_growth.max(1);
    }
    if settlement.prosperity > 65 {
        settlement.population += ((settlement.prosperity - 65) / 5).clamp(2, 8);
    }
    if settlement.stability > 70 && settlement.danger < 35 {
        settlement.population += ((settlement.stability - 70) / 8).clamp(1, 5);
    }
    settlement.stability = clamp_stat(settlement.stability + 1);
}

fn apply_focus_drift(settlement: &mut SettlementRuntimeState, data: &GameData) {
    let Some(focus) = settlement.focus(data) else {
        return;
    };

    settlement.prosperity = clamp_stat(settlement.prosperity + focus.prosperity_delta);
    settlement.stability = clamp_stat(settlement.stability + focus.stability_delta);
    settlement.loyalty = clamp_stat(settlement.loyalty + focus.loyalty_delta);
    settlement.defence = clamp_stat(settlement.defence + focus.defence_delta);
    settlement.danger = clamp_stat(settlement.danger + focus.danger_delta);
}

fn should_lose_to_famine(settlement: &SettlementRuntimeState, data: &GameData) -> bool {
    let famine = &data.settlement_balance.famine;
    settlement.population < famine.collapse_population_below
        || (settlement.famine_seasons >= famine.collapse_after_seasons
            && (settlement.stability <= 20 || settlement.loyalty <= 15))
}

fn add_unique_tag(tags: &mut Vec<String>, tag: &str) {
    if !tags.iter().any(|existing| existing == tag) {
        tags.push(tag.to_owned());
    }
}

fn add_unique_issue(issue_ids: &mut Vec<String>, issue_id: &str) {
    if !issue_ids.iter().any(|existing| existing == issue_id) {
        issue_ids.push(issue_id.to_owned());
    }
}

fn remove_issue(issue_ids: &mut Vec<String>, issue_id: &str) {
    issue_ids.retain(|existing| existing != issue_id);
}

fn clamp_stat(value: i32) -> i32 {
    value.clamp(0, 100)
}
