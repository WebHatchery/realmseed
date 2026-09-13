//! Complete prototype campaign flow, ambitions, projects, institutions, and endings.

use super::{
    GameSession, SeasonAdvanceReport, SeasonSummaryRow, SettlementActionStatus, SettlementStatus,
};
use crate::data::{GameData, ProjectDef, RouteLevel, SettlementTier, SiteCategory};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdvisorPriority {
    AnswerEvent,
    SpendActions,
    ScoutSelected,
    RevealFrontier,
    FoundCamp,
    BuildRoad,
    AdvanceSeason,
    ResolveIssues,
}

impl GameSession {
    pub fn advisor_priority(&self, data: &GameData) -> AdvisorPriority {
        if self.pending_event.is_some() {
            return AdvisorPriority::AnswerEvent;
        }
        if self.council_actions_remaining <= 0 {
            return AdvisorPriority::SpendActions;
        }
        if self.is_adjacent_unknown(data, &self.selected_site_id) {
            return AdvisorPriority::ScoutSelected;
        }
        if data
            .sites
            .iter()
            .any(|site| self.is_adjacent_unknown(data, &site.id))
        {
            return AdvisorPriority::RevealFrontier;
        }
        if self
            .settlements
            .iter()
            .filter(|settlement| settlement.status == SettlementStatus::Active)
            .count()
            < 2
            && data.sites.iter().any(|site| {
                site.category == SiteCategory::Settlement
                    && self.is_known(&site.id)
                    && site.owner.is_none()
                    && self.settlement_at_site(&site.id).is_none()
            })
        {
            return AdvisorPriority::FoundCamp;
        }
        if self.routes.iter().any(|route| {
            route.known
                && route.level == RouteLevel::None
                && self.route_action_status(data, &route.id).enabled
        }) {
            return AdvisorPriority::BuildRoad;
        }
        if self.active_issues.is_empty() {
            AdvisorPriority::AdvanceSeason
        } else {
            AdvisorPriority::ResolveIssues
        }
    }

    pub fn guidance_text(&self, data: &GameData) -> String {
        if self.known_site_count() <= 8 {
            return data.text("state.guidance.scouting");
        }
        if self
            .settlements
            .iter()
            .filter(|settlement| settlement.is_active())
            .count()
            < 2
        {
            return data.text("state.guidance.founding");
        }
        if !self.routes.iter().any(|route| route.level.is_built()) {
            return data.text("state.guidance.roads");
        }
        if self.pending_event.is_some() || !self.active_issues.is_empty() {
            return data.text("state.guidance.crisis");
        }
        if self
            .rival_faction
            .action_log
            .iter()
            .any(|entry| entry.action == "Independent Request")
        {
            return data.text("state.guidance.independent_request");
        }
        if self.rival_faction.action_log.is_empty() {
            return data.text("state.guidance.rival");
        }
        if self.rival_faction.action_log.len() == 1 {
            return data.text("state.guidance.rival_action");
        }
        if self.selected_ambition_id.is_none() {
            return data.text("state.guidance.ambition");
        }
        if self
            .independent_settlements
            .iter()
            .all(|independent| !independent.trade_relationship)
        {
            return data.text("state.guidance.independents");
        }
        data.text("state.guidance.default")
    }

