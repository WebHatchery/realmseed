//! Active issue state machine, event triggering, and choice resolution.

use super::{event_candidates::EventCandidate, GameSession, SettlementActionStatus};
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
    #[serde(default)]
    pub stage: EventStage,
}

#[derive(Debug, Clone, Default)]
pub struct EventAdvanceReport {
    pub events_triggered: usize,
    pub issues_escalated: usize,
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
            .unwrap_or_else(|| data.text("event.frontier"))
    }

    pub fn event_choice_status(
        &self,
        data: &GameData,
        choice: &EventChoiceDef,
    ) -> SettlementActionStatus {
        let Some(pending) = &self.pending_event else {
            return SettlementActionStatus::disabled(data.text("event.no_pending"));
        };
        let Some(source) = self.event_resource_source(data, &pending.target_site_id) else {
            return SettlementActionStatus::disabled(data.text("event.invalid_target"));
        };
        if let Some(reason) = source
            .stored
            .deficit_text_with(choice.requirements.resources, data)
        {
            return SettlementActionStatus::disabled(reason);
        }
        if let Some(settlement) = self.settlement_at_site(&pending.target_site_id) {
            if choice
                .blocked_by_tags
                .iter()
                .any(|tag| settlement.memory_tags.contains(tag))
            {
                return SettlementActionStatus::disabled(data.text("event.blocked_memory"));
            }
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
            .ok_or_else(|| data.text("event.no_pending"))?;
        let template = data
            .event_template(&pending.template_id)
            .ok_or_else(|| data.text("event.template_missing"))?;
        let choice = template
            .choices
            .iter()
            .find(|choice| choice.id == choice_id)
            .ok_or_else(|| data.text("event.choice_unknown"))?;
        let status = self.event_choice_status(data, choice);
        if !status.enabled {
            return Err(status.reason);
        }

        self.apply_event_effects(data, &pending, choice)?;
        self.pending_event = None;

        Ok(data.text_with("event.resolved", &[("{choice}", &choice.label)]))
    }

    pub fn defer_pending_event(&mut self, data: &GameData) -> Result<String, String> {
        let pending = self
            .pending_event
            .clone()
            .ok_or_else(|| data.text("event.no_pending"))?;
        if let Some(issue) = self
            .active_issues
            .iter_mut()
            .find(|issue| issue.id == pending.issue_id)
        {
            issue.ignored_seasons += 1;
            issue.response_score = 0;
        }
        self.pending_event = None;
        Ok(data.text("event.deferred"))
    }

    pub fn force_next_event(&mut self, data: &GameData) -> Result<String, String> {
        if self.pending_event.is_some() {
            return Err(data.text("event.resolve_first"));
        }
        let family_index = (self.event_history.len() as u64 + self.campaign_seed) as usize
            % data.event_families.len().max(1);
        for offset in 0..data.event_families.len() {
            let family = &data.event_families[(family_index + offset) % data.event_families.len()];
            if let Some(mut candidate) = self.candidate_for_family(data, family) {
                let Some(template_id) = self.select_template_for_stage(
                    data,
                    family,
                    EventStage::Opening,
                    &candidate.target_site_id,
                    candidate.severity,
                    &candidate.cause,
                ) else {
                    continue;
                };
                candidate.template_id = template_id;
                self.open_event_for_candidate(data, candidate);
                return Ok(data.text("event.forced"));
            }
        }

        Err(data.text("event.no_eligible"))
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
            template_id: candidate.template_id,
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
        let event_history = self.event_history.clone();
        let current_turn = self.clock.turn;
        let campaign_seed = self.campaign_seed;

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
                let cause = data.text("event.cause.response");
                if let Some(template_id) = select_template_for_stage(TemplateSelection {
                    event_history: &event_history,
                    current_turn,
                    campaign_seed,
                    data,
                    family,
                    stage: EventStage::Resolution,
                    target_site_id: &issue.target_site_id,
                    severity: issue.severity,
                    cause: &cause,
                }) {
                    pending_events.push(make_pending_for_issue(
                        family,
                        issue,
                        &template_id,
                        EventStage::Resolution,
                        &cause,
                    ));
                }
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
                let cause = data.text("event.cause.escalation");
                if let Some(template_id) = select_template_for_stage(TemplateSelection {
                    event_history: &event_history,
                    current_turn,
                    campaign_seed,
                    data,
                    family,
                    stage: EventStage::FollowUp,
                    target_site_id: &issue.target_site_id,
                    severity: issue.severity,
                    cause: &cause,
                }) {
                    pending_events.push(make_pending_for_issue(
                        family,
                        issue,
                        &template_id,
                        EventStage::FollowUp,
                        &cause,
                    ));
                }
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
            .filter_map(|family| {
                let mut candidate = self.candidate_for_family(data, family)?;
                candidate.weight =
                    self.adjust_event_weight(data, family, candidate.weight, candidate.severity);
                candidate.template_id = self.select_template_for_stage(
                    data,
                    family,
                    EventStage::Opening,
                    &candidate.target_site_id,
                    candidate.severity,
                    &candidate.cause,
                )?;
                Some(candidate)
            })
            .max_by_key(|candidate| candidate.weight)
    }

    fn select_template_for_stage(
        &self,
        data: &GameData,
        family: &EventFamilyDef,
        stage: EventStage,
        target_site_id: &str,
        severity: i32,
        cause: &str,
    ) -> Option<String> {
        select_template_for_stage(TemplateSelection {
            event_history: &self.event_history,
            current_turn: self.clock.turn,
            campaign_seed: self.campaign_seed,
            data,
            family,
            stage,
            target_site_id,
            severity,
            cause,
        })
    }

    pub fn can_present_event(&self, data: &GameData, pending: &PendingEventRuntimeState) -> bool {
        can_present_event_from_history(data, &self.event_history, self.clock.turn, pending)
    }

    fn record_event_appearance(&mut self, pending: &PendingEventRuntimeState) {
        self.event_history.push(EventHistoryEntry {
            template_id: pending.template_id.clone(),
            family_id: pending.family_id.clone(),
            target_site_id: pending.target_site_id.clone(),
            turn: self.clock.turn,
            stage: pending.stage,
        });
    }

    fn event_resource_source(
        &self,
        data: &GameData,
        target_site_id: &str,
    ) -> Option<&super::SettlementRuntimeState> {
        self.settlement_at_site(target_site_id).or_else(|| {
            self.independent_settlements
                .iter()
                .any(|independent| independent.site_id == target_site_id)
                .then(|| self.settlement_at_site(&data.road_balance.source_site_id))
                .flatten()
        })
    }

    fn apply_event_effects(
        &mut self,
        data: &GameData,
        pending: &PendingEventRuntimeState,
        choice: &EventChoiceDef,
    ) -> Result<(), String> {
        if let Some(settlement) = self
            .settlements
            .iter_mut()
            .find(|settlement| settlement.location_id == pending.target_site_id)
        {
            apply_choice_effects_to_settlement(settlement, &choice.effects);
        } else if self
            .independent_settlements
            .iter()
            .any(|independent| independent.site_id == pending.target_site_id)
        {
            let source = self
                .settlements
                .iter_mut()
                .find(|settlement| settlement.location_id == data.road_balance.source_site_id)
                .ok_or_else(|| data.text("event.source_missing"))?;
            apply_resource_delta(&mut source.stored, choice.effects.resource_delta);
            let independent = self
                .independent_settlements
                .iter_mut()
                .find(|independent| independent.site_id == pending.target_site_id)
                .ok_or_else(|| data.text("event.target_missing"))?;
            apply_choice_effects_to_independent(independent, &choice.effects, data);
        } else {
            return Err(data.text("event.target_missing"));
        }

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

struct TemplateSelection<'a> {
    event_history: &'a [EventHistoryEntry],
    current_turn: u32,
    campaign_seed: u64,
    data: &'a GameData,
    family: &'a EventFamilyDef,
    stage: EventStage,
    target_site_id: &'a str,
    severity: i32,
    cause: &'a str,
}

fn select_template_for_stage(request: TemplateSelection<'_>) -> Option<String> {
    let TemplateSelection {
        event_history,
        current_turn,
        campaign_seed,
        data,
        family,
        stage,
        target_site_id,
        severity,
        cause,
    } = request;
    let ids = family.template_ids_for_stage(stage);
    if ids.is_empty() {
        return None;
    }

    let start = (campaign_seed as usize
        + current_turn as usize
        + event_history.len()
        + target_site_id.len()
        + severity.max(0) as usize)
        % ids.len();
    for offset in 0..ids.len() {
        let template_id = ids[(start + offset) % ids.len()];
        if data.event_template(template_id).is_none() {
            continue;
        }
        let pending = PendingEventRuntimeState {
            family_id: family.id.clone(),
            template_id: template_id.to_owned(),
            target_site_id: target_site_id.to_owned(),
            issue_id: issue_id(&family.id, target_site_id),
            severity,
            cause: cause.to_owned(),
            stage,
        };
        if can_present_event_from_history(data, event_history, current_turn, &pending) {
            return Some(template_id.to_owned());
        }
    }
    None
}

fn can_present_event_from_history(
    data: &GameData,
    event_history: &[EventHistoryEntry],
    current_turn: u32,
    pending: &PendingEventRuntimeState,
) -> bool {
    let Some(family) = data.event_family(&pending.family_id) else {
        return false;
    };
    let per_target_count = event_history
        .iter()
        .filter(|entry| {
            entry.template_id == pending.template_id
                && entry.target_site_id == pending.target_site_id
        })
        .count() as u32;
    if per_target_count >= family.max_per_target {
        return false;
    }
    let local_blocked = event_history.iter().any(|entry| {
        entry.template_id == pending.template_id
            && entry.target_site_id == pending.target_site_id
            && current_turn.saturating_sub(entry.turn) < family.local_cooldown
    });
    if local_blocked {
        return false;
    }
    let global_blocked = event_history.iter().any(|entry| {
        entry.template_id == pending.template_id
            && current_turn.saturating_sub(entry.turn) < family.global_cooldown
    });
    !global_blocked
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

fn apply_choice_effects_to_independent(
    independent: &mut super::IndependentSettlementRuntimeState,
    effects: &EventChoiceEffects,
    data: &GameData,
) {
    let fallback_trust = effects.loyalty_delta + effects.stability_delta / 2;
    let fallback_integration = effects.prosperity_delta.max(0) / 2;
    let fallback_autonomy = if effects.response_score > 0 { -2 } else { 0 };
    independent.trust =
        (independent.trust + effects.independent_trust_delta + fallback_trust).clamp(0, 100);
    independent.autonomy =
        (independent.autonomy + effects.independent_autonomy_delta + fallback_autonomy)
            .clamp(0, 100);
    independent.rival_pressure = (independent.rival_pressure
        + effects.independent_rival_pressure_delta
        + effects.danger_delta)
        .clamp(0, 100);
    independent.integration_progress = (independent.integration_progress
        + effects.independent_integration_delta
        + fallback_integration)
        .clamp(0, 100);
    if effects.response_score >= 18 {
        independent.local_need = data.text("event.answered");
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
