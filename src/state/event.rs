//! Active issue state machine, event triggering, and choice resolution.

use super::{GameSession, SettlementActionStatus, SettlementStatus};
use crate::data::{
    ActiveIssueState, EventChoiceDef, EventChoiceEffects, EventFamilyDef, EventStage,
    EventTemplateDef, GameData, ResourceStock,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveIssueRuntimeState {
    pub id: String,
    pub family_id: String,
    pub target_site_id: String,
    pub state: ActiveIssueState,
    pub severity: i32,
    pub age_seasons: u32,
    pub ignored_seasons: u32,
    pub last_player_response: Option<String>,
    pub escalation_threshold: i32,
    pub improvement_threshold: i32,
    pub cooldown_remaining: u32,
    pub response_score: i32,
    pub memory_tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingEventRuntimeState {
    pub family_id: String,
    pub template_id: String,
    pub target_site_id: String,
    pub issue_id: String,
    pub severity: i32,
    pub cause: String,
    pub stage: EventStage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventHistoryEntry {
    pub template_id: String,
    pub family_id: String,
    pub target_site_id: String,
    pub turn: u32,
}

#[derive(Debug, Clone, Default)]
pub struct EventAdvanceReport {
    pub events_triggered: usize,
    pub issues_escalated: usize,
}

#[derive(Debug, Clone)]
struct EventCandidate {
    family_id: String,
    target_site_id: String,
    severity: i32,
    weight: i32,
    cause: String,
}

impl GameSession {
    pub fn pending_event_template<'a>(&self, data: &'a GameData) -> Option<&'a EventTemplateDef> {
        let pending = self.pending_event.as_ref()?;
        data.event_template(&pending.template_id)
    }

    pub fn pending_event_site_name(&self, data: &GameData) -> String {
        self.pending_event
            .as_ref()
            .and_then(|event| data.site(&event.target_site_id))
            .map(|site| site.name.clone())
            .unwrap_or_else(|| "the frontier".to_owned())
    }

    pub fn event_choice_status(&self, choice: &EventChoiceDef) -> SettlementActionStatus {
        let Some(pending) = &self.pending_event else {
            return SettlementActionStatus::disabled("No event is pending.");
        };
        let Some(settlement) = self.settlement_at_site(&pending.target_site_id) else {
            return SettlementActionStatus::disabled("This event has no settlement target.");
        };
        if let Some(reason) = settlement
            .stored
            .deficit_text(choice.requirements.resources)
        {
            return SettlementActionStatus::disabled(reason);
        }
        if choice
            .blocked_by_tags
            .iter()
            .any(|tag| settlement.memory_tags.contains(tag))
        {
            return SettlementActionStatus::disabled("Blocked by settlement memory.");
        }

        SettlementActionStatus::enabled(choice.visible_consequence.clone())
    }

    pub fn resolve_pending_event_choice(
        &mut self,
        data: &GameData,
        choice_id: &str,
    ) -> Result<String, String> {
        let pending = self
            .pending_event
            .clone()
            .ok_or_else(|| "No event is pending.".to_owned())?;
        let template = data
            .event_template(&pending.template_id)
            .ok_or_else(|| "Pending event template is missing.".to_owned())?;
        let choice = template
            .choices
            .iter()
            .find(|choice| choice.id == choice_id)
            .ok_or_else(|| "Unknown event choice.".to_owned())?;
        let status = self.event_choice_status(choice);
        if !status.enabled {
            return Err(status.reason);
        }

        self.apply_event_effects(data, &pending, choice)?;
        self.pending_event = None;

        Ok(format!("Resolved event choice: {}", choice.label))
    }

    pub fn defer_pending_event(&mut self) -> Result<String, String> {
        let pending = self
            .pending_event
            .clone()
            .ok_or_else(|| "No event is pending.".to_owned())?;
        if let Some(issue) = self
            .active_issues
            .iter_mut()
            .find(|issue| issue.id == pending.issue_id)
        {
            issue.ignored_seasons += 1;
            issue.response_score = 0;
        }
        self.pending_event = None;
        Ok("Deferred event; the issue will keep aging.".to_owned())
    }

    pub fn force_next_event(&mut self, data: &GameData) -> Result<String, String> {
        if self.pending_event.is_some() {
            return Err("Resolve or defer the current event first.".to_owned());
        }
        let family_index = (self.event_history.len() as u64 + self.campaign_seed) as usize
            % data.event_families.len().max(1);
        for offset in 0..data.event_families.len() {
            let family = &data.event_families[(family_index + offset) % data.event_families.len()];
            if let Some(candidate) = self.candidate_for_family(data, family) {
                self.open_event_for_candidate(data, candidate);
                return Ok("Forced next eligible event.".to_owned());
            }
        }

        Err("No eligible event target found.".to_owned())
    }

    pub fn advance_event_chains(&mut self, data: &GameData) -> EventAdvanceReport {
        if self.pending_event.is_some() {
            return EventAdvanceReport::default();
        }

        let mut report = EventAdvanceReport::default();
        let queued = self.age_active_issues(data);
        if let Some(pending) = queued.into_iter().next() {
            self.record_event_appearance(&pending);
            self.pending_event = Some(pending);
            report.events_triggered += 1;
            report.issues_escalated += 1;
            return report;
        }

        let Some(candidate) = self.best_event_candidate(data) else {
            return report;
        };
        self.open_event_for_candidate(data, candidate);
        report.events_triggered += 1;
        report
    }

    fn open_event_for_candidate(&mut self, data: &GameData, candidate: EventCandidate) {
        let Some(family) = data.event_family(&candidate.family_id) else {
            return;
        };
        let issue_id = issue_id(&family.id, &candidate.target_site_id);
        if !self.active_issues.iter().any(|issue| issue.id == issue_id) {
            self.active_issues.push(ActiveIssueRuntimeState {
                id: issue_id.clone(),
                family_id: family.id.clone(),
                target_site_id: candidate.target_site_id.clone(),
                state: ActiveIssueState::Warning,
                severity: candidate.severity.clamp(1, 3),
                age_seasons: 0,
                ignored_seasons: 0,
                last_player_response: None,
                escalation_threshold: 35,
                improvement_threshold: 18,
                cooldown_remaining: 0,
                response_score: 0,
                memory_tags: family.memory_tags.clone(),
            });
        }

        let pending = PendingEventRuntimeState {
            family_id: family.id.clone(),
            template_id: family.opening_template_id.clone(),
            target_site_id: candidate.target_site_id,
            issue_id,
            severity: candidate.severity,
            cause: candidate.cause,
            stage: EventStage::Opening,
        };
        self.record_event_appearance(&pending);
        self.pending_event = Some(pending);
    }

    fn age_active_issues(&mut self, data: &GameData) -> Vec<PendingEventRuntimeState> {
        let mut pending_events = Vec::new();
        let risk_by_site: Vec<(String, i32)> = self
            .settlements
            .iter()
            .map(|settlement| {
                (
                    settlement.location_id.clone(),
                    (100 - settlement.stability).max(0) / 8
                        + (100 - settlement.loyalty).max(0) / 10
                        + settlement.autonomy_pressure,
                )
            })
            .collect();
        let unmanaged_strain = self.unmanaged_strain;

        for issue in &mut self.active_issues {
            if matches!(
                issue.state,
                ActiveIssueState::Dormant | ActiveIssueState::Resolution
            ) {
                issue.cooldown_remaining = issue.cooldown_remaining.saturating_sub(1);
                continue;
            }
            if issue.state == ActiveIssueState::Collapse {
                continue;
            }

            issue.age_seasons += 1;
            let Some(family) = data.event_family(&issue.family_id) else {
                continue;
            };
            if issue.response_score >= issue.improvement_threshold {
                issue.state = ActiveIssueState::Resolution;
                pending_events.push(make_pending_for_issue(
                    family,
                    issue,
                    &family.resolution_template_id,
                    EventStage::Resolution,
                    "Player response improved the issue.",
                ));
                issue.response_score = 0;
                continue;
            }

            issue.ignored_seasons += 1;
            let escalation_score = issue.severity * 10
                + issue.ignored_seasons as i32 * 8
                + unmanaged_strain * 3
                + risk_by_site
                    .iter()
                    .find(|(site_id, _)| site_id == &issue.target_site_id)
                    .map(|(_, risk)| *risk)
                    .unwrap_or(0);
            if escalation_score >= issue.escalation_threshold {
                issue.state = match issue.state {
                    ActiveIssueState::Warning => ActiveIssueState::Active,
                    ActiveIssueState::Active => ActiveIssueState::Escalating,
                    ActiveIssueState::Escalating if issue.severity >= 3 => {
                        ActiveIssueState::Collapse
                    }
                    ActiveIssueState::Escalating => ActiveIssueState::Escalating,
                    other => other,
                };
                if issue.state == ActiveIssueState::Escalating {
                    issue.severity = (issue.severity + 1).clamp(1, 3);
                }
                pending_events.push(make_pending_for_issue(
                    family,
                    issue,
                    &family.followup_template_id,
                    EventStage::FollowUp,
                    "The active issue escalated after neglect.",
                ));
            }
            issue.response_score = (issue.response_score - 4).max(0);
        }

        pending_events
            .into_iter()
            .filter(|pending| self.can_present_event(data, pending))
            .collect()
    }

    fn best_event_candidate(&self, data: &GameData) -> Option<EventCandidate> {
        data.event_families
            .iter()
            .filter_map(|family| self.candidate_for_family(data, family))
            .filter(|candidate| {
                let Some(family) = data.event_family(&candidate.family_id) else {
                    return false;
                };
                let pending = PendingEventRuntimeState {
                    family_id: family.id.clone(),
                    template_id: family.opening_template_id.clone(),
                    target_site_id: candidate.target_site_id.clone(),
                    issue_id: issue_id(&family.id, &candidate.target_site_id),
                    severity: candidate.severity,
                    cause: candidate.cause.clone(),
                    stage: EventStage::Opening,
                };
                self.can_present_event(data, &pending)
            })
            .max_by_key(|candidate| candidate.weight)
    }

    fn candidate_for_family(
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
            _ => None,
        }
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
                target_site_id: route.site_a.clone(),
                severity: if route.condition == super::RouteCondition::Blocked {
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
                target_site_id: settlement.location_id.clone(),
                severity: 1 + ((70 - settlement.loyalty).max(0) / 20).clamp(0, 2),
                weight: 35 + (70 - settlement.loyalty).max(0) + settlement.autonomy_pressure,
                cause: "Low loyalty or autonomy pressure made civic unrest likely.".to_owned(),
            })
            .max_by_key(|candidate| candidate.weight)
    }

    fn can_present_event(&self, data: &GameData, pending: &PendingEventRuntimeState) -> bool {
        let Some(family) = data.event_family(&pending.family_id) else {
            return false;
        };
        let per_target_count = self
            .event_history
            .iter()
            .filter(|entry| {
                entry.template_id == pending.template_id
                    && entry.target_site_id == pending.target_site_id
            })
            .count() as u32;
        if per_target_count >= family.max_per_target {
            return false;
        }
        let current_turn = self.clock.turn;
        let local_blocked = self.event_history.iter().any(|entry| {
            entry.template_id == pending.template_id
                && entry.target_site_id == pending.target_site_id
                && current_turn.saturating_sub(entry.turn) < family.local_cooldown
        });
        if local_blocked {
            return false;
        }
        let global_blocked = self.event_history.iter().any(|entry| {
            entry.template_id == pending.template_id
                && current_turn.saturating_sub(entry.turn) < family.global_cooldown
        });
        !global_blocked
    }

    fn record_event_appearance(&mut self, pending: &PendingEventRuntimeState) {
        self.event_history.push(EventHistoryEntry {
            template_id: pending.template_id.clone(),
            family_id: pending.family_id.clone(),
            target_site_id: pending.target_site_id.clone(),
            turn: self.clock.turn,
        });
    }

    fn apply_event_effects(
        &mut self,
        data: &GameData,
        pending: &PendingEventRuntimeState,
        choice: &EventChoiceDef,
    ) -> Result<(), String> {
        let settlement = self
            .settlements
            .iter_mut()
            .find(|settlement| settlement.location_id == pending.target_site_id)
            .ok_or_else(|| "Event target settlement is missing.".to_owned())?;
        apply_choice_effects_to_settlement(settlement, &choice.effects);

        if let Some(issue) = self
            .active_issues
            .iter_mut()
            .find(|issue| issue.id == pending.issue_id)
        {
            issue.last_player_response = Some(choice.id.clone());
            issue.response_score += choice.effects.response_score;
            issue.severity = (issue.severity + choice.effects.severity_delta).clamp(1, 3);
            if let Some(next_state) = choice.effects.issue_state {
                issue.state = next_state;
            }
            if choice.effects.close_issue {
                issue.state = ActiveIssueState::Dormant;
                issue.cooldown_remaining = data
                    .event_family(&issue.family_id)
                    .map(|family| family.local_cooldown)
                    .unwrap_or(8);
            }
            for tag in &choice.effects.memory_tags {
                if !issue.memory_tags.contains(tag) {
                    issue.memory_tags.push(tag.clone());
                }
            }
        }

        if let Some(template_id) = &choice.effects.chronicle_template_id {
            self.add_chronicle_entry(data, template_id, Some(&pending.target_site_id));
        }

        Ok(())
    }
}

