//! Event candidate scoring and strategic weighting.

use super::{GameSession, RouteCondition, SettlementStatus};
use crate::data::{EventFamilyDef, GameData};

#[derive(Debug, Clone)]
pub(super) struct EventCandidate {
    pub(super) family_id: String,
    pub(super) template_id: String,
    pub(super) target_site_id: String,
    pub(super) severity: i32,
    pub(super) weight: i32,
    pub(super) cause: String,
}

impl GameSession {
    pub(super) fn candidate_for_family(
        &self,
        data: &GameData,
        family: &EventFamilyDef,
    ) -> Option<EventCandidate> {
        match family.trigger_kind.as_str() {
            "low_food" => self.low_food_candidate(family),
            "road_warning" => self.road_warning_candidate(family),
            "isolated" => self.isolation_candidate(data, family),
            "trade_pressure" => self.trade_pressure_candidate(family),
            "migration_pressure" => self.migration_candidate(family),
            "low_loyalty" => self.low_loyalty_candidate(family),
            "bandit_pressure" => self.bandit_pressure_candidate(data, family),
            "border_tension" => self.border_tension_candidate(data, family),
            "tax_dispute" => self.tax_dispute_candidate(family),
            "local_leadership" => self.local_leadership_candidate(family),
            "resource_boom" => self.resource_boom_candidate(family),
            "disaster" => self.disaster_candidate(data, family),
            "independent_request" => self.independent_request_candidate(data, family),
            "rival_move" => self.rival_move_candidate(data, family),
            _ => None,
        }
    }

    pub(super) fn adjust_event_weight(
        &self,
        data: &GameData,
        family: &EventFamilyDef,
        weight: i32,
        severity: i32,
    ) -> i32 {
        let difficulty = data
            .campaign_balance
            .difficulty(&self.selected_difficulty_id)
            .map(|preset| preset.event_weight_modifier)
            .unwrap_or(0);
        weight + difficulty + self.ambition_event_weight(family) + severity
    }

    fn low_food_candidate(&self, family: &EventFamilyDef) -> Option<EventCandidate> {
        self.settlements
            .iter()
            .filter(|settlement| settlement.status == SettlementStatus::Active)
            .filter(|settlement| settlement.stored.food < settlement.population * 2)
            .map(|settlement| {
                let severity = if settlement.stored.food < settlement.population / 2 {
                    2
                } else {
                    1
                };
                EventCandidate {
                    family_id: family.id.clone(),
                    template_id: String::new(),
                    target_site_id: settlement.location_id.clone(),
                    severity,
                    weight: 40 + severity * 10 - settlement.stored.food / 10,
                    cause: "Food stores are below two seasons of population need.".to_owned(),
                }
            })
            .max_by_key(|candidate| candidate.weight)
    }

    fn road_warning_candidate(&self, family: &EventFamilyDef) -> Option<EventCandidate> {
        self.routes
            .iter()
            .filter(|route| !route.active_warning_ids.is_empty())
            .map(|route| EventCandidate {
                family_id: family.id.clone(),
                template_id: String::new(),
                target_site_id: route.site_a.clone(),
                severity: if route.condition == RouteCondition::Blocked {
                    2
                } else {
                    1
                },
                weight: 55 + route.active_warning_ids.len() as i32 * 5,
                cause: "A route warning is active.".to_owned(),
            })
            .max_by_key(|candidate| candidate.weight)
    }

    fn isolation_candidate(
        &self,
        data: &GameData,
        family: &EventFamilyDef,
    ) -> Option<EventCandidate> {
        self.settlements
            .iter()
            .filter(|settlement| settlement.status == SettlementStatus::Active)
            .filter(|settlement| settlement.location_id != data.road_balance.source_site_id)
            .filter(|settlement| !self.is_site_in_capital_network(data, &settlement.location_id))
            .map(|settlement| EventCandidate {
                family_id: family.id.clone(),
                template_id: String::new(),
                target_site_id: settlement.location_id.clone(),
                severity: 1 + (settlement.autonomy_pressure / 4).clamp(0, 2),
                weight: 45 + settlement.autonomy_pressure * 2,
                cause: "No built, unblocked road reaches the capital network.".to_owned(),
            })
            .max_by_key(|candidate| candidate.weight)
    }

    fn trade_pressure_candidate(&self, family: &EventFamilyDef) -> Option<EventCandidate> {
        self.settlements
            .iter()
            .filter(|settlement| settlement.status == SettlementStatus::Active)
            .filter(|settlement| settlement.focus_id == "trade" || settlement.prosperity >= 55)
            .map(|settlement| EventCandidate {
                family_id: family.id.clone(),
                template_id: String::new(),
                target_site_id: settlement.location_id.clone(),
                severity: 1,
                weight: 30 + settlement.prosperity / 2,
                cause: "Prosperity and trade have created local ambition.".to_owned(),
            })
            .max_by_key(|candidate| candidate.weight)
    }

