//! Runtime campaign state, seasonal turns, scouting, and save migration.

pub mod road;
pub mod settlement;

pub use road::*;
pub use settlement::*;

use crate::data::{ChronicleTemplateDef, GameData, SiteDef};
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
        let selected_site_id = data
            .sites
            .iter()
            .find(|site| site.id == "charter_hall")
            .or_else(|| data.sites.iter().find(|site| site.initially_visible))
            .map(|site| site.id.clone())
            .unwrap_or_default();

        let mut session = Self {
            clock,
            selected_site_id,
            site_states: data
                .sites
                .iter()
                .map(|site| SiteRuntimeState {
                    site_id: site.id.clone(),
                    knowledge: if site.initially_visible {
                        SiteKnowledge::Known
                    } else {
                        SiteKnowledge::Unknown
                    },
                })
                .collect(),
            settlements: Self::create_starting_settlements(data),
            routes: Self::create_starting_routes(data),
            regional_projects: Vec::new(),
            unmanaged_strain: 0,
            unmanaged_strain_seasons: 0,
            migrant_pool: data.settlement_balance.starting_migrant_pool,
            council_actions_remaining: data.settlement_balance.council_actions_per_season,
            chronicle: Vec::new(),
            campaign_seed: data.config.campaign_seed,
        };
        session.add_chronicle_entry(data, "campaign_start", Some("charter_hall"));
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

    pub fn can_scout_selected_site(&self, data: &GameData) -> bool {
        self.is_adjacent_unknown(data, &self.selected_site_id)
    }

    pub fn scout_selected_site(&mut self, data: &GameData) -> bool {
        if !self.can_scout_selected_site(data) {
            return false;
        }

        let site_id = self.selected_site_id.clone();
        if let Some(state) = self
            .site_states
            .iter_mut()
            .find(|state| state.site_id == site_id)
        {
            state.knowledge = SiteKnowledge::Known;
        }
        self.refresh_route_knowledge();
        self.add_chronicle_entry(data, "site_scouted", Some(&site_id));
        true
    }

    pub fn advance_season(&mut self, data: &GameData) -> SeasonAdvanceReport {
        let mut report = self.advance_settlement_economy(data);
        let road_report = self.advance_road_and_supply(data);
        report.isolated_settlements = road_report.isolated_settlements;
        report.road_warnings = road_report.road_warnings;
        report.unmanaged_strain = road_report.unmanaged_strain;
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
            .map(|region| region.name.as_str())
            .unwrap_or("the frontier");
        let site_name = site
            .map(|site_def| site_def.name.as_str())
            .unwrap_or("the charter map");

        ChronicleEntry {
            year: self.clock.year,
            season: self.clock.season,
            title: fill_template(
                &template.title,
                self.clock.year,
                self.clock.season,
                site_name,
                region_name,
            ),
            body: fill_template(
                &template.body,
                self.clock.year,
                self.clock.season,
                site_name,
                region_name,
            ),
            site_id: site_id.map(str::to_owned),
            importance: template.importance.clone(),
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
    _detected_version: Option<String>,
    value: Value,
    data: &GameData,
) -> Result<SaveData, String> {
    let payload = value.get("data").cloned().unwrap_or(value);

    if let Ok(mut current) = serde_json::from_value::<SaveData>(payload) {
        current.version = data.config.version.clone();
        return Ok(current);
    }

    Ok(GameSession::new(data).to_save(&data.config.version))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_data() -> GameData {
        GameData::load().unwrap()
    }

    #[test]
    fn new_campaign_starts_with_eight_known_sites_and_chronicle() {
        let data = test_data();
        let session = GameSession::new(&data);

        assert_eq!(session.known_site_count(), 8);
        assert_eq!(session.selected_site_id, "charter_hall");
        assert_eq!(session.clock.season, Season::Spring);
        assert_eq!(session.clock.year, 1);
        assert_eq!(session.chronicle.len(), 1);
    }

    #[test]
    fn scouting_reveals_adjacent_unknown_sites() {
        let data = test_data();
        let mut session = GameSession::new(&data);

        assert!(session.select_site(&data, "old_king_road"));
        assert!(session.can_scout_selected_site(&data));
        assert!(session.scout_selected_site(&data));
        assert!(session.is_known("old_king_road"));
        assert_eq!(session.known_site_count(), 9);
        assert_eq!(session.chronicle.len(), 2);
    }

    #[test]
    fn seasons_cycle_and_year_advances_after_winter() {
        let data = test_data();
        let mut session = GameSession::new(&data);

        session.advance_season(&data);
        assert_eq!(session.clock.season, Season::Summer);
        session.advance_season(&data);
        assert_eq!(session.clock.season, Season::Autumn);
        session.advance_season(&data);
        assert_eq!(session.clock.season, Season::Winter);
        session.advance_season(&data);
        assert_eq!(session.clock.season, Season::Spring);
        assert_eq!(session.clock.year, 2);
    }
}