fn issue_id(family_id: &str, target_site_id: &str) -> String {
    format!("{}:{}", family_id, target_site_id)
}

fn make_pending_for_issue(
    family: &EventFamilyDef,
    issue: &ActiveIssueRuntimeState,
    template_id: &str,
    stage: EventStage,
    cause: &str,
) -> PendingEventRuntimeState {
    PendingEventRuntimeState {
        family_id: family.id.clone(),
        template_id: template_id.to_owned(),
        target_site_id: issue.target_site_id.clone(),
        issue_id: issue.id.clone(),
        severity: issue.severity,
        cause: cause.to_owned(),
        stage,
    }
}

fn apply_choice_effects_to_settlement(
    settlement: &mut super::SettlementRuntimeState,
    effects: &EventChoiceEffects,
) {
    apply_resource_delta(&mut settlement.stored, effects.resource_delta);
    settlement.population = (settlement.population + effects.population_delta).max(0);
    settlement.stability = (settlement.stability + effects.stability_delta).clamp(0, 100);
    settlement.loyalty = (settlement.loyalty + effects.loyalty_delta).clamp(0, 100);
    settlement.prosperity = (settlement.prosperity + effects.prosperity_delta).clamp(0, 100);
    settlement.danger = (settlement.danger + effects.danger_delta).clamp(0, 100);
    for tag in &effects.memory_tags {
        if !settlement.memory_tags.contains(tag) {
            settlement.memory_tags.push(tag.clone());
        }
    }
}