    fn migration_candidate(&self, family: &EventFamilyDef) -> Option<EventCandidate> {
        self.settlements
            .iter()
            .filter(|settlement| settlement.status == SettlementStatus::Active)
            .filter(|settlement| {
                settlement.stored.food > settlement.population && settlement.stability > 55
            })
            .map(|settlement| EventCandidate {
                family_id: family.id.clone(),
                template_id: String::new(),
                target_site_id: settlement.location_id.clone(),
                severity: 1,
                weight: 32 + settlement.stability / 2,
                cause: "Food security and order attracted migrant families.".to_owned(),
            })
            .max_by_key(|candidate| candidate.weight)
    }

    fn low_loyalty_candidate(&self, family: &EventFamilyDef) -> Option<EventCandidate> {
        self.settlements
            .iter()
            .filter(|settlement| settlement.status == SettlementStatus::Active)
            .filter(|settlement| {
                settlement.loyalty <= 68
                    || settlement.autonomy_pressure > 0
                    || settlement
                        .active_issue_ids
                        .iter()
                        .any(|issue| issue == "isolated")
            })
            .map(|settlement| EventCandidate {
                family_id: family.id.clone(),
                template_id: String::new(),
                target_site_id: settlement.location_id.clone(),
                severity: 1 + ((70 - settlement.loyalty).max(0) / 20).clamp(0, 2),
                weight: 35 + (70 - settlement.loyalty).max(0) + settlement.autonomy_pressure,
                cause: "Low loyalty or autonomy pressure made civic unrest likely.".to_owned(),
            })
            .max_by_key(|candidate| candidate.weight)
    }

    fn bandit_pressure_candidate(
        &self,
        data: &GameData,
        family: &EventFamilyDef,
    ) -> Option<EventCandidate> {
        self.settlements
            .iter()
            .filter(|settlement| settlement.status == SettlementStatus::Active)
            .map(|settlement| {
                let wilderness = wilderness_for_site(self, data, &settlement.location_id);
                EventCandidate {
                    family_id: family.id.clone(),
                    template_id: String::new(),
                    target_site_id: settlement.location_id.clone(),
                    severity: 1 + (wilderness / 35).clamp(0, 2),
                    weight: settlement.danger + wilderness / 2,
                    cause: "Wilderness and local danger are giving bandits room to move."
                        .to_owned(),
                }
            })
            .max_by_key(|candidate| candidate.weight)
            .filter(|candidate| candidate.weight >= 40)
    }

    fn border_tension_candidate(
        &self,
        data: &GameData,
        family: &EventFamilyDef,
    ) -> Option<EventCandidate> {
        self.settlements
            .iter()
            .filter(|settlement| settlement.status == SettlementStatus::Active)
            .map(|settlement| EventCandidate {
                family_id: family.id.clone(),
                template_id: String::new(),
                target_site_id: settlement.location_id.clone(),
                severity: 1 + (settlement.rival_pressure / 30).clamp(0, 2),
                weight: target_weakness_for_event(self, data, &settlement.location_id)
                    + self.rival_faction.border_pressure / 2,
                cause: "Rival pressure is testing a weak border settlement.".to_owned(),
            })
            .max_by_key(|candidate| candidate.weight)
            .filter(|candidate| candidate.weight >= 35)
    }

    fn tax_dispute_candidate(&self, family: &EventFamilyDef) -> Option<EventCandidate> {
        self.settlements
            .iter()
            .filter(|settlement| settlement.status == SettlementStatus::Active)
            .filter(|settlement| settlement.prosperity >= 50 || settlement.loyalty <= 70)
            .map(|settlement| EventCandidate {
                family_id: family.id.clone(),
                template_id: String::new(),
                target_site_id: settlement.location_id.clone(),
                severity: 1 + ((70 - settlement.loyalty).max(0) / 25).clamp(0, 2),
                weight: settlement.prosperity / 2 + (80 - settlement.loyalty).max(0),
                cause: "Prosperity and charter dues have raised a tax dispute.".to_owned(),
            })
            .max_by_key(|candidate| candidate.weight)
    }

    fn local_leadership_candidate(&self, family: &EventFamilyDef) -> Option<EventCandidate> {
        self.settlements
            .iter()
            .filter(|settlement| settlement.status == SettlementStatus::Active)
            .map(|settlement| EventCandidate {
                family_id: family.id.clone(),
                template_id: String::new(),
                target_site_id: settlement.location_id.clone(),
                severity: 1,
                weight: 30 + settlement.prosperity / 3 + settlement.loyalty / 4,
                cause: "Local leaders are asking for a clearer place in the realm.".to_owned(),
            })
            .max_by_key(|candidate| candidate.weight)
    }

