//! Complete prototype campaign flow, ambitions, projects, institutions, and endings.

use super::{
    GameSession, SeasonAdvanceReport, SeasonSummaryRow, SettlementActionStatus, SettlementStatus,
};
use crate::data::{GameData, ProjectDef, SettlementTier};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedProjectState {
    pub id: String,
    pub target_id: String,
    pub completed_year: u32,
    pub completed_season: super::Season,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveInstitutionState {
    pub id: String,
    pub unlocked_year: u32,
    pub unlocked_season: super::Season,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndgameSummary {
    pub legacy_score: i32,
    pub ending_band: String,
    pub worst_year: u32,
    pub golden_year: u32,
    pub strongest_identity: String,
    pub largest_settlement: String,
    pub defining_event: String,
    pub arcs: Vec<String>,
    pub summary_text: String,
}

#[derive(Debug, Clone, Default)]
pub struct CampaignAdvanceReport {
    pub campaign_finished: bool,
}

impl GameSession {
    pub fn guidance_text(&self) -> String {
        if self.known_site_count() <= 8 {
            return "Guidance - scouting: inspect an adjacent question marker before committing actions.".to_owned();
        }
        if self
            .settlements
            .iter()
            .filter(|settlement| settlement.is_active())
            .count()
            < 2
        {
            return "Guidance - founding: choose a known settlement site and found the first frontier camp.".to_owned();
        }
        if !self.routes.iter().any(|route| route.level.is_built()) {
            return "Guidance - roads: build a path so new settlements can reach the capital network."
                .to_owned();
        }
        if self.pending_event.is_some() || !self.active_issues.is_empty() {
            return "Guidance - first crisis: resolve warnings by reading causes and choosing a response.".to_owned();
        }
        if self
            .rival_faction
            .action_log
            .iter()
            .any(|entry| entry.action == "Independent Request")
        {
            return "Guidance - independent request: aid, trade, or protection changes trust and autonomy.".to_owned();
        }
        if self.rival_faction.action_log.is_empty() {
            return "Guidance - rival pressure: advance seasons and watch the faction log with F."
                .to_owned();
        }
        if self.rival_faction.action_log.len() == 1 {
            return "Guidance - first rival action: the log explains target, reason, and result."
                .to_owned();
        }
        if self.selected_ambition_id.is_none() {
            return "Guidance - ambition: open Faction Pressure with F and declare a realm ambition."
                .to_owned();
        }
        if self
            .independent_settlements
            .iter()
            .all(|independent| !independent.trade_relationship)
        {
            return "Guidance - independents: select an independent settlement and open trade or integration."
                .to_owned();
        }
        "Guidance: build projects, unlock institutions, and shape the realm's final legacy."
            .to_owned()
    }

    pub fn ambition_objective_text(&self, data: &GameData, ambition_id: &str) -> String {
        let progress = self.ambition_progress(data, ambition_id);
        match ambition_id {
            "breadbasket" => format!(
                "Objective: keep food reserves high, answer hunger events, and finish granary projects. Progress {}.",
                progress
            ),
            "roadbound" => format!(
                "Objective: connect settlements, upgrade roads, and reduce isolation. Progress {}.",
                progress
            ),
            "civic" => format!(
                "Objective: sustain loyalty/stability, settle disputes, and integrate independents. Progress {}.",
                progress
            ),
            _ => format!("Objective progress {}.", progress),
        }
    }

    pub fn wilderness_modifier(&self, data: &GameData) -> i32 {
        data.campaign_balance
            .difficulty(&self.selected_difficulty_id)
            .map(|preset| preset.wilderness_modifier)
            .unwrap_or(0)
    }

    pub fn select_ambition(
        &mut self,
        data: &GameData,
        ambition_id: &str,
    ) -> Result<String, String> {
        let ambition = data
            .campaign_balance
            .ambition(ambition_id)
            .ok_or_else(|| "Unknown ambition.".to_owned())?;
        self.selected_ambition_id = Some(ambition.id.clone());
        self.add_chronicle_entry(
            data,
            "ambition_declared",
            Some(&self.selected_site_id.clone()),
        );
        Ok(format!("Declared {}", ambition.name))
    }

    pub fn ambition_progress(&self, data: &GameData, ambition_id: &str) -> i32 {
        match ambition_id {
            "breadbasket" => {
                let food = self
                    .settlements
                    .iter()
                    .map(|settlement| settlement.stored.food)
                    .sum::<i32>();
                (food / 8).clamp(0, 100)
            }
            "roadbound" => {
                let built = self
                    .routes
                    .iter()
                    .filter(|route| route.level.is_built())
                    .count() as i32;
                (built * 12).clamp(0, 100)
            }
            "civic" => average_loyalty_stability(self).clamp(0, 100),
            _ => data
                .campaign_balance
                .ambition(ambition_id)
                .map(|_| 0)
                .unwrap_or(0),
        }
    }

    pub fn project_status(&self, data: &GameData, project_id: &str) -> SettlementActionStatus {
        let Some(project) = data.campaign_balance.project(project_id) else {
            return SettlementActionStatus::disabled("Unknown project.");
        };
        let target_id = self.project_target_id(data, project);
        if self
            .completed_projects
            .iter()
            .any(|completed| completed.id == project.id && completed.target_id == target_id)
        {
            return SettlementActionStatus::disabled("Project already completed here.");
        }
        let Some(source) = self.project_source_settlement(project) else {
            return SettlementActionStatus::disabled("Select a valid settlement or region.");
        };
        if let Some(reason) = source.stored.deficit_text(project.cost) {
            return SettlementActionStatus::disabled(reason);
        }

        SettlementActionStatus::enabled(format!(
            "Costs {}. {}",
            project.cost.cost_text(),
            project.description
        ))
    }

    pub fn complete_project(
        &mut self,
        data: &GameData,
        project_id: &str,
    ) -> Result<String, String> {
        let status = self.project_status(data, project_id);
        if !status.enabled {
            return Err(status.reason);
        }
        let project = data
            .campaign_balance
            .project(project_id)
            .ok_or_else(|| "Unknown project.".to_owned())?
            .clone();
        let target_id = self.project_target_id(data, &project);
        let source_site_id = if project.scope == "settlement" {
            self.selected_site_id.clone()
        } else {
            data.road_balance.source_site_id.clone()
        };
        let source = self
            .settlements
            .iter_mut()
            .find(|settlement| settlement.location_id == source_site_id)
            .ok_or_else(|| "Project source settlement is unavailable.".to_owned())?;
        source.stored.subtract(project.cost);
        self.apply_project_effect(data, &project, &target_id);
        self.completed_projects.push(CompletedProjectState {
            id: project.id.clone(),
            target_id,
            completed_year: self.clock.year,
            completed_season: self.clock.season,
        });
        self.add_chronicle_entry(
            data,
            "project_completed",
            Some(&self.selected_site_id.clone()),
        );

        Ok(format!("Completed {}", project.name))
    }

    pub fn institution_status(
        &self,
        data: &GameData,
        institution_id: &str,
    ) -> SettlementActionStatus {
        let Some(institution) = data.campaign_balance.institution(institution_id) else {
            return SettlementActionStatus::disabled("Unknown institution.");
        };
        if self
            .active_institutions
            .iter()
            .any(|active| active.id == institution.id)
        {
            return SettlementActionStatus::disabled("Institution already active.");
        }
        let unlocked = match institution.id.as_str() {
            "road_wardens" => {
                self.routes
                    .iter()
                    .filter(|route| route.level.is_built())
                    .count()
                    >= 2
            }
            "reeve_courts" => {
                self.settlements
                    .iter()
                    .filter(|settlement| settlement.is_active())
                    .count()
                    >= 3
                    || self
                        .settlements
                        .iter()
                        .any(|settlement| settlement.tier >= SettlementTier::Town)
            }
            _ => false,
        };
        if !unlocked {
            return SettlementActionStatus::disabled("Unlock requirement not met.");
        }

        SettlementActionStatus::enabled(institution.description.clone())
    }

    pub fn activate_institution(
        &mut self,
        data: &GameData,
        institution_id: &str,
    ) -> Result<String, String> {
        let status = self.institution_status(data, institution_id);
        if !status.enabled {
            return Err(status.reason);
        }
        let institution = data
            .campaign_balance
            .institution(institution_id)
            .ok_or_else(|| "Unknown institution.".to_owned())?;
        self.active_institutions.push(ActiveInstitutionState {
            id: institution.id.clone(),
            unlocked_year: self.clock.year,
            unlocked_season: self.clock.season,
        });
        self.add_chronicle_entry(
            data,
            "institution_unlocked",
            Some(&self.selected_site_id.clone()),
        );

        Ok(format!("Unlocked {}", institution.name))
    }

    pub fn advance_campaign_systems(
        &mut self,
        data: &GameData,
        report: &SeasonAdvanceReport,
    ) -> CampaignAdvanceReport {
        self.apply_institution_effects();
        self.record_last_season_summary(report);
        if self.endgame_summary.is_none() && self.clock.turn >= data.campaign_balance.campaign_turns
        {
            self.endgame_summary = Some(self.build_endgame_summary(data));
            return CampaignAdvanceReport {
                campaign_finished: true,
            };
        }

        CampaignAdvanceReport::default()
    }

    fn project_target_id(&self, data: &GameData, project: &ProjectDef) -> String {
        if project.scope == "regional" {
            data.site(&self.selected_site_id)
                .map(|site| site.region_id.clone())
                .unwrap_or_else(|| "greenvale_basin".to_owned())
        } else {
            self.selected_site_id.clone()
        }
    }

    fn project_source_settlement(
        &self,
        project: &ProjectDef,
    ) -> Option<&super::SettlementRuntimeState> {
        if project.scope == "settlement" {
            self.selected_settlement()
        } else {
            self.settlement_at_site("charter_hall")
        }
    }

    fn apply_project_effect(&mut self, _data: &GameData, project: &ProjectDef, target_id: &str) {
        match project.id.as_str() {
            "expand_granaries" => {
                if let Some(settlement) = self
                    .settlements
                    .iter_mut()
                    .find(|settlement| settlement.location_id == target_id)
                {
                    settlement.stored.food += 90;
                    add_unique_tag(&mut settlement.memory_tags, "granaries_expanded");
                }
            }
            "charter_market" => {
                if let Some(settlement) = self
                    .settlements
                    .iter_mut()
                    .find(|settlement| settlement.location_id == target_id)
                {
                    settlement.prosperity = (settlement.prosperity + 8).clamp(0, 100);
                    settlement.stored.wealth += 45;
                    add_unique_tag(&mut settlement.memory_tags, "charter_market");
                }
            }
            "frontier_watch" => {
                if let Some(pressure) = self
                    .wilderness_pressure
                    .iter_mut()
                    .find(|pressure| pressure.region_id == target_id)
                {
                    pressure.pressure = (pressure.pressure - 12).clamp(0, 100);
                    pressure.last_delta -= 12;
                }
            }
            _ => {}
        }
    }

    fn apply_institution_effects(&mut self) {
        if self.has_institution("road_wardens") {
            for pressure in &mut self.wilderness_pressure {
                pressure.pressure = (pressure.pressure - 1).clamp(0, 100);
            }
        }
        if self.has_institution("reeve_courts") {
            for settlement in &mut self.settlements {
                if settlement.status == SettlementStatus::Active && settlement.stability < 70 {
                    settlement.stability += 1;
                }
            }
        }
    }

    fn has_institution(&self, institution_id: &str) -> bool {
        self.active_institutions
            .iter()
            .any(|institution| institution.id == institution_id)
    }

    fn record_last_season_summary(&mut self, report: &SeasonAdvanceReport) {
        let mut rows = vec![SeasonSummaryRow {
            label: "Production".to_owned(),
            detail: format!(
                "+{} food, +{} timber, +{} stone, +{} wealth; consumed {} food; population {:+}.",
                report.produced.food,
                report.produced.timber,
                report.produced.stone,
                report.produced.wealth,
                report.food_consumed,
                report.population_delta
            ),
            site_id: None,
            tag: "player_progress".to_owned(),
        }];
        if report.food_shortages > 0 || report.settlements_lost > 0 {
            rows.push(SeasonSummaryRow {
                label: "Warnings".to_owned(),
                detail: format!(
                    "{} food shortages; {} settlements lost.",
                    report.food_shortages, report.settlements_lost
                ),
                site_id: report
                    .first_food_shortage_site_id
                    .clone()
                    .or_else(|| report.first_lost_site_id.clone()),
                tag: "crisis".to_owned(),
            });
        }
        rows.push(SeasonSummaryRow {
            label: "Roads".to_owned(),
            detail: format!(
                "{} road warnings; {} isolated settlements; unmanaged strain {}.",
                report.road_warnings, report.isolated_settlements, report.unmanaged_strain
            ),
            site_id: None,
            tag: "road".to_owned(),
        });
        rows.push(SeasonSummaryRow {
            label: "Issues".to_owned(),
            detail: format!(
                "{} events triggered; {} issue follow-ups.",
                report.events_triggered, report.issues_escalated
            ),
            site_id: None,
            tag: "crisis".to_owned(),
        });
        rows.push(SeasonSummaryRow {
            label: "World Pressure".to_owned(),
            detail: format!(
                "{} rival actions; {} independent requests; {} wilderness changes.",
                report.rival_actions, report.independent_requests, report.wilderness_changes
            ),
            site_id: None,
            tag: "faction".to_owned(),
        });

        self.last_season_summary = rows
            .iter()
            .map(|row| format!("{}: {}", row.label, row.detail))
            .collect::<Vec<_>>()
            .join(" ");
        self.last_season_rows = rows;
    }

    fn build_endgame_summary(&self, data: &GameData) -> EndgameSummary {
        let controlled = self
            .settlements
            .iter()
            .filter(|settlement| settlement.status == SettlementStatus::Active)
            .count() as i32;
        let towns = self
            .settlements
            .iter()
            .filter(|settlement| settlement.tier >= SettlementTier::Town)
            .count() as i32;
        let population = self
            .settlements
            .iter()
            .map(|settlement| settlement.population)
            .sum::<i32>();
        let connected_roads = self
            .routes
            .iter()
            .filter(|route| route.level.is_built())
            .count() as i32;
        let integrated = self
            .independent_settlements
            .iter()
            .filter(|independent| {
                independent.integration_state == super::IntegrationState::Integrated
                    || independent.integration_progress > 0
            })
            .count() as i32;
        let lost = self
            .settlements
            .iter()
            .filter(|settlement| settlement.status != SettlementStatus::Active)
            .count() as i32;
        let legacy_score = controlled * 10
            + towns * 25
            + population / 25
            + connected_roads * 10
            + integrated * 40
            + if average_loyalty_stability(self) > 60 {
                75
            } else {
                0
            }
            - lost * 50;
        let ending_band = ending_band(legacy_score).to_owned();
        let largest_settlement = self
            .settlements
            .iter()
            .max_by_key(|settlement| settlement.population)
            .map(|settlement| settlement.name.clone())
            .unwrap_or_else(|| "Charter Hall".to_owned());
        let strongest_identity = strongest_identity(self, data);
        let (worst_year, golden_year) = chronicle_years(self);
        let defining_event = self
            .chronicle
            .iter()
            .rev()
            .find(|entry| entry.importance == "major")
            .map(|entry| entry.title.clone())
            .unwrap_or_else(|| "The Charter Is Raised".to_owned());
        let arcs = build_arc_labels(self);
        let ambition = self
            .selected_ambition_id
            .as_deref()
            .and_then(|id| data.campaign_balance.ambition(id))
            .map(|ambition| ambition.name.clone())
            .unwrap_or_else(|| "No declared ambition".to_owned());
        let summary_text = format!(
            "After 20 years, the realm ended as a {} with {} legacy points. Its strongest identity was {}. Largest settlement: {}. Worst year: Year {}; golden year: Year {}. Defining event: {}. Ambition: {}. Arcs: {}.",
            ending_band,
            legacy_score,
            strongest_identity,
            largest_settlement,
            worst_year,
            golden_year,
            defining_event,
            ambition,
            arcs.join(" / ")
        );

        EndgameSummary {
            legacy_score,
            ending_band,
            worst_year,
            golden_year,
            strongest_identity,
            largest_settlement,
            defining_event,
            arcs,
            summary_text,
        }
    }
}

fn average_loyalty_stability(session: &GameSession) -> i32 {
    let active: Vec<_> = session
        .settlements
        .iter()
        .filter(|settlement| settlement.status == SettlementStatus::Active)
        .collect();
    if active.is_empty() {
        return 0;
    }
    let total = active
        .iter()
        .map(|settlement| settlement.loyalty + settlement.stability)
        .sum::<i32>();
    total / (active.len() as i32 * 2)
}

fn ending_band(score: i32) -> &'static str {
    match score {
        i32::MIN..=-1 => "Fallen Charter",
        0..=149 => "Scarred Survival",
        150..=299 => "Fragile Realm",
        300..=499 => "Enduring Realm",
        _ => "Founding Legend",
    }
}

fn strongest_identity(session: &GameSession, data: &GameData) -> String {
    if let Some(ambition_id) = &session.selected_ambition_id {
        if session.ambition_progress(data, ambition_id) >= 45 {
            if let Some(ambition) = data.campaign_balance.ambition(ambition_id) {
                return ambition.identity_tag.clone();
            }
        }
    }
    let food = session
        .settlements
        .iter()
        .map(|settlement| settlement.stored.food)
        .sum::<i32>();
    let roads = session
        .routes
        .iter()
        .filter(|route| route.level.is_built())
        .count();
    let wealth = session
        .settlements
        .iter()
        .map(|settlement| settlement.stored.wealth)
        .sum::<i32>();
    if roads >= 6 {
        "Roadbound Realm".to_owned()
    } else if wealth > 250 {
        "Merchant Realm".to_owned()
    } else if food > 350 {
        "Breadbasket Realm".to_owned()
    } else if average_loyalty_stability(session) > 65 {
        "Civic Realm".to_owned()
    } else {
        "Frontier Realm".to_owned()
    }
}

fn chronicle_years(session: &GameSession) -> (u32, u32) {
    let mut scores: HashMap<u32, i32> = HashMap::new();
    for entry in &session.chronicle {
        let value = match entry.importance.as_str() {
            "major" => 3,
            "notable" => 2,
            _ => 1,
        };
        let signed = if entry.title.contains("Lost")
            || entry.title.contains("Hunger")
            || entry.title.contains("Raid")
            || entry.title.contains("Behind")
        {
            -value
        } else {
            value
        };
        *scores.entry(entry.year).or_insert(0) += signed;
    }
    let worst = scores
        .iter()
        .min_by_key(|(_, score)| *score)
        .map(|(year, _)| *year)
        .unwrap_or(1);
    let golden = scores
        .iter()
        .max_by_key(|(_, score)| *score)
        .map(|(year, _)| *year)
        .unwrap_or(1);
    (worst, golden)
}

fn build_arc_labels(session: &GameSession) -> Vec<String> {
    let mut arcs = Vec::new();
    let mut site_mentions: HashMap<String, usize> = HashMap::new();
    for entry in &session.chronicle {
        if let Some(site_id) = &entry.site_id {
            *site_mentions.entry(site_id.clone()).or_insert(0) += 1;
        }
    }

    if session
        .chronicle
        .iter()
        .any(|entry| entry.tag == "settlement" || entry.tag == "road")
        && site_mentions.values().any(|mentions| *mentions >= 2)
    {
        arcs.push("Settlement Rise".to_owned());
    }
    if !session.completed_projects.is_empty()
        || session
            .chronicle
            .iter()
            .any(|entry| entry.tag == "ambition" || entry.tag == "project")
    {
        arcs.push("Ambition and Projects".to_owned());
    }
    if !session.active_institutions.is_empty()
        || session
            .chronicle
            .iter()
            .any(|entry| entry.tag == "institution")
    {
        arcs.push("Institutions".to_owned());
    }
    if session
        .chronicle
        .iter()
        .any(|entry| entry.tag == "crisis" || entry.tag == "wilderness")
    {
        arcs.push("Frontier Trial".to_owned());
    }
    if session
        .independent_settlements
        .iter()
        .any(|independent| independent.integration_progress > 0)
        || session
            .chronicle
            .iter()
            .any(|entry| entry.tag == "independent")
    {
        arcs.push("Integration".to_owned());
    }
    if !session.rival_faction.action_log.is_empty()
        || session.chronicle.iter().any(|entry| entry.tag == "faction")
    {
        arcs.push("Frontier Rivalry".to_owned());
    }
    arcs.dedup();
    arcs.truncate(4);
    arcs
}

fn add_unique_tag(tags: &mut Vec<String>, tag: &str) {
    if !tags.iter().any(|existing| existing == tag) {
        tags.push(tag.to_owned());
    }
}
