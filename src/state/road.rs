//! Runtime roads, supply graph utilities, isolation pressure, and regional projects.

use super::{
    GameSession, Season, SettlementActionStatus, SettlementRuntimeState, SettlementStatus,
};
use crate::data::{GameData, ResourceStock, RoadActionDef, RoadDef, RouteLevel};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteCondition {
    Clear,
    Damaged,
    Blocked,
}

impl Default for RouteCondition {
    fn default() -> Self {
        Self::Clear
    }
}

impl RouteCondition {
    pub fn label(self) -> &'static str {
        match self {
            Self::Clear => "Clear",
            Self::Damaged => "Damaged",
            Self::Blocked => "Blocked",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRuntimeState {
    pub id: String,
    pub site_a: String,
    pub site_b: String,
    pub level: RouteLevel,
    pub known: bool,
    pub condition: RouteCondition,
    pub region_ids: Vec<String>,
    pub active_warning_ids: Vec<String>,
}

impl RouteRuntimeState {
    pub fn from_def(data: &GameData, road: &RoadDef) -> Self {
        let mut region_ids = Vec::new();
        for site_id in [&road.from, &road.to] {
            if let Some(site) = data.site(site_id) {
                if !region_ids.contains(&site.region_id) {
                    region_ids.push(site.region_id.clone());
                }
            }
        }
        let known = endpoints_initially_visible(data, &road.from, &road.to);

        Self {
            id: road.id.clone(),
            site_a: road.from.clone(),
            site_b: road.to.clone(),
            level: RouteLevel::from_author_level(road.level),
            known,
            condition: RouteCondition::Clear,
            region_ids,
            active_warning_ids: Vec::new(),
        }
    }

    pub fn connects(&self, site_id: &str) -> bool {
        self.site_a == site_id || self.site_b == site_id
    }

    pub fn other_end<'a>(&'a self, site_id: &str) -> Option<&'a str> {
        if self.site_a == site_id {
            Some(&self.site_b)
        } else if self.site_b == site_id {
            Some(&self.site_a)
        } else {
            None
        }
    }

    pub fn is_passable(&self) -> bool {
        self.level.is_built() && self.condition != RouteCondition::Blocked
    }

