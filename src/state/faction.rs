//! Rival faction, independent settlement, and wilderness pressure simulation.

use super::{GameSession, SettlementActionStatus, SettlementStatus};
use crate::data::{FactionGoal, GameData, SiteCategory};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionRuntimeState {
    pub id: String,
    pub name: String,
    pub personality: String,
    pub description: String,
    pub confidence: i32,
    pub fear: i32,
    pub hostility: i32,
    pub border_pressure: i32,
    pub recent_losses: i32,
    pub current_goal: FactionGoal,
    pub goal_age: u32,
    pub action_cooldowns: Vec<FactionCooldown>,
    pub controlled_locations: Vec<String>,
    pub known_targets: Vec<String>,
    pub memory_tags: Vec<String>,
    pub action_log: Vec<FactionActionLogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionCooldown {
    pub key: String,
    pub remaining: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionActionLogEntry {
    pub year: u32,
    pub season: super::Season,
    pub action: String,
    pub target_site_id: Option<String>,
    pub reason: String,
    pub result: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationState {
    Independent,
    Trading,
    Integrating,
    Integrated,
    Resistant,
}

impl IntegrationState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Independent => "Independent",
            Self::Trading => "Trading",
            Self::Integrating => "Integrating",
            Self::Integrated => "Integrated",
            Self::Resistant => "Resistant",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndependentSettlementRuntimeState {
    pub site_id: String,
    pub trust: i32,
    pub autonomy: i32,
    pub integration_progress: i32,
    pub rival_pressure: i32,
    pub local_need: String,
    pub trade_relationship: bool,
    pub protection_relationship: bool,
    pub integration_state: IntegrationState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WildernessPressureState {
    pub region_id: String,
    pub pressure: i32,
    pub last_delta: i32,
}

impl WildernessPressureState {
    pub fn band(&self) -> &'static str {
        match self.pressure {
            0..=24 => "Quiet",
            25..=49 => "Watchful",
            50..=74 => "Dangerous",
            _ => "Lawless",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct FactionAdvanceReport {
    pub rival_actions: usize,
    pub independent_requests: usize,
    pub wilderness_changes: usize,
}

#[derive(Debug, Clone)]
struct GoalScore {
    goal: FactionGoal,
    score: i32,
}

impl GameSession {
    pub fn create_starting_rival(data: &GameData) -> FactionRuntimeState {
        let rival = &data.faction_balance.rival;
        FactionRuntimeState {
            id: rival.id.clone(),
            name: rival.name.clone(),
            personality: rival.personality.clone(),
            description: rival.description.clone(),
            confidence: rival.confidence,
            fear: rival.fear,
            hostility: rival.hostility,
            border_pressure: rival.border_pressure,
            recent_losses: 0,
            current_goal: FactionGoal::Expand,
            goal_age: 4,
            action_cooldowns: Vec::new(),
            controlled_locations: rival.controlled_locations.clone(),
            known_targets: Vec::new(),
            memory_tags: rival.memory_tags.clone(),
            action_log: Vec::new(),
        }
    }

    pub fn create_independent_states(data: &GameData) -> Vec<IndependentSettlementRuntimeState> {
        data.sites
            .iter()
            .filter(|site| site.category == SiteCategory::Independent)
            .map(|site| IndependentSettlementRuntimeState {
                site_id: site.id.clone(),
                trust: 30,
                autonomy: 70,
                integration_progress: 0,
                rival_pressure: 10,
                local_need: "road access".to_owned(),
                trade_relationship: false,
                protection_relationship: false,
                integration_state: IntegrationState::Independent,
            })
            .collect()
    }

    pub fn create_wilderness_pressure(data: &GameData) -> Vec<WildernessPressureState> {
        data.regions
            .iter()
            .map(|region| WildernessPressureState {
                region_id: region.id.clone(),
                pressure: data.faction_balance.wilderness_base_pressure,
                last_delta: 0,
            })
            .collect()
    }

    pub fn selected_independent(&self) -> Option<&IndependentSettlementRuntimeState> {
        self.independent_settlements
            .iter()
            .find(|state| state.site_id == self.selected_site_id)
    }

    pub fn independent_trade_status(&self) -> SettlementActionStatus {
        let Some(independent) = self.selected_independent() else {
            return SettlementActionStatus::disabled("No independent settlement selected.");
        };
        if independent.trade_relationship {
            return SettlementActionStatus::disabled("Trade is already open.");
        }
        if independent.trust < 20 {
            return SettlementActionStatus::disabled("Trust is too low for trade.");
        }

        SettlementActionStatus::enabled("Open trade: +15 trust, starts a trade relationship.")
    }

    pub fn integration_status(&self) -> SettlementActionStatus {
        let Some(independent) = self.selected_independent() else {
            return SettlementActionStatus::disabled("No independent settlement selected.");
        };
        if independent.integration_state == IntegrationState::Integrated {
            return SettlementActionStatus::disabled("Already integrated.");
        }
        if independent.trust < 45 {
            return SettlementActionStatus::disabled("Needs trust 45+.");
        }

        SettlementActionStatus::enabled("Begin integration: +30 progress, autonomy falls.")
    }

    pub fn open_trade_with_selected_independent(&mut self) -> Result<String, String> {
        let status = self.independent_trade_status();
        if !status.enabled {
            return Err(status.reason);
        }
        let independent = self
            .independent_settlements
            .iter_mut()
            .find(|state| state.site_id == self.selected_site_id)
            .ok_or_else(|| "No independent settlement selected.".to_owned())?;
        independent.trade_relationship = true;
        independent.integration_state = IntegrationState::Trading;
        independent.trust = (independent.trust + 15).clamp(0, 100);
        independent.local_need = "market access".to_owned();

        Ok("Opened independent trade relationship.".to_owned())
    }

    pub fn begin_selected_integration(&mut self, data: &GameData) -> Result<String, String> {
        let status = self.integration_status();
        if !status.enabled {
            return Err(status.reason);
        }
        let selected_site_id = self.selected_site_id.clone();
        let independent = self
            .independent_settlements
            .iter_mut()
            .find(|state| state.site_id == selected_site_id)
            .ok_or_else(|| "No independent settlement selected.".to_owned())?;
        independent.integration_state = IntegrationState::Integrating;
        independent.integration_progress = (independent.integration_progress + 30).clamp(0, 100);
        independent.autonomy = (independent.autonomy - 10).clamp(0, 100);
        if independent.integration_progress >= 100 {
            independent.integration_state = IntegrationState::Integrated;
            self.add_chronicle_entry(data, "independent_integrated", Some(&selected_site_id));
        } else {
            self.add_chronicle_entry(
                data,
                "independent_integration_started",
                Some(&selected_site_id),
            );
        }

        Ok("Began independent settlement integration.".to_owned())
    }

    pub fn advance_faction_systems(&mut self, data: &GameData) -> FactionAdvanceReport {
        let wilderness_changes = self.update_wilderness_pressure(data);
        let independent_requests = self.update_independents(data);
        let rival_actions = self.update_rival(data);

        FactionAdvanceReport {
            rival_actions,
            independent_requests,
            wilderness_changes,
        }
    }

    pub fn rival_controls_site(&self, site_id: &str) -> bool {
        self.rival_faction
            .controlled_locations
            .iter()
            .any(|location_id| location_id == site_id)
    }

    fn update_rival(&mut self, data: &GameData) -> usize {
        for cooldown in &mut self.rival_faction.action_cooldowns {
            cooldown.remaining = cooldown.remaining.saturating_sub(1);
        }
        self.rival_faction
            .action_cooldowns
            .retain(|cooldown| cooldown.remaining > 0);
        self.rival_faction.goal_age += 1;
        self.rival_faction.known_targets = self
            .site_states
            .iter()
            .filter(|state| state.knowledge == super::SiteKnowledge::Known)
            .map(|state| state.site_id.clone())
            .collect();

        if self.rival_faction.goal_age >= 4
            || !self.goal_has_target(data, self.rival_faction.current_goal)
        {
            self.rival_faction.current_goal = self.score_rival_goal(data);
            self.rival_faction.goal_age = 0;
        }

        if self
            .rival_faction
            .action_cooldowns
            .iter()
            .any(|cooldown| cooldown.key == self.rival_faction.current_goal.label())
        {
            return 0;
        }
        let Some((target_site_id, reason)) =
            self.best_target_for_goal(data, self.rival_faction.current_goal)
        else {
            return 0;
        };
        let result =
            self.execute_rival_action(data, self.rival_faction.current_goal, &target_site_id);
        self.rival_faction.action_log.push(FactionActionLogEntry {
            year: self.clock.year,
            season: self.clock.season,
            action: self.rival_faction.current_goal.label().to_owned(),
            target_site_id: Some(target_site_id.clone()),
            reason,
            result: result.clone(),
        });
        if self.rival_faction.action_log.len() == 1 {
            self.add_chronicle_entry(data, "rival_first_move", Some(&target_site_id));
        }
        self.rival_faction.action_cooldowns.push(FactionCooldown {
            key: self.rival_faction.current_goal.label().to_owned(),
            remaining: 2,
        });
        1
    }

    fn score_rival_goal(&self, data: &GameData) -> FactionGoal {
        let weak_target = self.weakest_player_settlement_score(data);
        let open_sites = data
            .sites
            .iter()
            .filter(|site| {
                site.category == SiteCategory::Settlement
                    && self.is_known(&site.id)
                    && self.settlement_at_site(&site.id).is_none()
            })
            .count() as i32;
        let disconnected = self
            .settlements
            .iter()
            .filter(|settlement| {
                settlement.status == SettlementStatus::Active
                    && !self.is_site_in_capital_network(data, &settlement.location_id)
            })
            .count() as i32;

        let mut scores = Vec::new();
        for goal in FactionGoal::all() {
            let mut score = match goal {
                FactionGoal::Expand => open_sites * 12 + self.rival_faction.confidence / 2,
                FactionGoal::Fortify => {
                    self.rival_faction.fear + self.rival_faction.border_pressure / 2
                }
                FactionGoal::Raid => {
                    weak_target + self.rival_faction.hostility / 2 + disconnected * 8
                }
                FactionGoal::Trade => 35 + (100 - self.rival_faction.hostility) / 3,
                FactionGoal::Influence => weak_target + self.rival_faction.border_pressure / 2,
                FactionGoal::Recover => {
                    self.rival_faction.recent_losses * 2 + self.rival_faction.fear / 2
                }
                FactionGoal::Confront => {
                    self.rival_faction.border_pressure + self.rival_faction.confidence / 2
                }
                FactionGoal::Appease => {
                    self.rival_faction.fear + (100 - self.rival_faction.hostility) / 3
                }
            };
            if self.rival_faction.personality == "Opportunist"
                && matches!(goal, FactionGoal::Raid | FactionGoal::Influence)
            {
                score += 20;
            }
            if goal == self.rival_faction.current_goal {
                score += 8;
            }
            scores.push(GoalScore { goal, score });
        }
        scores
            .into_iter()
            .max_by_key(|score| score.score)
            .map(|score| score.goal)
            .unwrap_or(FactionGoal::Expand)
    }

    fn goal_has_target(&self, data: &GameData, goal: FactionGoal) -> bool {
        self.best_target_for_goal(data, goal).is_some()
    }

    fn best_target_for_goal(&self, data: &GameData, goal: FactionGoal) -> Option<(String, String)> {
        match goal {
            FactionGoal::Expand => data
                .sites
                .iter()
                .find(|site| {
                    site.category == SiteCategory::Settlement
                        && self.is_known(&site.id)
                        && self.settlement_at_site(&site.id).is_none()
                        && !self.rival_controls_site(&site.id)
                })
                .map(|site| {
                    (
                        site.id.clone(),
                        "open known settlement site near the contested frontier".to_owned(),
                    )
                }),
            FactionGoal::Raid | FactionGoal::Influence | FactionGoal::Confront => self
                .settlements
                .iter()
                .filter(|settlement| settlement.status == SettlementStatus::Active)
                .max_by_key(|settlement| target_weakness(self, data, &settlement.location_id))
                .map(|settlement| {
                    (
                        settlement.location_id.clone(),
                        "low loyalty, isolation, or weak defence made this target attractive"
                            .to_owned(),
                    )
                }),
            FactionGoal::Trade => self.independent_settlements.first().map(|state| {
                (
                    state.site_id.clone(),
                    "independent trade could increase rival influence".to_owned(),
                )
            }),
            FactionGoal::Fortify | FactionGoal::Recover | FactionGoal::Appease => self
                .rival_faction
                .controlled_locations
                .first()
                .map(|site_id| {
                    (
                        site_id.clone(),
                        "current clan holdings need attention".to_owned(),
                    )
                }),
        }
    }

    fn execute_rival_action(
        &mut self,
        data: &GameData,
        goal: FactionGoal,
        target_site_id: &str,
    ) -> String {
        match goal {
            FactionGoal::Expand => {
                if !self.rival_controls_site(target_site_id) {
                    self.rival_faction
                        .controlled_locations
                        .push(target_site_id.to_owned());
                }
                self.rival_faction.border_pressure =
                    (self.rival_faction.border_pressure + 4).clamp(0, 100);
                "claimed a frontier site".to_owned()
            }
            FactionGoal::Raid => self.resolve_rival_raid(data, target_site_id),
            FactionGoal::Influence => {
                if let Some(settlement) = self
                    .settlements
                    .iter_mut()
                    .find(|settlement| settlement.location_id == target_site_id)
                {
                    settlement.rival_pressure += 8;
                    settlement.loyalty = (settlement.loyalty - 4).clamp(0, 100);
                    add_unique_issue(&mut settlement.active_issue_ids, "rival_influence");
                }
                "supported separatist voices".to_owned()
            }
            FactionGoal::Trade => {
                if let Some(independent) = self
                    .independent_settlements
                    .iter_mut()
                    .find(|state| state.site_id == target_site_id)
                {
                    independent.rival_pressure += 6;
                    independent.trust = (independent.trust - 3).clamp(0, 100);
                }
                "sent merchants to an independent settlement".to_owned()
            }
            FactionGoal::Fortify => {
                self.rival_faction.confidence = (self.rival_faction.confidence + 4).clamp(0, 100);
                "fortified a border holding".to_owned()
            }
            FactionGoal::Recover => {
                self.rival_faction.recent_losses = (self.rival_faction.recent_losses - 4).max(0);
                self.rival_faction.confidence = (self.rival_faction.confidence + 2).clamp(0, 100);
                "recovered from recent losses".to_owned()
            }
            FactionGoal::Confront => {
                self.rival_faction.hostility = (self.rival_faction.hostility + 5).clamp(0, 100);
                "issued a border demand".to_owned()
            }
            FactionGoal::Appease => {
                self.rival_faction.hostility = (self.rival_faction.hostility - 6).clamp(0, 100);
                "sent a cautious truce feeler".to_owned()
            }
        }
    }

    fn resolve_rival_raid(&mut self, data: &GameData, target_site_id: &str) -> String {
        let connected_bonus = if self.is_site_in_capital_network(data, target_site_id) {
            10
        } else {
            -10
        };
        let Some(settlement) = self
            .settlements
            .iter_mut()
            .find(|settlement| settlement.location_id == target_site_id)
        else {
            return "found no legal raid target".to_owned();
        };
        let attack = self.rival_faction.confidence + self.rival_faction.hostility / 2;
        let defence = settlement.defence + settlement.stability / 2 + connected_bonus;
        let margin = attack - defence;
        if margin > 15 {
            settlement.stored.food = (settlement.stored.food - 25).max(0);
            settlement.stored.wealth = (settlement.stored.wealth - 20).max(0);
            settlement.stability = (settlement.stability - 8).clamp(0, 100);
            settlement.loyalty = (settlement.loyalty - 5).clamp(0, 100);
            settlement.rival_pressure += 10;
            self.rival_faction.hostility = (self.rival_faction.hostility + 4).clamp(0, 100);
            self.add_chronicle_entry(data, "rival_major_raid", Some(target_site_id));
            "raided successfully after reading weak supply".to_owned()
        } else {
            settlement.defence = (settlement.defence + 2).clamp(0, 100);
            self.rival_faction.recent_losses += 3;
            self.rival_faction.fear = (self.rival_faction.fear + 3).clamp(0, 100);
            "tested the border but met resistance".to_owned()
        }
    }

    fn update_independents(&mut self, data: &GameData) -> usize {
        let mut requests = Vec::new();
        for independent in &mut self.independent_settlements {
            if independent.integration_state == IntegrationState::Integrated {
                continue;
            }
            independent.rival_pressure = (independent.rival_pressure + 1).clamp(0, 100);
            if independent.trade_relationship {
                independent.trust = (independent.trust + 1).clamp(0, 100);
            }
            if independent.rival_pressure > 35 && independent.local_need != "aid requested" {
                independent.local_need = "aid requested".to_owned();
                independent.autonomy = (independent.autonomy + 4).clamp(0, 100);
                requests.push(independent.site_id.clone());
            }
        }
        for site_id in &requests {
            self.add_chronicle_entry(data, "independent_request", Some(site_id));
            self.rival_faction.action_log.push(FactionActionLogEntry {
                year: self.clock.year,
                season: self.clock.season,
                action: "Independent Request".to_owned(),
                target_site_id: Some(site_id.clone()),
                reason: "rival pressure and local need crossed the request threshold".to_owned(),
                result: "the settlement asked the charter for aid or trade".to_owned(),
            });
        }
        requests.len()
    }

    fn update_wilderness_pressure(&mut self, data: &GameData) -> usize {
        let mut changes = Vec::new();
        let completed_project_regions: Vec<String> = self
            .regional_projects
            .iter()
            .map(|project| project.region_id.clone())
            .collect();
        for pressure in &mut self.wilderness_pressure {
            let old_pressure = pressure.pressure;
            let isolated = self
                .settlements
                .iter()
                .filter(|settlement| {
                    data.site(&settlement.location_id)
                        .map(|site| site.region_id == pressure.region_id)
                        .unwrap_or(false)
                        && settlement
                            .active_issue_ids
                            .iter()
                            .any(|issue| issue == "isolated")
                })
                .count() as i32;
            let fortifications = self
                .settlements
                .iter()
                .filter(|settlement| {
                    data.site(&settlement.location_id)
                        .map(|site| site.region_id == pressure.region_id)
                        .unwrap_or(false)
                        && settlement.focus_id == "fortification"
                })
                .count() as i32;
            let roads = self
                .routes
                .iter()
                .filter(|route| {
                    route.level.is_built() && route.region_ids.contains(&pressure.region_id)
                })
                .count() as i32;
            let patrol = completed_project_regions
                .iter()
                .any(|region_id| region_id == &pressure.region_id);
            let mut delta = 2 + isolated;
            delta -= roads * data.faction_balance.road_pressure_reduction;
            delta -= fortifications * data.faction_balance.fortification_pressure_reduction;
            if patrol {
                delta -= data.faction_balance.trail_warden_pressure_reduction;
            }
            pressure.last_delta = delta;
            pressure.pressure = (pressure.pressure + delta).clamp(0, 100);
            if old_pressure < 50 && pressure.pressure >= 50 {
                changes.push((pressure.region_id.clone(), "wilderness_escalation"));
            } else if old_pressure >= 50 && pressure.pressure < 50 {
                changes.push((pressure.region_id.clone(), "wilderness_recovery"));
            }
        }
        for (region_id, template_id) in &changes {
            let site_id = data
                .sites
                .iter()
                .find(|site| site.region_id == *region_id && self.is_known(&site.id))
                .map(|site| site.id.clone());
            self.add_chronicle_entry(data, template_id, site_id.as_deref());
        }
        changes.len()
    }

    fn weakest_player_settlement_score(&self, data: &GameData) -> i32 {
        self.settlements
            .iter()
            .filter(|settlement| settlement.status == SettlementStatus::Active)
            .map(|settlement| target_weakness(self, data, &settlement.location_id))
            .max()
            .unwrap_or(0)
    }
}

fn target_weakness(session: &GameSession, data: &GameData, site_id: &str) -> i32 {
    let Some(settlement) = session.settlement_at_site(site_id) else {
        return 0;
    };
    let isolation = if session.is_site_in_capital_network(data, site_id) {
        0
    } else {
        20
    };
    (100 - settlement.loyalty).max(0)
        + (100 - settlement.stability).max(0) / 2
        + (40 - settlement.defence).max(0)
        + settlement.rival_pressure
        + isolation
}

fn add_unique_issue(issue_ids: &mut Vec<String>, issue_id: &str) {
    if !issue_ids.iter().any(|existing| existing == issue_id) {
        issue_ids.push(issue_id.to_owned());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::GameSession;

    fn test_data() -> GameData {
        GameData::load().unwrap()
    }

    #[test]
    fn new_campaign_has_rival_independents_and_wilderness() {
        let data = test_data();
        let session = GameSession::new(&data);

        assert_eq!(session.rival_faction.id, "ashthorn_clan");
        assert_eq!(session.independent_settlements.len(), 4);
        assert_eq!(session.wilderness_pressure.len(), data.regions.len());
    }

    #[test]
    fn rival_executes_visible_action() {
        let data = test_data();
        let mut session = GameSession::new(&data);

        let report = session.advance_faction_systems(&data);
        assert_eq!(report.rival_actions, 1);
        assert_eq!(session.rival_faction.action_log.len(), 1);
    }

    #[test]
    fn independent_trade_and_integration_progress() {
        let data = test_data();
        let mut session = GameSession::new(&data);

        assert!(session.select_site(&data, "briarford"));
        session.open_trade_with_selected_independent().unwrap();
        {
            let independent = session
                .independent_settlements
                .iter_mut()
                .find(|state| state.site_id == "briarford")
                .unwrap();
            independent.trust = 50;
        }
        session.begin_selected_integration(&data).unwrap();
        let independent = session.selected_independent().unwrap();
        assert!(independent.integration_progress > 0);
    }
}