    fn resource_boom_candidate(&self, family: &EventFamilyDef) -> Option<EventCandidate> {
        self.settlements
            .iter()
            .filter(|settlement| settlement.status == SettlementStatus::Active)
            .map(|settlement| EventCandidate {
                family_id: family.id.clone(),
                template_id: String::new(),
                target_site_id: settlement.location_id.clone(),
                severity: 1,
                weight: 20
                    + settlement.prosperity / 2
                    + (settlement.stored.timber + settlement.stored.stone) / 20,
                cause: "Stored materials and local prosperity have created a boom.".to_owned(),
            })
            .max_by_key(|candidate| candidate.weight)
            .filter(|candidate| candidate.weight >= 45)
    }

    fn disaster_candidate(
        &self,
        data: &GameData,
        family: &EventFamilyDef,
    ) -> Option<EventCandidate> {
        self.settlements
            .iter()
            .filter(|settlement| settlement.status == SettlementStatus::Active)
            .map(|settlement| {
                let wilderness = wilderness_for_site(self, data, &settlement.location_id);
                EventCandidate {
                    family_id: family.id.clone(),
                    template_id: String::new(),
                    target_site_id: settlement.location_id.clone(),
                    severity: 1 + ((settlement.danger + wilderness) / 70).clamp(0, 2),
                    weight: 25 + settlement.danger / 2 + wilderness / 2,
                    cause: "Terrain danger and seasonal pressure threaten a local disaster."
                        .to_owned(),
                }
            })
            .max_by_key(|candidate| candidate.weight)
            .filter(|candidate| candidate.weight >= 45)
    }

    fn independent_request_candidate(
        &self,
        _data: &GameData,
        family: &EventFamilyDef,
    ) -> Option<EventCandidate> {
        let independent = self
            .independent_settlements
            .iter()
            .max_by_key(|independent| independent.rival_pressure + independent.autonomy)?;
        Some(EventCandidate {
            family_id: family.id.clone(),
            template_id: String::new(),
            target_site_id: independent.site_id.clone(),
            severity: 1 + (independent.rival_pressure / 40).clamp(0, 2),
            weight: 25 + independent.rival_pressure + (100 - independent.trust) / 3,
            cause: "An independent settlement needs aid, trade, or protection.".to_owned(),
        })
    }

    fn rival_move_candidate(
        &self,
        data: &GameData,
        family: &EventFamilyDef,
    ) -> Option<EventCandidate> {
        self.settlements
            .iter()
            .filter(|settlement| settlement.status == SettlementStatus::Active)
            .map(|settlement| EventCandidate {
                family_id: family.id.clone(),
                template_id: String::new(),
                target_site_id: settlement.location_id.clone(),
                severity: 1 + (self.rival_faction.hostility / 35).clamp(0, 2),
                weight: target_weakness_for_event(self, data, &settlement.location_id)
                    + self.rival_faction.hostility / 2
                    + self.rival_faction.confidence / 3,
                cause: "The rival clan is converting pressure into a visible move.".to_owned(),
            })
            .max_by_key(|candidate| candidate.weight)
    }

    fn ambition_event_weight(&self, family: &EventFamilyDef) -> i32 {
        match self.selected_ambition_id.as_deref() {
            Some("breadbasket")
                if matches!(
                    family.id.as_str(),
                    "food_crisis" | "migration_wave" | "resource_boom"
                ) =>
            {
                18
            }
            Some("roadbound")
                if matches!(
                    family.id.as_str(),
                    "road_trouble" | "bandit_pressure" | "border_tension"
                ) =>
            {
                18
            }
            Some("civic")
                if matches!(
                    family.id.as_str(),
                    "settlement_unrest"
                        | "tax_dispute"
                        | "local_leadership"
                        | "independent_request"
                ) =>
            {
                18
            }
            _ => 0,
        }
    }
}

fn wilderness_for_site(session: &GameSession, data: &GameData, site_id: &str) -> i32 {
    data.site(site_id)
        .and_then(|site| {
            session
                .wilderness_pressure
                .iter()
                .find(|pressure| pressure.region_id == site.region_id)
        })
        .map(|pressure| pressure.pressure)
        .unwrap_or(0)
}

fn target_weakness_for_event(session: &GameSession, data: &GameData, site_id: &str) -> i32 {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::GameSession;

    fn test_data() -> GameData {
        GameData::load().unwrap()
    }

    #[test]
    fn independent_request_candidate_targets_independent_site() {
        let data = test_data();
        let mut session = GameSession::new(&data);
        let expected_site_id = session.independent_settlements[0].site_id.clone();
        session.independent_settlements[0].rival_pressure = 80;

        let family = data.event_family("independent_request").unwrap();
        let candidate = session.candidate_for_family(&data, family).unwrap();

        assert_eq!(candidate.target_site_id, expected_site_id);
        assert!(session
            .independent_settlements
            .iter()
            .any(|independent| independent.site_id == candidate.target_site_id));
    }
}