    pub fn supply_cost(&self) -> Option<i32> {
        let base = self.level.supply_cost()?;
        Some(match self.condition {
            RouteCondition::Clear => base,
            RouteCondition::Damaged => base + 2,
            RouteCondition::Blocked => return None,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionalProjectRuntimeState {
    pub id: String,
    pub region_id: String,
    pub completed_year: u32,
    pub completed_season: Season,
}

#[derive(Debug, Clone, Default)]
pub struct RoadAdvanceReport {
    pub isolated_settlements: usize,
    pub road_warnings: usize,
    pub unmanaged_strain: i32,
}

impl GameSession {
    pub fn create_starting_routes(data: &GameData) -> Vec<RouteRuntimeState> {
        data.roads
            .iter()
            .map(|road| RouteRuntimeState::from_def(data, road))
            .collect()
    }

    pub fn routes_for_site<'a>(
        &'a self,
        site_id: &'a str,
    ) -> impl Iterator<Item = &'a RouteRuntimeState> {
        self.routes
            .iter()
            .filter(move |route| route.connects(site_id))
    }

    pub fn route_action_label(&self, data: &GameData, route_id: &str) -> String {
        let Some(route) = self.routes.iter().find(|route| route.id == route_id) else {
            return "Route Action".to_owned();
        };
        if route.condition != RouteCondition::Clear {
            return "Repair Route".to_owned();
        }
        data.road_balance
            .action_for_level(route.level)
            .map(|action| match action.to {
                RouteLevel::Path => "Build Path".to_owned(),
                RouteLevel::Road => "Upgrade to Road".to_owned(),
                RouteLevel::StoneRoad => "Upgrade to Stone Road".to_owned(),
                RouteLevel::None => "Route Action".to_owned(),
            })
            .unwrap_or_else(|| "Route Complete".to_owned())
    }

    pub fn route_action_status(&self, data: &GameData, route_id: &str) -> SettlementActionStatus {
        let Some(route) = self.routes.iter().find(|route| route.id == route_id) else {
            return SettlementActionStatus::disabled("Unknown route.");
        };
        if !route.known {
            return SettlementActionStatus::disabled("Both endpoints must be known.");
        }
        if !self.route_has_controlled_endpoint(route) {
            return SettlementActionStatus::disabled(
                "Settle one endpoint before funding this route.",
            );
        }

        let Some(action) = self.route_action_for(data, route) else {
            return SettlementActionStatus::disabled("This route is already fully upgraded.");
        };
        if self.council_actions_remaining < action.action_cost {
            return SettlementActionStatus::disabled(format!(
                "Needs {} council action.",
                action.action_cost
            ));
        }
        let cost = adjusted_route_cost(data, route, action.cost);
        let Some(source) = self.road_source_settlement(data) else {
            return SettlementActionStatus::disabled("The charter capital is unavailable.");
        };
        if let Some(reason) = source.stored.deficit_text(cost) {
            return SettlementActionStatus::disabled(reason);
        }

        SettlementActionStatus::enabled(format!(
            "Costs {} action and {} from Charter Hall.",
            action.action_cost,
            cost.cost_text()
        ))
    }

    pub fn build_or_upgrade_route(
        &mut self,
        data: &GameData,
        route_id: &str,
    ) -> Result<String, String> {
        let status = self.route_action_status(data, route_id);
        if !status.enabled {
            return Err(status.reason);
        }

        let route_index = self
            .routes
            .iter()
            .position(|route| route.id == route_id)
            .ok_or_else(|| "Unknown route.".to_owned())?;
        let action = self
            .route_action_for(data, &self.routes[route_index])
            .ok_or_else(|| "This route is already fully upgraded.".to_owned())?
            .clone();
        let cost = adjusted_route_cost(data, &self.routes[route_index], action.cost);
        let source_index = self
            .settlements
            .iter()
            .position(|settlement| {
                settlement.location_id == data.road_balance.source_site_id
                    && settlement.status == SettlementStatus::Active
            })
            .ok_or_else(|| "The charter capital is unavailable.".to_owned())?;
        self.settlements[source_index].stored.subtract(cost);
        self.council_actions_remaining -= action.action_cost;

        let mut chronicle_event: Option<(&str, String)> = None;
        let message = {
            let route = &mut self.routes[route_index];
            let old_level = route.level;
            if route.condition != RouteCondition::Clear {
                route.condition = RouteCondition::Clear;
                route.active_warning_ids.clear();
                format!("Repaired route {}", route.id)
            } else {
                route.level = action.to;
                let template_id = if old_level == RouteLevel::None {
                    "road_built"
                } else {
                    "road_upgraded"
                };
                chronicle_event = Some((template_id, route.site_a.clone()));
                format!(
                    "{} now has {}",
                    route_label(data, route),
                    route.level.label()
                )
            }
        };

        if let Some((template_id, site_id)) = chronicle_event {
            self.add_chronicle_entry(data, template_id, Some(&site_id));
        }

        Ok(message)
    }

    pub fn regional_project_status(
        &self,
        data: &GameData,
        region_id: &str,
    ) -> SettlementActionStatus {
        if self.has_regional_project(region_id) {
            return SettlementActionStatus::disabled("Trail Wardens already patrol this region.");
        }
        let Some(project) = data
            .road_balance
            .regional_project(&data.road_balance.regional_project_id)
        else {
            return SettlementActionStatus::disabled("Regional project data is missing.");
        };
        if self.council_actions_remaining < project.action_cost {
            return SettlementActionStatus::disabled(format!(
                "Needs {} council action.",
                project.action_cost
            ));
        }
        let Some(source) = self.road_source_settlement(data) else {
            return SettlementActionStatus::disabled("The charter capital is unavailable.");
        };
        if let Some(reason) = source.stored.deficit_text(project.cost) {
            return SettlementActionStatus::disabled(reason);
        }

        SettlementActionStatus::enabled(format!(
            "Costs {} action and {}. {}",
            project.action_cost,
            project.cost.cost_text(),
            project.description
        ))
    }

    pub fn complete_regional_project(
        &mut self,
        data: &GameData,
        region_id: &str,
    ) -> Result<String, String> {
        let status = self.regional_project_status(data, region_id);
        if !status.enabled {
            return Err(status.reason);
        }
        let project = data
            .road_balance
            .regional_project(&data.road_balance.regional_project_id)
            .ok_or_else(|| "Regional project data is missing.".to_owned())?
            .clone();
        let source_index = self
            .settlements
            .iter()
            .position(|settlement| {
                settlement.location_id == data.road_balance.source_site_id
                    && settlement.status == SettlementStatus::Active
            })
            .ok_or_else(|| "The charter capital is unavailable.".to_owned())?;
        self.settlements[source_index].stored.subtract(project.cost);
        self.council_actions_remaining -= project.action_cost;
        self.regional_projects.push(RegionalProjectRuntimeState {
            id: project.id.clone(),
            region_id: region_id.to_owned(),
            completed_year: self.clock.year,
            completed_season: self.clock.season,
        });
        for settlement in &mut self.settlements {
            let Some(site) = data.site(&settlement.location_id) else {
                continue;
            };
            if site.region_id == region_id {
                settlement.danger = (settlement.danger - project.danger_reduction).clamp(0, 100);
            }
        }
        let chronicle_site_id = data
            .site(&self.selected_site_id)
            .filter(|site| site.region_id == region_id)
            .map(|site| site.id.clone())
            .or_else(|| {
                data.sites
                    .iter()
                    .find(|site| site.region_id == region_id && self.is_known(&site.id))
                    .map(|site| site.id.clone())
            });
        self.add_chronicle_entry(
            data,
            "regional_project_completed",
            chronicle_site_id.as_deref(),
        );

        Ok(format!(
            "Completed {} in {}",
            project.name,
            region_label(data, region_id)
        ))
    }

    pub fn settlement_supply_summary(&self, data: &GameData, site_id: &str) -> String {
        if site_id == data.road_balance.source_site_id {
            return "Capital network root.".to_owned();
        }
        match self.supply_distance(data, site_id) {
            Some(distance) => format!("Connected to capital. Supply distance {}.", distance),
            None => "Isolated: no built, unblocked route reaches Charter Hall.".to_owned(),
        }
    }

    pub fn is_site_in_capital_network(&self, data: &GameData, site_id: &str) -> bool {
        self.supply_distance(data, site_id).is_some()
    }

    pub fn supply_distance(&self, data: &GameData, site_id: &str) -> Option<i32> {
        self.capital_distances(data).get(site_id).copied()
    }

    pub fn advance_road_and_supply(&mut self, data: &GameData) -> RoadAdvanceReport {
        self.refresh_route_knowledge();
        let road_warnings = self.apply_route_warnings(data);
        let distances = self.capital_distances(data);
        let disconnected_sites: HashSet<String> = self
            .settlements
            .iter()
            .filter(|settlement| {
                settlement.status == SettlementStatus::Active
                    && settlement.owner_faction == "player"
                    && settlement.location_id != data.road_balance.source_site_id
                    && !distances.contains_key(&settlement.location_id)
            })
            .map(|settlement| settlement.location_id.clone())
            .collect();

        let unmanaged_strain = self.calculate_unmanaged_strain(data, disconnected_sites.len());
        self.unmanaged_strain = unmanaged_strain;
        self.unmanaged_strain_seasons = if unmanaged_strain >= 5 {
            self.unmanaged_strain_seasons + 1
        } else {
            0
        };

        let mut isolation_events = Vec::new();
        for settlement in &mut self.settlements {
            if settlement.status != SettlementStatus::Active || settlement.owner_faction != "player"
            {
                continue;
            }
            if settlement.location_id == data.road_balance.source_site_id {
                continue;
            }

            if disconnected_sites.contains(&settlement.location_id) {
                apply_isolation_penalty(settlement, unmanaged_strain);
                if !settlement
                    .memory_tags
                    .iter()
                    .any(|tag| tag == "isolation_recorded")
                {
                    settlement.memory_tags.push("isolation_recorded".to_owned());
                    isolation_events.push(settlement.location_id.clone());
                }
            } else {
                apply_connection_bonus(settlement, &distances);
            }
        }
        for site_id in isolation_events {
            self.add_chronicle_entry(data, "settlement_isolated", Some(&site_id));
        }
        if self.unmanaged_strain_seasons == 3 {
            self.add_chronicle_entry(data, "administrative_backlog", None);
        }

        RoadAdvanceReport {
            isolated_settlements: disconnected_sites.len(),
            road_warnings,
            unmanaged_strain,
        }
    }

    pub fn refresh_route_knowledge(&mut self) {
        let known_sites: HashSet<&str> = self
            .site_states
            .iter()
            .filter(|state| state.knowledge == super::SiteKnowledge::Known)
            .map(|state| state.site_id.as_str())
            .collect();
        for route in &mut self.routes {
            route.known = known_sites.contains(route.site_a.as_str())
                && known_sites.contains(route.site_b.as_str());
        }
    }

    fn route_action_for<'a>(
        &self,
        data: &'a GameData,
        route: &RouteRuntimeState,
    ) -> Option<&'a RoadActionDef> {
        if route.condition != RouteCondition::Clear {
            return data.road_balance.repair_action();
        }
        data.road_balance.action_for_level(route.level)
    }