fn apply_resource_delta(resources: &mut ResourceStock, delta: ResourceStock) {
    resources.food = (resources.food + delta.food).max(0);
    resources.timber = (resources.timber + delta.timber).max(0);
    resources.stone = (resources.stone + delta.stone).max(0);
    resources.wealth = (resources.wealth + delta.wealth).max(0);
}

pub fn fill_event_text(text: &str, site_name: &str) -> String {
    text.replace("{site}", site_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::GameSession;

    fn test_data() -> GameData {
        GameData::load().unwrap()
    }

    #[test]
    fn event_trigger_creates_pending_event_and_active_issue() {
        let data = test_data();
        let mut session = GameSession::new(&data);

        let report = session.advance_event_chains(&data);
        assert_eq!(report.events_triggered, 1);
        assert!(session.pending_event.is_some());
        assert_eq!(session.active_issues.len(), 1);
    }

    #[test]
    fn event_choice_applies_memory_and_can_resolve_chain() {
        let data = test_data();
        let mut session = GameSession::new(&data);

        session.force_next_event(&data).unwrap();
        let template = session.pending_event_template(&data).unwrap();
        let choice_id = template.choices[0].id.clone();
        session
            .resolve_pending_event_choice(&data, &choice_id)
            .unwrap();
        assert!(session.pending_event.is_none());
        assert!(!session.event_history.is_empty());
        assert!(session
            .settlements
            .iter()
            .any(|settlement| !settlement.memory_tags.is_empty()));
    }

    #[test]
    fn repetition_control_blocks_same_template_for_same_target() {
        let data = test_data();
        let mut session = GameSession::new(&data);

        session.force_next_event(&data).unwrap();
        let first = session.pending_event.clone().unwrap();
        session.pending_event = None;
        assert!(!session.can_present_event(&data, &first));
    }
}
