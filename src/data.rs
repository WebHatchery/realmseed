//! Embedded Realmseed campaign data and validation helpers.

pub mod event;
pub mod road;
pub mod settlement;

pub use event::*;
pub use road::*;
pub use settlement::*;

use macroquad_toolkit::assets::TextureConfig;
use macroquad_toolkit::data_loader::load_embedded_json;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const GAME_CONFIG_JSON: &str = include_str!("../assets/data/game_config.json");
const TEXTURE_MANIFEST_JSON: &str = include_str!("../assets/data/texture_manifest.json");
const TERRAIN_JSON: &str = include_str!("../assets/data/terrain.json");
const REGIONS_JSON: &str = include_str!("../assets/data/regions.json");
const SITES_JSON: &str = include_str!("../assets/data/sites.json");
const ROADS_JSON: &str = include_str!("../assets/data/roads.json");
const CHRONICLE_TEMPLATES_JSON: &str = include_str!("../assets/data/chronicle_templates.json");
const SETTLEMENT_BALANCE_JSON: &str = include_str!("../assets/data/settlement_balance.json");
const ROAD_BALANCE_JSON: &str = include_str!("../assets/data/road_balance.json");
const EVENT_FAMILIES_JSON: &str = include_str!("../assets/data/event_families.json");
const EVENT_TEMPLATES_JSON: &str = include_str!("../assets/data/event_templates.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub game_name: String,
    pub display_name: String,
    pub save_slot: String,
    pub version: String,
    pub world_width: usize,
    pub world_height: usize,
    pub starting_year: u32,
    pub starting_season: String,
    pub campaign_seed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapPoint {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainDef {
    pub id: String,
    pub code: String,
    pub name: String,
    pub fertility: i32,
    pub timber: i32,
    pub stone: i32,
    pub danger: i32,
    pub travel_cost: f32,
    pub color: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainMapDef {
    pub width: usize,
    pub height: usize,
    pub terrains: Vec<TerrainDef>,
    pub tiles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub label_position: MapPoint,
    pub tint: [f32; 4],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SiteCategory {
    Settlement,
    Independent,
    Landmark,
}

impl SiteCategory {
    pub fn label(self) -> &'static str {
        match self {
            Self::Settlement => "Settlement-capable",
            Self::Independent => "Independent settlement",
            Self::Landmark => "Landmark",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteDef {
    pub id: String,
    pub name: String,
    pub region_id: String,
    pub site_type: String,
    pub category: SiteCategory,
    pub position: MapPoint,
    pub traits: Vec<String>,
    pub owner: Option<String>,
    pub initially_visible: bool,
    pub description: String,
}

impl SiteDef {
    pub fn type_label(&self) -> String {
        self.site_type.replace('_', " ")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadDef {
    pub id: String,
    pub from: String,
    pub to: String,
    pub route_type: String,
    pub level: u8,
}

impl RoadDef {
    pub fn connects(&self, site_id: &str) -> bool {
        self.from == site_id || self.to == site_id
    }

    pub fn other_end<'a>(&'a self, site_id: &str) -> Option<&'a str> {
        if self.from == site_id {
            Some(&self.to)
        } else if self.to == site_id {
            Some(&self.from)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronicleTemplateDef {
    pub id: String,
    pub title: String,
    pub body: String,
    pub importance: String,
}

#[derive(Debug, Clone)]
pub struct GameData {
    pub config: GameConfig,
    pub terrain: TerrainMapDef,
    pub regions: Vec<RegionDef>,
    pub sites: Vec<SiteDef>,
    pub roads: Vec<RoadDef>,
    pub chronicle_templates: Vec<ChronicleTemplateDef>,
    pub settlement_balance: SettlementBalance,
    pub road_balance: RoadBalance,
    pub event_families: Vec<EventFamilyDef>,
    pub event_templates: Vec<EventTemplateDef>,
    pub texture_manifest: Vec<TextureConfig>,
}

impl GameData {
    pub fn load() -> Result<Self, String> {
        let data = Self {
            config: load_embedded_json(GAME_CONFIG_JSON)?,
            terrain: load_embedded_json(TERRAIN_JSON)?,
            regions: load_embedded_json(REGIONS_JSON)?,
            sites: load_embedded_json(SITES_JSON)?,
            roads: load_embedded_json(ROADS_JSON)?,
            chronicle_templates: load_embedded_json(CHRONICLE_TEMPLATES_JSON)?,
            settlement_balance: load_embedded_json(SETTLEMENT_BALANCE_JSON)?,
            road_balance: load_embedded_json(ROAD_BALANCE_JSON)?,
            event_families: load_embedded_json(EVENT_FAMILIES_JSON)?,
            event_templates: load_embedded_json(EVENT_TEMPLATES_JSON)?,
            texture_manifest: load_embedded_json(TEXTURE_MANIFEST_JSON)?,
        };
        data.validate()?;
        Ok(data)
    }

    pub fn site(&self, id: &str) -> Option<&SiteDef> {
        self.sites.iter().find(|site| site.id == id)
    }

    pub fn region(&self, id: &str) -> Option<&RegionDef> {
        self.regions.iter().find(|region| region.id == id)
    }

    pub fn chronicle_template(&self, id: &str) -> Option<&ChronicleTemplateDef> {
        self.chronicle_templates
            .iter()
            .find(|template| template.id == id)
    }

    pub fn event_family(&self, id: &str) -> Option<&EventFamilyDef> {
        self.event_families.iter().find(|family| family.id == id)
    }

    pub fn event_template(&self, id: &str) -> Option<&EventTemplateDef> {
        self.event_templates
            .iter()
            .find(|template| template.id == id)
    }

    pub fn terrain_at(&self, x: i32, y: i32) -> Option<&TerrainDef> {
        if x < 0 || y < 0 || x as usize >= self.terrain.width || y as usize >= self.terrain.height {
            return None;
        }

        let row = self.terrain.tiles.get(y as usize)?;
        let code = row.as_bytes().get(x as usize)?;
        self.terrain
            .terrains
            .iter()
            .find(|terrain| terrain.code.as_bytes().first() == Some(code))
    }

    pub fn roads_for_site<'a>(&'a self, site_id: &'a str) -> impl Iterator<Item = &'a RoadDef> {
        self.roads.iter().filter(move |road| road.connects(site_id))
    }

    fn validate(&self) -> Result<(), String> {
        if self.config.world_width != self.terrain.width
            || self.config.world_height != self.terrain.height
        {
            return Err("game_config world size must match terrain size".to_owned());
        }
        if self.terrain.tiles.len() != self.terrain.height {
            return Err("terrain row count does not match terrain height".to_owned());
        }
        for (index, row) in self.terrain.tiles.iter().enumerate() {
            if row.len() != self.terrain.width {
                return Err(format!(
                    "terrain row {} is not {} tiles",
                    index, self.terrain.width
                ));
            }
        }

        let terrain_codes: HashSet<&str> = self
            .terrain
            .terrains
            .iter()
            .map(|terrain| terrain.code.as_str())
            .collect();
        for row in &self.terrain.tiles {
            for byte in row.as_bytes() {
                let code = (*byte as char).to_string();
                if !terrain_codes.contains(code.as_str()) {
                    return Err(format!("terrain uses unknown code {}", code));
                }
            }
        }

        let site_ids: HashSet<&str> = self.sites.iter().map(|site| site.id.as_str()).collect();
        if self.sites.len() != 30 {
            return Err(format!("expected 30 sites, found {}", self.sites.len()));
        }
        if self
            .sites
            .iter()
            .filter(|site| site.category == SiteCategory::Settlement)
            .count()
            != 18
        {
            return Err("expected 18 settlement-capable sites".to_owned());
        }
        if self
            .sites
            .iter()
            .filter(|site| site.category == SiteCategory::Independent)
            .count()
            != 4
        {
            return Err("expected 4 independent settlements".to_owned());
        }
        if self
            .sites
            .iter()
            .filter(|site| site.category == SiteCategory::Landmark)
            .count()
            != 8
        {
            return Err("expected 8 landmark/resource/pass sites".to_owned());
        }
        if self
            .sites
            .iter()
            .filter(|site| site.initially_visible)
            .count()
            < 8
        {
            return Err("expected at least 8 starting visible sites".to_owned());
        }

        for road in &self.roads {
            if !site_ids.contains(road.from.as_str()) || !site_ids.contains(road.to.as_str()) {
                return Err(format!("road {} references an unknown site", road.id));
            }
        }

        self.settlement_balance.validate()?;
        self.road_balance.validate()?;
        self.validate_events()?;

        Ok(())
    }

    fn validate_events(&self) -> Result<(), String> {
        if self.event_families.len() < 6 {
            return Err("expected at least 6 event families".to_owned());
        }
        for family in &self.event_families {
            for template_id in [
                &family.opening_template_id,
                &family.followup_template_id,
                &family.resolution_template_id,
            ] {
                if self.event_template(template_id).is_none() {
                    return Err(format!(
                        "event family {} references missing template {}",
                        family.id, template_id
                    ));
                }
            }
            let chronicle_count = [
                &family.opening_chronicle_template_id,
                &family.resolution_chronicle_template_id,
            ]
            .iter()
            .filter(|template_id| self.chronicle_template(template_id).is_some())
            .count();
            if chronicle_count < 2 {
                return Err(format!(
                    "event family {} needs two chronicle templates",
                    family.id
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_data_loads_and_matches_phase_contract() {
        let data = GameData::load().unwrap();

        assert_eq!(data.config.game_name, "realmseed");
        assert_eq!(data.terrain.width, 60);
        assert_eq!(data.terrain.height, 40);
        assert_eq!(data.sites.len(), 30);
        assert_eq!(data.roads.len(), 50);
        assert_eq!(data.settlement_balance.focuses.len(), 6);
        assert_eq!(data.road_balance.road_event_issue_ids.len(), 3);
        assert_eq!(data.event_families.len(), 6);
        assert_eq!(
            data.sites
                .iter()
                .filter(|site| site.initially_visible)
                .count(),
            8
        );
    }
}