    fn route_has_controlled_endpoint(&self, route: &RouteRuntimeState) -> bool {
        self.settlements.iter().any(|settlement| {
            settlement.status == SettlementStatus::Active
                && settlement.owner_faction == "player"
                && (settlement.location_id == route.site_a
                    || settlement.location_id == route.site_b)
        })
    }

    fn road_source_settlement(&self, data: &GameData) -> Option<&SettlementRuntimeState> {
        self.settlements.iter().find(|settlement| {
            settlement.location_id == data.road_balance.source_site_id
                && settlement.status == SettlementStatus::Active
        })
    }

    fn capital_distances(&self, data: &GameData) -> HashMap<String, i32> {
        let capital_site_id = data.road_balance.source_site_id.as_str();
        let mut distances = HashMap::from([(capital_site_id.to_owned(), 0)]);
        let mut changed = true;

        while changed {
            changed = false;
            for route in &self.routes {
                if !route.known || !route.is_passable() {
                    continue;
                }
                let Some(cost) = route.supply_cost() else {
                    continue;
                };
                changed |= relax_distance(&mut distances, &route.site_a, &route.site_b, cost);
                changed |= relax_distance(&mut distances, &route.site_b, &route.site_a, cost);
            }
        }

        distances
    }

    fn apply_route_warnings(&mut self, data: &GameData) -> usize {
        let completed_regions: HashSet<&str> = self
            .regional_projects
            .iter()
            .map(|project| project.region_id.as_str())
            .collect();
        let high_danger_sites: HashSet<&str> = self
            .settlements
            .iter()
            .filter(|settlement| {
                settlement.status == SettlementStatus::Active && settlement.danger >= 55
            })
            .map(|settlement| settlement.location_id.as_str())
            .collect();
        let mut warning_events = Vec::new();

        for route in &mut self.routes {
            if !route.known
                || !route.level.is_built()
                || route.condition != RouteCondition::Clear
                || route
                    .region_ids
                    .iter()
                    .any(|region_id| completed_regions.contains(region_id.as_str()))
            {
                continue;
            }

            let Some(road) = data.roads.iter().find(|road| road.id == route.id) else {
                continue;
            };
            let warning_id = route_warning_for(
                route,
                road,
                self.clock.season,
                self.clock.turn,
                &high_danger_sites,
            );
            if let Some(warning_id) = warning_id {
                apply_warning_to_route(route, warning_id);
                warning_events.push((warning_id, route.site_a.clone()));
            }
        }

        for (warning_id, site_id) in &warning_events {
            let template_id = match *warning_id {
                "bridge_washout" => "road_bridge_washout",
                "caravan_attacked" => "road_caravan_attacked",
                "winter_blockage" => "road_winter_blockage",
                _ => "road_winter_blockage",
            };
            self.add_chronicle_entry(data, template_id, Some(site_id));
        }

        warning_events.len()
    }

