//! Runtime campaign state, seasonal turns, scouting, and save migration.

pub mod advisor;
pub mod campaign;
pub mod event;
pub mod event_candidates;
pub mod faction;
pub mod road;
pub mod road_supply;
pub mod settlement;

pub use advisor::*;
pub use campaign::*;
pub use event::*;
pub use faction::*;
pub use road::*;
pub use settlement::*;

use crate::data::{ChronicleTemplateDef, GameData, SiteCategory, SiteDef};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    pub fn from_config(value: &str) -> Self {
        match value {
            "summer" => Self::Summer,
            "autumn" => Self::Autumn,
            "winter" => Self::Winter,
            _ => Self::Spring,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Spring => "Spring",
            Self::Summer => "Summer",
            Self::Autumn => "Autumn",
            Self::Winter => "Winter",
        }
    }

    fn next(self) -> Self {
        match self {
            Self::Spring => Self::Summer,
            Self::Summer => Self::Autumn,
            Self::Autumn => Self::Winter,
            Self::Winter => Self::Spring,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignClock {
    pub year: u32,
    pub season: Season,
    pub turn: u32,
}

impl CampaignClock {
    pub fn new(year: u32, season: Season) -> Self {
        Self {
            year: year.max(1),
            season,
            turn: 1,
        }
    }

    pub fn advance(&mut self) -> bool {
        let next = self.season.next();
        let year_advanced = self.season == Season::Winter;
        self.season = next;
        if year_advanced {
            self.year += 1;
        }
        self.turn += 1;
        year_advanced
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SiteKnowledge {
    Unknown,
    Known,
}

impl SiteKnowledge {
    pub fn label(self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::Known => "Known",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteRuntimeState {
    pub site_id: String,
    pub knowledge: SiteKnowledge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronicleEntry {
    pub year: u32,
    pub season: Season,
    pub title: String,
    pub body: String,
    pub site_id: Option<String>,
    pub importance: String,
    #[serde(default = "default_chronicle_tag")]
    pub tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonSummaryRow {
    pub label: String,
    pub detail: String,
    pub site_id: Option<String>,
    pub tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub version: String,
    pub clock: CampaignClock,
    pub selected_site_id: String,
    pub site_states: Vec<SiteRuntimeState>,
    #[serde(default)]
    pub settlements: Vec<SettlementRuntimeState>,
    #[serde(default)]
    pub routes: Vec<RouteRuntimeState>,
    #[serde(default)]
    pub regional_projects: Vec<RegionalProjectRuntimeState>,
    #[serde(default)]
    pub active_issues: Vec<ActiveIssueRuntimeState>,
    #[serde(default)]
    pub pending_event: Option<PendingEventRuntimeState>,
    #[serde(default)]
    pub event_history: Vec<EventHistoryEntry>,
    #[serde(default = "default_rival_faction")]
    pub rival_faction: FactionRuntimeState,
    #[serde(default)]
    pub independent_settlements: Vec<IndependentSettlementRuntimeState>,
    #[serde(default)]
    pub wilderness_pressure: Vec<WildernessPressureState>,
    #[serde(default)]
    pub selected_ambition_id: Option<String>,
    #[serde(default = "default_difficulty_id")]
    pub selected_difficulty_id: String,
    #[serde(default)]
    pub completed_projects: Vec<CompletedProjectState>,
    #[serde(default)]
    pub active_institutions: Vec<ActiveInstitutionState>,
    #[serde(default)]
    pub last_season_summary: String,
    #[serde(default)]
    pub last_season_rows: Vec<SeasonSummaryRow>,
    #[serde(default)]
    pub last_season_flow: LastSeasonFlow,
    #[serde(default)]
    pub endgame_summary: Option<EndgameSummary>,
    #[serde(default)]
    pub unmanaged_strain: i32,
    #[serde(default)]
    pub unmanaged_strain_seasons: i32,
    #[serde(default)]
    pub migrant_pool: i32,
    #[serde(default)]
    pub council_actions_remaining: i32,
    pub chronicle: Vec<ChronicleEntry>,
    pub campaign_seed: u64,
}

#[derive(Debug, Clone)]
pub struct GameSession {
    pub clock: CampaignClock,
    pub selected_site_id: String,
    pub site_states: Vec<SiteRuntimeState>,
    pub settlements: Vec<SettlementRuntimeState>,
    pub routes: Vec<RouteRuntimeState>,
    pub regional_projects: Vec<RegionalProjectRuntimeState>,
    pub active_issues: Vec<ActiveIssueRuntimeState>,
    pub pending_event: Option<PendingEventRuntimeState>,
    pub event_history: Vec<EventHistoryEntry>,
    pub rival_faction: FactionRuntimeState,
    pub independent_settlements: Vec<IndependentSettlementRuntimeState>,
    pub wilderness_pressure: Vec<WildernessPressureState>,
    pub selected_ambition_id: Option<String>,
    pub selected_difficulty_id: String,
    pub completed_projects: Vec<CompletedProjectState>,
    pub active_institutions: Vec<ActiveInstitutionState>,
    pub last_season_summary: String,
    pub last_season_rows: Vec<SeasonSummaryRow>,
    pub last_season_flow: LastSeasonFlow,
    pub endgame_summary: Option<EndgameSummary>,
    pub unmanaged_strain: i32,
    pub unmanaged_strain_seasons: i32,
    pub migrant_pool: i32,
    pub council_actions_remaining: i32,
    pub chronicle: Vec<ChronicleEntry>,
    pub campaign_seed: u64,
}

impl GameSession {
    pub fn new(data: &GameData) -> Self {
        let clock = CampaignClock::new(
            data.config.starting_year,
            Season::from_config(&data.config.starting_season),
        );
        let charter_site_id = data.settlement_balance.founding.source_site_id.as_str();
        let selected_site_id = data
            .sites
            .iter()
            .find(|site| site.id == charter_site_id)
            .or_else(|| data.sites.iter().find(|site| site.id == "charter_hall"))
            .or_else(|| data.sites.iter().find(|site| site.initially_visible))
            .map(|site| site.id.clone())
            .unwrap_or_default();
        let starting_site_id = selected_site_id.clone();

        let mut session = Self {
            clock,
            selected_site_id,
            site_states: data
                .sites
                .iter()
                .map(|site| SiteRuntimeState {
                    site_id: site.id.clone(),
                    knowledge: if site.id == starting_site_id {
                        SiteKnowledge::Known
                    } else {
                        SiteKnowledge::Unknown
                    },
                })
                .collect(),
            settlements: Self::create_starting_settlements(data),
            routes: Self::create_starting_routes(data),
            regional_projects: Vec::new(),
            active_issues: Vec::new(),
            pending_event: None,
            event_history: Vec::new(),
            rival_faction: Self::create_starting_rival(data),
            independent_settlements: Self::create_independent_states(data),
            wilderness_pressure: Self::create_wilderness_pressure(data),
            selected_ambition_id: None,
            selected_difficulty_id: default_difficulty_id(),
            completed_projects: Vec::new(),
            active_institutions: Vec::new(),
            last_season_summary: data.text("state.no_season_summary"),
            last_season_rows: Vec::new(),
            last_season_flow: LastSeasonFlow::default(),
            endgame_summary: None,
            unmanaged_strain: 0,
            unmanaged_strain_seasons: 0,
            migrant_pool: data.settlement_balance.starting_migrant_pool,
            council_actions_remaining: data.settlement_balance.council_actions_per_season,
            chronicle: Vec::new(),
            campaign_seed: data.config.campaign_seed,
        };
        session.refresh_route_knowledge();
        session.add_chronicle_entry(data, "campaign_start", Some(&starting_site_id));
        session
    }

    pub fn from_save(save: SaveData, data: &GameData) -> Self {
        let mut session = Self {
            clock: save.clock,
            selected_site_id: save.selected_site_id,
            site_states: save.site_states,
            settlements: save.settlements,
            routes: save.routes,
            regional_projects: save.regional_projects,
            active_issues: save.active_issues,
            pending_event: save.pending_event,
            event_history: save.event_history,
            rival_faction: save.rival_faction,
            independent_settlements: save.independent_settlements,
            wilderness_pressure: save.wilderness_pressure,
            selected_ambition_id: save.selected_ambition_id,
            selected_difficulty_id: save.selected_difficulty_id,
            completed_projects: save.completed_projects,
            active_institutions: save.active_institutions,
            last_season_summary: save.last_season_summary,
            last_season_rows: save.last_season_rows,
            last_season_flow: save.last_season_flow,
            endgame_summary: save.endgame_summary,
            unmanaged_strain: save.unmanaged_strain,
            unmanaged_strain_seasons: save.unmanaged_strain_seasons,
            migrant_pool: save.migrant_pool,
            council_actions_remaining: save.council_actions_remaining,
            chronicle: save.chronicle,
            campaign_seed: save.campaign_seed,
        };
        session.ensure_phase_2_defaults(data);
        session
    }

    pub fn to_save(&self, version: &str) -> SaveData {
        SaveData {
            version: version.to_owned(),
            clock: self.clock.clone(),
            selected_site_id: self.selected_site_id.clone(),
            site_states: self.site_states.clone(),
            settlements: self.settlements.clone(),
            routes: self.routes.clone(),
            regional_projects: self.regional_projects.clone(),
            active_issues: self.active_issues.clone(),
            pending_event: self.pending_event.clone(),
            event_history: self.event_history.clone(),
            rival_faction: self.rival_faction.clone(),
            independent_settlements: self.independent_settlements.clone(),
            wilderness_pressure: self.wilderness_pressure.clone(),
            selected_ambition_id: self.selected_ambition_id.clone(),
            selected_difficulty_id: self.selected_difficulty_id.clone(),
            completed_projects: self.completed_projects.clone(),
            active_institutions: self.active_institutions.clone(),
            last_season_summary: self.last_season_summary.clone(),
            last_season_rows: self.last_season_rows.clone(),
            last_season_flow: self.last_season_flow.clone(),
            endgame_summary: self.endgame_summary.clone(),
            unmanaged_strain: self.unmanaged_strain,
            unmanaged_strain_seasons: self.unmanaged_strain_seasons,
            migrant_pool: self.migrant_pool,
            council_actions_remaining: self.council_actions_remaining,
            chronicle: self.chronicle.clone(),
            campaign_seed: self.campaign_seed,
        }
    }

    pub fn selected_site<'a>(&self, data: &'a GameData) -> Option<&'a SiteDef> {
        data.site(&self.selected_site_id)
    }

    pub fn site_knowledge(&self, site_id: &str) -> SiteKnowledge {
        self.site_states
            .iter()
            .find(|state| state.site_id == site_id)
            .map(|state| state.knowledge)
            .unwrap_or(SiteKnowledge::Unknown)
    }

    pub fn is_known(&self, site_id: &str) -> bool {
        self.site_knowledge(site_id) == SiteKnowledge::Known
    }

    pub fn is_adjacent_unknown(&self, data: &GameData, site_id: &str) -> bool {
        if self.is_known(site_id) {
            return false;
        }

        data.roads_for_site(site_id)
            .filter_map(|road| road.other_end(site_id))
            .any(|neighbor_id| self.is_known(neighbor_id))
    }

    pub fn can_select_site(&self, data: &GameData, site_id: &str) -> bool {
        data.site(site_id).is_some()
            && (self.is_known(site_id) || self.is_adjacent_unknown(data, site_id))
    }

    pub fn select_site(&mut self, data: &GameData, site_id: &str) -> bool {
        if self.can_select_site(data, site_id) {
            self.selected_site_id = site_id.to_owned();
            true
        } else {
            false
        }
    }

    pub fn scout_status(&self, data: &GameData) -> SettlementActionStatus {
        let Some(site) = self.selected_site(data) else {
            return SettlementActionStatus::disabled(data.text("state.select_scout_target"));
        };

        if self.is_known(&site.id) {
            return SettlementActionStatus::disabled(data.text("state.site_already_known"));
        }
        if !self.is_adjacent_unknown(data, &site.id) {
            return SettlementActionStatus::disabled(data.text("state.scout_adjacent_first"));
        }

        let scouting = &data.settlement_balance.scouting;
        if self.council_actions_remaining < scouting.action_cost {
            let actions = scouting.action_cost.to_string();
            return SettlementActionStatus::disabled(
                data.text_with("state.needs_council_action", &[("{actions}", &actions)]),
            );
        }
        let source_site_id = &data.settlement_balance.founding.source_site_id;
        let Some(source) = self
            .settlements
            .iter()
            .find(|settlement| settlement.location_id == *source_site_id && settlement.is_active())
        else {
            return SettlementActionStatus::disabled(data.text("state.capital_unavailable"));
        };
        if let Some(reason) = source.stored.deficit_text_with(scouting.cost, data) {
            return SettlementActionStatus::disabled(reason);
        }

        let source_name = data
            .site(source_site_id)
            .map(|site| site.name.clone())
            .unwrap_or_else(|| data.text("state.charter_capital"));
        let actions = scouting.action_cost.to_string();
        let cost = scouting.cost.cost_text_with(data);
        SettlementActionStatus::enabled(data.text_with(
            "state.scout_cost",
            &[
                ("{actions}", &actions),
                ("{cost}", &cost),
                ("{source}", &source_name),
            ],
        ))
    }

    pub fn scout_selected_site(&mut self, data: &GameData) -> Result<String, String> {
        let status = self.scout_status(data);
        if !status.enabled {
            return Err(status.reason);
        }

        let site_id = self.selected_site_id.clone();
        let site_name = data
            .site(&site_id)
            .map(|site| site.name.clone())
            .unwrap_or_else(|| data.text("state.site"));
        let source_site_id = &data.settlement_balance.founding.source_site_id;
        let source_index = self
            .settlements
            .iter()
            .position(|settlement| {
                settlement.location_id == *source_site_id && settlement.is_active()
            })
            .ok_or_else(|| data.text("state.capital_unavailable"))?;
        self.settlements[source_index]
            .stored
            .subtract(data.settlement_balance.scouting.cost);
        self.council_actions_remaining -= data.settlement_balance.scouting.action_cost;
        if let Some(state) = self
            .site_states
            .iter_mut()
            .find(|state| state.site_id == site_id)
        {
            state.knowledge = SiteKnowledge::Known;
        }
        self.refresh_route_knowledge();
        self.add_chronicle_entry(data, "site_scouted", Some(&site_id));
        let report = data
            .site(&site_id)
            .map(|site| self.scout_report_for_site(data, site))
            .unwrap_or_else(|| data.text("state.scout_report_unavailable"));
        Ok(data.text_with(
            "state.scouted",
            &[("{site}", &site_name), ("{report}", &report)],
        ))
    }

    fn scout_report_for_site(&self, data: &GameData, site: &SiteDef) -> String {
        let terrain = data.terrain_at(site.position.x, site.position.y);
        let terrain_name = terrain
            .map(|terrain| terrain.name.clone())
            .unwrap_or_else(|| data.text("state.unknown_ground"));
        let region_name = data
            .region(&site.region_id)
            .map(|region| region.name.clone())
            .unwrap_or_else(|| data.text("state.frontier"));
        let route_count = data.roads_for_site(&site.id).count();
        let route_text = if route_count == 1 {
            data.text("state.one_route")
        } else {
            let count = route_count.to_string();
            data.text_with("state.routes", &[("{count}", &count)])
        };
        let traits = if site.traits.is_empty() {
            data.text("state.no_traits")
        } else {
            site.traits
                .iter()
                .take(2)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        };

        match site.category {
            SiteCategory::Settlement => {
                let prospect = terrain
                    .map(|terrain| {
                        strongest_terrain_prospect(
                            data,
                            terrain.fertility,
                            terrain.timber,
                            terrain.stone,
                            terrain.danger,
                        )
                    })
                    .unwrap_or_else(|| data.text("state.uncertain_prospects"));
                data.text_with(
                    "state.scout_settlement_report",
                    &[
                        ("{terrain}", &terrain_name),
                        ("{region}", &region_name),
                        ("{routes}", &route_text),
                        ("{traits}", &traits),
                        ("{prospect}", &prospect),
                    ],
                )
            }
            SiteCategory::Independent => {
                if let Some(independent) = self
                    .independent_settlements
                    .iter()
                    .find(|independent| independent.site_id == site.id)
                {
                    let trust = independent.trust.to_string();
                    let autonomy = independent.autonomy.to_string();
                    let pressure = independent.rival_pressure.to_string();
                    data.text_with(
                        "state.scout_independent_report",
                        &[
                            ("{region}", &region_name),
                            ("{trust}", &trust),
                            ("{autonomy}", &autonomy),
                            ("{pressure}", &pressure),
                            ("{need}", &independent.local_need),
                        ],
                    )
                } else {
                    data.text_with(
                        "state.scout_independent_brief",
                        &[
                            ("{region}", &region_name),
                            ("{routes}", &route_text),
                            ("{traits}", &traits),
                        ],
                    )
                }
            }
            SiteCategory::Landmark => data.text_with(
                "state.scout_landmark_report",
                &[
                    ("{terrain}", &terrain_name),
                    ("{routes}", &route_text),
                    ("{traits}", &traits),
                    ("{description}", &site.description),
                ],
            ),
        }
    }

    pub fn advance_season(&mut self, data: &GameData) -> SeasonAdvanceReport {
        let mut report = self.advance_settlement_economy(data);
        let road_report = self.advance_road_and_supply(data);
        report.isolated_settlements = road_report.isolated_settlements;
        report.road_warnings = road_report.road_warnings;
        report.unmanaged_strain = road_report.unmanaged_strain;
        let event_report = self.advance_event_chains(data);
        report.events_triggered = event_report.events_triggered;
        report.issues_escalated = event_report.issues_escalated;
        let faction_report = self.advance_faction_systems(data);
        report.rival_actions = faction_report.rival_actions;
        report.independent_requests = faction_report.independent_requests;
        report.wilderness_changes = faction_report.wilderness_changes;
        let campaign_report = self.advance_campaign_systems(data, &report);
        report.campaign_finished = campaign_report.campaign_finished;
        if self.clock.advance() {
            self.add_chronicle_entry(data, "new_year", None);
        }
        report
    }

    pub fn known_site_count(&self) -> usize {
        self.site_states
            .iter()
            .filter(|state| state.knowledge == SiteKnowledge::Known)
            .count()
    }

    fn add_chronicle_entry(&mut self, data: &GameData, template_id: &str, site_id: Option<&str>) {
        let Some(template) = data.chronicle_template(template_id) else {
            return;
        };
        self.chronicle
            .push(self.render_chronicle_entry(data, template, site_id));
    }

    fn render_chronicle_entry(
        &self,
        data: &GameData,
        template: &ChronicleTemplateDef,
        site_id: Option<&str>,
    ) -> ChronicleEntry {
        let site = site_id.and_then(|id| data.site(id));
        let region_name = site
            .and_then(|site_def| data.region(&site_def.region_id))
            .map(|region| region.name.clone())
            .unwrap_or_else(|| data.text("state.frontier"));
        let site_name = site
            .map(|site_def| site_def.name.clone())
            .unwrap_or_else(|| data.text("state.charter_map"));

        ChronicleEntry {
            year: self.clock.year,
            season: self.clock.season,
            title: fill_template(
                &template.title,
                self.clock.year,
                self.clock.season,
                &site_name,
                &region_name,
            ),
            body: fill_template(
                &template.body,
                self.clock.year,
                self.clock.season,
                &site_name,
                &region_name,
            ),
            site_id: site_id.map(str::to_owned),
            importance: template.importance.clone(),
            tag: template.tag.clone(),
        }
    }

    fn ensure_phase_2_defaults(&mut self, data: &GameData) {
        if self.settlements.is_empty() {
            self.settlements = Self::create_starting_settlements(data);
            self.migrant_pool = data.settlement_balance.starting_migrant_pool;
            self.council_actions_remaining = data.settlement_balance.council_actions_per_season;
        }
        if self.routes.is_empty() {
            self.routes = Self::create_starting_routes(data);
            self.refresh_route_knowledge();
        }
        if self.rival_faction.id.is_empty() {
            self.rival_faction = Self::create_starting_rival(data);
        }
        if self.independent_settlements.is_empty() {
            self.independent_settlements = Self::create_independent_states(data);
        }
        if self.wilderness_pressure.is_empty() {
            self.wilderness_pressure = Self::create_wilderness_pressure(data);
        }
        if self.last_season_summary.is_empty() {
            self.last_season_summary = data.text("state.no_season_summary");
        }
        if self.selected_difficulty_id.is_empty() {
            self.selected_difficulty_id = default_difficulty_id();
        }
    }
}

fn default_chronicle_tag() -> String {
    "memory".to_owned()
}

fn default_difficulty_id() -> String {
    "frontier".to_owned()
}

fn default_rival_faction() -> FactionRuntimeState {
    FactionRuntimeState {
        id: String::new(),
        name: String::new(),
        personality: String::new(),
        description: String::new(),
        confidence: 0,
        fear: 0,
        hostility: 0,
        border_pressure: 0,
        recent_losses: 0,
        current_goal: crate::data::FactionGoal::Expand,
        goal_age: 0,
        action_cooldowns: Vec::new(),
        controlled_locations: Vec::new(),
        known_targets: Vec::new(),
        memory_tags: Vec::new(),
        action_log: Vec::new(),
    }
}

fn strongest_terrain_prospect(
    data: &GameData,
    fertility: i32,
    timber: i32,
    stone: i32,
    danger: i32,
) -> String {
    let best_yield = fertility.max(timber).max(stone);
    if danger >= best_yield + 2 {
        data.text("state.prospect.hazards")
    } else if fertility >= timber && fertility >= stone {
        data.text("state.prospect.food")
    } else if timber >= stone {
        data.text("state.prospect.timber")
    } else {
        data.text("state.prospect.stone")
    }
}

fn fill_template(
    text: &str,
    year: u32,
    season: Season,
    site_name: &str,
    region_name: &str,
) -> String {
    text.replace("{year}", &year.to_string())
        .replace("{season}", season.label())
        .replace("{site}", site_name)
        .replace("{region}", region_name)
}

pub fn migrate_save_value(
    detected_version: Option<String>,
    value: Value,
    data: &GameData,
) -> Result<SaveData, String> {
    const SUPPORTED_LEGACY_VERSIONS: &[&str] = &["0.1.0"];
    if let Some(version) = detected_version.as_deref() {
        if version != data.config.version && !SUPPORTED_LEGACY_VERSIONS.contains(&version) {
            return Err(data.text_with(
                "state.migrate.unsupported",
                &[("{version}", version), ("{current}", &data.config.version)],
            ));
        }
    }

    let payload = value.get("data").cloned().unwrap_or(value);
    let version = detected_version
        .as_deref()
        .map(|version| format!(" version `{version}`"))
        .unwrap_or_default();
    let mut current = serde_json::from_value::<SaveData>(payload).map_err(|error| {
        data.text_with(
            "state.migrate.read",
            &[("{version}", &version), ("{error}", &error.to_string())],
        )
    })?;
    current.version = data.config.version.clone();
    Ok(current)
}
