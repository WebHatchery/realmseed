//! Seasonal road warnings, capital-network distances, and isolation pressure.

use super::{
    GameSession, RouteCondition, RouteRuntimeState, Season, SettlementRuntimeState,
    SettlementStatus,
};
use crate::data::{GameData, RoadDef, RouteLevel};
use std::collections::{HashMap, HashSet};

impl GameSession {
    pub fn advance_road_and_supply(&mut self, data: &GameData) -> super::RoadAdvanceReport {
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

        super::RoadAdvanceReport {
            isolated_settlements: disconnected_sites.len(),
            road_warnings,
            unmanaged_strain,
        }
    }

    pub(super) fn capital_distances(&self, data: &GameData) -> HashMap<String, i32> {
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
    if turn.is_multiple_of(4)
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
    settlement.autonomy_pressure =
        (settlement.autonomy_pressure + 1 + unmanaged_strain).clamp(0, 100);
    settlement.rival_pressure = (settlement.rival_pressure + 2 * unmanaged_strain).clamp(0, 100);
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