    fn calculate_unmanaged_strain(&self, data: &GameData, disconnected_count: usize) -> i32 {
        let controlled_settlements = self
            .settlements
            .iter()
            .filter(|settlement| {
                settlement.status == SettlementStatus::Active
                    && settlement.owner_faction == "player"
            })
            .count() as i32;
        let active_crises = self
            .settlements
            .iter()
            .map(|settlement| settlement.active_issue_ids.len() as i32)
            .sum::<i32>();
        let project_reduction = self
            .regional_projects
            .iter()
            .filter_map(|project| data.road_balance.regional_project(&project.id))
            .map(|project| project.unmanaged_strain_reduction)
            .sum::<i32>();
        let strain = controlled_settlements + active_crises + disconnected_count as i32;
        (strain - council_capacity(self) - project_reduction).max(0)
    }

    fn has_regional_project(&self, region_id: &str) -> bool {
        self.regional_projects
            .iter()
            .any(|project| project.region_id == region_id)
    }
}

fn endpoints_initially_visible(data: &GameData, site_a: &str, site_b: &str) -> bool {
    let known_a = data
        .site(site_a)
        .map(|site| site.initially_visible)
        .unwrap_or(false);
    let known_b = data
        .site(site_b)
        .map(|site| site.initially_visible)
        .unwrap_or(false);
    known_a && known_b
}

