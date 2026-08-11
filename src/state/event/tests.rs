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
fn independent_request_choice_updates_independent_target() {
    let data = test_data();
    let mut session = GameSession::new(&data);
    let target_site_id = session.independent_settlements[0].site_id.clone();
    let issue_id = issue_id("independent_request", &target_site_id);
    let trust_before = session.independent_settlements[0].trust;
    let pressure_before = session.independent_settlements[0].rival_pressure;
    let source_wealth_before = session
        .settlement_at_site(&data.road_balance.source_site_id)
        .unwrap()
        .stored
        .wealth;
    session.pending_event = Some(PendingEventRuntimeState {
        family_id: "independent_request".to_owned(),
        template_id: "independent_request_opening_a".to_owned(),
        target_site_id: target_site_id.clone(),
        issue_id: issue_id.clone(),
        severity: 2,
        cause: "Independent regression test.".to_owned(),
        stage: EventStage::Opening,
    });
    session.active_issues.push(ActiveIssueRuntimeState {
        id: issue_id,
        family_id: "independent_request".to_owned(),
        target_site_id: target_site_id.clone(),
        state: ActiveIssueState::Warning,
        severity: 2,
        age_seasons: 0,
        ignored_seasons: 0,
        last_player_response: None,
        escalation_threshold: 35,
        improvement_threshold: 18,
        cooldown_remaining: 0,
        response_score: 0,
        memory_tags: Vec::new(),
    });

    session
        .resolve_pending_event_choice(&data, "answer_independent_request")
        .unwrap();

    let independent = session
        .independent_settlements
        .iter()
        .find(|independent| independent.site_id == target_site_id)
        .unwrap();
    let source_wealth_after = session
        .settlement_at_site(&data.road_balance.source_site_id)
        .unwrap()
        .stored
        .wealth;
    assert!(independent.trust > trust_before);
    assert!(independent.rival_pressure < pressure_before);
    assert!(source_wealth_after < source_wealth_before);
    assert!(session.pending_event.is_none());
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