    pub fn ambition_objective_text(&self, data: &GameData, ambition_id: &str) -> String {
        let progress = self.ambition_progress(data, ambition_id);
        match ambition_id {
            "breadbasket" => data.text_with(
                "state.objective.breadbasket",
                &[("{progress}", &progress.to_string())],
            ),
            "roadbound" => data.text_with(
                "state.objective.roadbound",
                &[("{progress}", &progress.to_string())],
            ),
            "civic" => data.text_with(
                "state.objective.civic",
                &[("{progress}", &progress.to_string())],
            ),
            _ => data.text_with(
                "state.objective.default",
                &[("{progress}", &progress.to_string())],
            ),
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
            .ok_or_else(|| data.text("state.unknown_ambition"))?;
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
            return SettlementActionStatus::disabled(data.text("state.unknown_project"));
        };
        let target_id = self.project_target_id(data, project);
        if self
            .completed_projects
            .iter()
            .any(|completed| completed.id == project.id && completed.target_id == target_id)
        {
            return SettlementActionStatus::disabled(data.text("state.project_complete"));
        }
        if project.action_cost > self.council_actions_remaining {
            return SettlementActionStatus::disabled(data.text("state.not_enough_actions"));
        }
        let Some(source) = self.project_source_settlement(data, project) else {
            return SettlementActionStatus::disabled(data.text("state.valid_project_target"));
        };
        if let Some(reason) = source.stored.deficit_text(project.cost) {
            return SettlementActionStatus::disabled(reason);
        }

        let cost = project.cost.cost_text();
        let actions = project.action_cost.to_string();
        SettlementActionStatus::enabled(data.text_with(
            "state.project_cost",
            &[
                ("{cost}", &cost),
                ("{actions}", &actions),
                ("{description}", &project.description),
            ],
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
            .ok_or_else(|| data.text("state.unknown_project"))?
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
            .ok_or_else(|| data.text("state.project_source_unavailable"))?;
        source.stored.subtract(project.cost);
        self.council_actions_remaining -= project.action_cost;
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

        Ok(data.text_with("state.project_completed", &[("{name}", &project.name)]))
    }

    pub fn institution_status(
        &self,
        data: &GameData,
        institution_id: &str,
    ) -> SettlementActionStatus {
        let Some(institution) = data.campaign_balance.institution(institution_id) else {
            return SettlementActionStatus::disabled(data.text("state.unknown_institution"));
        };
        if self
            .active_institutions
            .iter()
            .any(|active| active.id == institution.id)
        {
            return SettlementActionStatus::disabled(data.text("state.institution_active"));
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
            return SettlementActionStatus::disabled(data.text("state.unlock_requirement"));
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
            .ok_or_else(|| data.text("state.unknown_institution"))?;
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

        Ok(data.text_with(
            "state.institution_unlocked",
            &[("{name}", &institution.name)],
        ))
    }

    pub fn advance_campaign_systems(
        &mut self,
        data: &GameData,
        report: &SeasonAdvanceReport,
    ) -> CampaignAdvanceReport {
        self.apply_institution_effects();
        self.record_last_season_summary(data, report);
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
        data: &GameData,
        project: &ProjectDef,
    ) -> Option<&super::SettlementRuntimeState> {
        if project.scope == "settlement" {
            self.selected_settlement()
        } else {
            self.settlement_at_site(&data.road_balance.source_site_id)
        }
    }

    fn apply_project_effect(&mut self, _data: &GameData, project: &ProjectDef, target_id: &str) {
        if let Some(settlement) = self
            .settlements
            .iter_mut()
            .find(|settlement| settlement.location_id == target_id)
        {
            settlement.stored.add(project.effects.resource_delta);
            settlement.prosperity =
                (settlement.prosperity + project.effects.prosperity_delta).clamp(0, 100);
            settlement.stability =
                (settlement.stability + project.effects.stability_delta).clamp(0, 100);
            settlement.loyalty = (settlement.loyalty + project.effects.loyalty_delta).clamp(0, 100);
            settlement.danger = (settlement.danger + project.effects.danger_delta).clamp(0, 100);
            for tag in &project.effects.memory_tags {
                add_unique_tag(&mut settlement.memory_tags, tag);
            }
        }

        if project.effects.wilderness_pressure_delta != 0 {
            if let Some(pressure) = self
                .wilderness_pressure
                .iter_mut()
                .find(|pressure| pressure.region_id == target_id)
            {
                pressure.pressure =
                    (pressure.pressure + project.effects.wilderness_pressure_delta).clamp(0, 100);
                pressure.last_delta += project.effects.wilderness_pressure_delta;
            }
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

    fn record_last_season_summary(&mut self, data: &GameData, report: &SeasonAdvanceReport) {
        let mut rows = vec![SeasonSummaryRow {
            label: data.text("state.summary.production_label"),
            detail: data.text_with(
                "state.summary.production",
                &[
                    ("{food}", &report.produced.food.to_string()),
                    ("{timber}", &report.produced.timber.to_string()),
                    ("{stone}", &report.produced.stone.to_string()),
                    ("{wealth}", &report.produced.wealth.to_string()),
                    ("{consumed}", &report.food_consumed.to_string()),
                    ("{population}", &report.population_delta.to_string()),
                ],
            ),
            site_id: None,
            tag: "player_progress".to_owned(),
        }];
        if report.food_shortages > 0 || report.settlements_lost > 0 {
            rows.push(SeasonSummaryRow {
                label: data.text("state.summary.warnings_label"),
                detail: data.text_with(
                    "state.summary.warnings",
                    &[
                        ("{shortages}", &report.food_shortages.to_string()),
                        ("{lost}", &report.settlements_lost.to_string()),
                    ],
                ),
                site_id: report
                    .first_food_shortage_site_id
                    .clone()
                    .or_else(|| report.first_lost_site_id.clone()),
                tag: "crisis".to_owned(),
            });
        }
        rows.push(SeasonSummaryRow {
            label: data.text("state.summary.roads_label"),
            detail: data.text_with(
                "state.summary.roads",
                &[
                    ("{warnings}", &report.road_warnings.to_string()),
                    ("{isolated}", &report.isolated_settlements.to_string()),
                    ("{strain}", &report.unmanaged_strain.to_string()),
                ],
            ),
            site_id: None,
            tag: "road".to_owned(),
        });
        rows.push(SeasonSummaryRow {
            label: data.text("state.summary.issues_label"),
            detail: data.text_with(
                "state.summary.issues",
                &[
                    ("{events}", &report.events_triggered.to_string()),
                    ("{followups}", &report.issues_escalated.to_string()),
                ],
            ),
            site_id: None,
            tag: "crisis".to_owned(),
        });
        rows.push(SeasonSummaryRow {
            label: data.text("state.summary.pressure_label"),
            detail: data.text_with(
                "state.summary.pressure",
                &[
                    ("{rival}", &report.rival_actions.to_string()),
                    ("{requests}", &report.independent_requests.to_string()),
                    ("{wilderness}", &report.wilderness_changes.to_string()),
                ],
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
        let scoring = &data.campaign_balance.scoring;
        let legacy_score = controlled * scoring.controlled_settlement_points
            + towns * scoring.town_points
            + population / scoring.population_divisor
            + connected_roads * scoring.connected_road_points
            + integrated * scoring.integrated_settlement_points
            + if average_loyalty_stability(self) > scoring.high_civic_threshold {
                scoring.high_civic_bonus
            } else {
                0
            }
            - lost * scoring.lost_settlement_penalty;
        let ending_band = ending_band(legacy_score, &scoring.ending_bands).to_owned();
        let largest_settlement = self
            .settlements
            .iter()
            .max_by_key(|settlement| settlement.population)
            .map(|settlement| settlement.name.clone())
            .unwrap_or_else(|| data.text("state.charter_hall"));
        let strongest_identity = strongest_identity(self, data);
        let (worst_year, golden_year) = self.chronicle_years();
        let defining_event = self
            .chronicle
            .iter()
            .rev()
            .find(|entry| entry.importance == "major")
            .map(|entry| entry.title.clone())
            .unwrap_or_else(|| data.text("state.defining_event"));
        let arcs = build_arc_labels(self, data);
        let ambition = self
            .selected_ambition_id
            .as_deref()
            .and_then(|id| data.campaign_balance.ambition(id))
            .map(|ambition| ambition.name.clone())
            .unwrap_or_else(|| data.text("state.no_ambition"));
        let years = (data.campaign_balance.campaign_turns / 4).to_string();
        let score = legacy_score.to_string();
        let worst = worst_year.to_string();
        let golden = golden_year.to_string();
        let arc_text = arcs.join(" / ");
        let summary_text = data.text_with(
            "state.summary",
            &[
                ("{years}", &years),
                ("{ending}", &ending_band),
                ("{score}", &score),
                ("{identity}", &strongest_identity),
                ("{largest}", &largest_settlement),
                ("{worst}", &worst),
                ("{golden}", &golden),
                ("{event}", &defining_event),
                ("{ambition}", &ambition),
                ("{arcs}", &arc_text),
            ],
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

    /// Returns the deterministic worst and golden years derived from structured chronicle tags.
    pub fn chronicle_years(&self) -> (u32, u32) {
        chronicle_years(self)
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

fn ending_band(score: i32, bands: &[crate::data::EndingBandDef]) -> &str {
    bands
        .iter()
        .rev()
        .find(|band| score >= band.minimum_score)
        .map(|band| band.label.as_str())
        .unwrap_or_else(|| bands[0].label.as_str())
}

fn strongest_identity(session: &GameSession, data: &GameData) -> String {
    if let Some(ambition_id) = &session.selected_ambition_id {
        if session.ambition_progress(data, ambition_id)
            >= data.campaign_balance.scoring.ambition_identity_threshold
        {
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
    let scoring = &data.campaign_balance.scoring;
    if roads as i32 >= scoring.identity_road_minimum {
        data.text("state.identity.roadbound")
    } else if wealth > scoring.identity_wealth_minimum {
        data.text("state.identity.merchant")
    } else if food > scoring.identity_food_minimum {
        data.text("state.identity.breadbasket")
    } else if average_loyalty_stability(session) > scoring.identity_civic_minimum {
        data.text("state.identity.civic")
    } else {
        data.text("state.identity.frontier")
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
        let signed = if matches!(entry.tag.as_str(), "crisis" | "wilderness") {
            -value
        } else {
            value
        };
        *scores.entry(entry.year).or_insert(0) += signed;
    }
    let mut years: Vec<(u32, i32)> = scores.into_iter().collect();
    years.sort_by_key(|(year, score)| (*score, *year));
    let worst = years.first().map(|(year, _)| *year).unwrap_or(1);
    years.sort_by_key(|(year, score)| (-*score, *year));
    let golden = years.first().map(|(year, _)| *year).unwrap_or(1);
    (worst, golden)
}

fn build_arc_labels(session: &GameSession, data: &GameData) -> Vec<String> {
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
        arcs.push(data.text("state.arc.settlement"));
    }
    if !session.completed_projects.is_empty()
        || session
            .chronicle
            .iter()
            .any(|entry| entry.tag == "ambition" || entry.tag == "project")
    {
        arcs.push(data.text("state.arc.projects"));
    }
    if !session.active_institutions.is_empty()
        || session
            .chronicle
            .iter()
            .any(|entry| entry.tag == "institution")
    {
        arcs.push(data.text("state.arc.institutions"));
    }
    if session
        .chronicle
        .iter()
        .any(|entry| entry.tag == "crisis" || entry.tag == "wilderness")
    {
        arcs.push(data.text("state.arc.trial"));
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
        arcs.push(data.text("state.arc.integration"));
    }
    if !session.rival_faction.action_log.is_empty()
        || session.chronicle.iter().any(|entry| entry.tag == "faction")
    {
        arcs.push(data.text("state.arc.rivalry"));
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