fn adjusted_route_cost(
    data: &GameData,
    route: &RouteRuntimeState,
    base_cost: ResourceStock,
) -> ResourceStock {
    let Some(road) = data.roads.iter().find(|road| road.id == route.id) else {
        return base_cost;
    };
    let mut percent = 100;
    if road.route_type.contains("pass") || road.route_type.contains("causeway") {
        percent += 35;
    } else if road.route_type.contains("ford") || road.route_type.contains("bridge") {
        percent += 25;
    }
    for site_id in [&route.site_a, &route.site_b] {
        let Some(site) = data.site(site_id) else {
            continue;
        };
        if let Some(terrain) = data.terrain_at(site.position.x, site.position.y) {
            if terrain.id == "forest" || terrain.id == "marsh" {
                percent += 10;
            } else if terrain.id == "hills" || terrain.id == "mountain" {
                percent += 20;
            }
        }
        if site
            .traits
            .iter()
            .any(|site_trait| site_trait.contains("Bridge") || site_trait.contains("Gate"))
        {
            percent += 10;
        }
    }

    ResourceStock {
        food: scaled_cost(base_cost.food, percent),
        timber: scaled_cost(base_cost.timber, percent),
        stone: scaled_cost(base_cost.stone, percent),
        wealth: scaled_cost(base_cost.wealth, percent),
    }
}

fn scaled_cost(value: i32, percent: i32) -> i32 {
    ((value * percent) as f32 / 100.0).round() as i32
}

fn route_warning_for(
    route: &RouteRuntimeState,
    road: &RoadDef,
    season: Season,
    turn: u32,
    high_danger_sites: &HashSet<&str>,
) -> Option<&'static str> {
    if season == Season::Winter && route.level == RouteLevel::Path {
        return Some("winter_blockage");
    }
    if season == Season::Spring
        && (road.route_type.contains("ford") || road.route_type.contains("causeway"))
    {
        return Some("bridge_washout");
    }
    if turn % 4 == 0
        && (high_danger_sites.contains(route.site_a.as_str())
            || high_danger_sites.contains(route.site_b.as_str()))
    {
        return Some("caravan_attacked");
    }

    None
}

fn apply_warning_to_route(route: &mut RouteRuntimeState, warning_id: &str) {
    if !route
        .active_warning_ids
        .iter()
        .any(|existing| existing == warning_id)
    {
        route.active_warning_ids.push(warning_id.to_owned());
    }
    route.condition = match warning_id {
        "winter_blockage" => RouteCondition::Blocked,
        _ => RouteCondition::Damaged,
    };
}

fn relax_distance(
    distances: &mut HashMap<String, i32>,
    from_site_id: &str,
    to_site_id: &str,
    cost: i32,
) -> bool {
    let Some(from_distance) = distances.get(from_site_id).copied() else {
        return false;
    };
    let new_distance = from_distance + cost;
    if distances
        .get(to_site_id)
        .map(|existing| new_distance < *existing)
        .unwrap_or(true)
    {
        distances.insert(to_site_id.to_owned(), new_distance);
        true
    } else {
        false
    }
}

fn apply_isolation_penalty(settlement: &mut SettlementRuntimeState, unmanaged_strain: i32) {
    add_unique_issue(&mut settlement.active_issue_ids, "isolated");
    settlement.stability = (settlement.stability - 3 - unmanaged_strain).clamp(0, 100);
    settlement.loyalty = (settlement.loyalty - 2).clamp(0, 100);
    settlement.prosperity = (settlement.prosperity - 1).clamp(0, 100);
    settlement.danger = (settlement.danger + 2).clamp(0, 100);
    settlement.autonomy_pressure = (settlement.autonomy_pressure + 1 + unmanaged_strain).max(0);
    settlement.rival_pressure = (settlement.rival_pressure + 2 * unmanaged_strain).max(0);
}

fn apply_connection_bonus(
    settlement: &mut SettlementRuntimeState,
    distances: &HashMap<String, i32>,
) {
    remove_issue(&mut settlement.active_issue_ids, "isolated");
    settlement.loyalty = (settlement.loyalty + 1).clamp(0, 100);
    if settlement.focus_id == "trade" {
        settlement.prosperity = (settlement.prosperity + 2).clamp(0, 100);
        settlement.stored.wealth += 5;
    }
    if distances
        .get(&settlement.location_id)
        .map(|distance| *distance <= 4)
        .unwrap_or(false)
    {
        settlement.danger = (settlement.danger - 1).clamp(0, 100);
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

fn council_capacity(session: &GameSession) -> i32 {
    let mut capacity = 2;
    if session
        .settlements
        .iter()
        .any(|settlement| settlement.tier >= crate::data::SettlementTier::Town)
    {
        capacity += 1;
    }
    if session
        .settlements
        .iter()
        .filter(|settlement| settlement.status == SettlementStatus::Active)
        .count()
        >= 5
    {
        capacity += 1;
    }
    capacity.min(5)
}

fn route_label(data: &GameData, route: &RouteRuntimeState) -> String {
    let site_a = data
        .site(&route.site_a)
        .map(|site| site.name.as_str())
        .unwrap_or(route.site_a.as_str());
    let site_b = data
        .site(&route.site_b)
        .map(|site| site.name.as_str())
        .unwrap_or(route.site_b.as_str());
    format!("{} to {}", site_a, site_b)
}

fn region_label(data: &GameData, region_id: &str) -> String {
    data.region(region_id)
        .map(|region| region.name.clone())
        .unwrap_or_else(|| region_id.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::GameSession;

    fn test_data() -> GameData {
        GameData::load().unwrap()
    }

    fn found_two_settlements(data: &GameData) -> GameSession {
        let mut session = GameSession::new(data);
        assert!(session.select_site(data, "lowmeadow"));
        session.found_selected_camp(data).unwrap();
        assert!(session.select_site(data, "emberbrook"));
        session.found_selected_camp(data).unwrap();
        session.advance_season(data);
        session
    }

    #[test]
    fn starting_routes_track_authored_links() {
        let data = test_data();
        let session = GameSession::new(&data);
        let route = session
            .routes
            .iter()
            .find(|route| route.id == "road_06")
            .unwrap();

        assert_eq!(route.level, RouteLevel::None);
        assert!(route.known);
    }

    #[test]
    fn building_path_connects_settlement_to_capital_network() {
        let data = test_data();
        let mut session = found_two_settlements(&data);

        assert!(!session.is_site_in_capital_network(&data, "emberbrook"));
        let status = session.route_action_status(&data, "road_06");
        assert!(status.enabled, "{}", status.reason);
        session.build_or_upgrade_route(&data, "road_06").unwrap();

        assert!(session.is_site_in_capital_network(&data, "emberbrook"));
        assert_eq!(
            session
                .routes
                .iter()
                .find(|route| route.id == "road_06")
                .unwrap()
                .level,
            RouteLevel::Path
        );
    }

    #[test]
    fn disconnected_settlement_takes_isolation_pressure() {
        let data = test_data();
        let mut session = found_two_settlements(&data);

        let report = session.advance_road_and_supply(&data);
        let settlement = session.settlement_at_site("emberbrook").unwrap();
        assert_eq!(report.isolated_settlements, 1);
        assert!(settlement
            .active_issue_ids
            .iter()
            .any(|issue| issue == "isolated"));
        assert!(settlement.autonomy_pressure > 0);
    }
}
