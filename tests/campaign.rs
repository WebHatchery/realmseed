use realmseed::data::{EventStage, GameData, RouteLevel, SiteCategory};
use realmseed::state::{ChronicleEntry, GameSession, Season, SeasonAdvanceReport};
use std::collections::HashSet;

fn test_data() -> GameData {
    GameData::load().unwrap()
}

fn resolve_pending_with_first_choice(session: &mut GameSession, data: &GameData) {
    if let Some(template) = session.pending_event_template(data) {
        if let Some(choice) = template.choices.first() {
            let choice_id = choice.id.clone();
            if session
                .resolve_pending_event_choice(data, &choice_id)
                .is_err()
            {
                let _ = session.defer_pending_event();
            }
        }
    }
}

fn resolve_pending_with_compromise(session: &mut GameSession, data: &GameData) {
    if let Some(template) = session.pending_event_template(data) {
        if let Some(choice) = template.choices.last() {
            let choice_id = choice.id.clone();
            if session
                .resolve_pending_event_choice(data, &choice_id)
                .is_err()
            {
                let _ = session.defer_pending_event();
            }
        }
    }
}

fn perform_replay_actions(session: &mut GameSession, data: &GameData) {
    if session.pending_event.is_some() {
        resolve_pending_with_first_choice(session, data);
    }

    if session.council_actions_remaining == 0 {
        return;
    }

    let known_settlement = data.sites.iter().find(|site| {
        site.category == SiteCategory::Settlement
            && session.is_known(&site.id)
            && session.settlement_at_site(&site.id).is_none()
    });
    if let Some(site) = known_settlement {
        let site_id = site.id.clone();
        session.select_site(data, &site_id);
        if session.founding_status(data).enabled {
            session.found_selected_camp(data).unwrap();
            return;
        }
    }

    let route_to_upgrade = session
        .routes
        .iter()
        .find(|route| route.level == RouteLevel::Path && route.known)
        .map(|route| route.id.clone());
    if let Some(route_id) = route_to_upgrade {
        if session.route_action_status(data, &route_id).enabled {
            session.build_or_upgrade_route(data, &route_id).unwrap();
            return;
        }
    }

    let next_site = data
        .sites
        .iter()
        .find(|site| session.is_adjacent_unknown(data, &site.id))
        .map(|site| site.id.clone());
    if let Some(site_id) = next_site {
        session.select_site(data, &site_id);
        if session.scout_status(data).enabled {
            session.scout_selected_site(data).unwrap();
        }
    }
}

fn deterministic_replay(data: &GameData) -> Vec<String> {
    let mut session = GameSession::new(data);
    session.select_ambition(data, "civic").unwrap();
    let mut frames = Vec::new();

    for _ in 0..80 {
        if session.pending_event.is_some() {
            resolve_pending_with_compromise(&mut session, data);
        }
        perform_replay_actions(&mut session, data);
        let report = session.advance_season(data);
        frames.push(
            serde_json::to_string(&(
                session.clock.clone(),
                report.food_shortages,
                report.settlements_lost,
                report.road_warnings,
                report.events_triggered,
                report.issues_escalated,
                report.rival_actions,
                report.independent_requests,
                report.wilderness_changes,
                session.to_save("replay"),
            ))
            .unwrap(),
        );
    }

    assert!(session
        .event_history
        .iter()
        .any(|entry| entry.stage == EventStage::FollowUp));
    assert!(!session.rival_faction.action_log.is_empty());
    assert!(session
        .settlements
        .iter()
        .any(|settlement| settlement.population > 60));
    assert!(session
        .routes
        .iter()
        .any(|route| route.level == RouteLevel::Road));
    assert!(session.endgame_summary.is_some());
    frames
}

fn scout_next_unknown(session: &mut GameSession, data: &GameData) {
    let site_id = data
        .sites
        .iter()
        .find(|site| session.is_adjacent_unknown(data, &site.id))
        .map(|site| site.id.clone())
        .expect("an adjacent unknown site should be available");
    assert!(session.select_site(data, &site_id));
    if !session.scout_status(data).enabled
        && session.scout_status(data).reason.contains("council action")
    {
        session.advance_season(data);
    }
    session.scout_selected_site(data).unwrap();
}

#[test]
fn ambition_project_and_institution_flow_work() {
    let data = test_data();
    let mut session = GameSession::new(&data);
    session.select_ambition(&data, "breadbasket").unwrap();
    session.complete_project(&data, "expand_granaries").unwrap();

    assert!(session.ambition_progress(&data, "breadbasket") > 0);
    assert!(session
        .ambition_objective_text(&data, "breadbasket")
        .contains("Objective"));
}

#[test]
fn campaign_generates_endgame_summary_at_turn_limit() {
    let data = test_data();
    let mut session = GameSession::new(&data);
    session.clock.turn = 80;
    let report = SeasonAdvanceReport::default();
    let campaign_report = session.advance_campaign_systems(&data, &report);

    assert!(campaign_report.campaign_finished);
    assert!(session.endgame_summary.is_some());
}

#[test]
fn thirty_turn_validation_matches_complete_prototype_criteria() {
    let data = test_data();
    let mut session = GameSession::new(&data);
    session.select_ambition(&data, "roadbound").unwrap();

    for _ in 0..5 {
        scout_next_unknown(&mut session, &data);
    }
    assert!(session.select_site(&data, "lowmeadow"));
    session.found_selected_camp(&data).unwrap();
    let frontier = session
        .settlements
        .iter_mut()
        .find(|settlement| settlement.location_id == "lowmeadow")
        .unwrap();
    frontier.loyalty = 24;
    frontier.stability = 28;
    frontier.defence = 6;
    frontier.rival_pressure = 25;

    for _ in 0..30 {
        if let Some(pending) = session.pending_event.clone() {
            match pending.stage {
                EventStage::Opening => {
                    let _ = session.defer_pending_event();
                }
                EventStage::FollowUp | EventStage::Resolution => {
                    resolve_pending_with_first_choice(&mut session, &data);
                }
            }
        }
        session.advance_season(&data);
    }

    let distinct_chronicle_sites = session
        .chronicle
        .iter()
        .filter_map(|entry| entry.site_id.as_deref())
        .collect::<HashSet<_>>();
    let followup_issues = session
        .event_history
        .iter()
        .filter(|entry| entry.stage == EventStage::FollowUp)
        .map(|entry| (&entry.family_id, &entry.target_site_id))
        .collect::<HashSet<_>>();
    let resolution_count = session
        .event_history
        .iter()
        .filter(|entry| entry.stage == EventStage::Resolution)
        .count();
    let weakness_response = session.rival_faction.action_log.iter().any(|entry| {
        entry.reason.contains("low loyalty")
            || entry.reason.contains("weak defence")
            || entry.result.contains("weak supply")
            || entry.result.contains("supply road")
    });

    assert!(distinct_chronicle_sites.len() >= 5);
    assert!(followup_issues.len() >= 3);
    assert!(resolution_count >= 2);
    assert!(weakness_response);
    assert!(session.last_season_rows.len() >= 4);
    assert!(session
        .last_season_rows
        .iter()
        .any(|row| row.tag == "player_progress"));
}

#[test]
fn full_campaign_validation_reaches_end_summary_with_real_arcs() {
    let data = test_data();
    let mut session = GameSession::new(&data);
    session.select_ambition(&data, "civic").unwrap();

    for _ in 0..80 {
        resolve_pending_with_first_choice(&mut session, &data);
        session.advance_season(&data);
    }

    let summary = session.endgame_summary.as_ref().unwrap();
    assert!(summary.arcs.len() >= 2);
    assert!(summary.summary_text.contains("Worst year"));
    assert!(summary.summary_text.contains("golden year"));
    assert!(!summary.defining_event.is_empty());
}

#[test]
fn deterministic_action_replay_covers_the_major_campaign_systems() {
    let data = test_data();
    let first = deterministic_replay(&data);
    let second = deterministic_replay(&data);

    assert_eq!(first, second);
}

#[test]
fn chronicle_years_use_structured_tags_and_stable_ties() {
    let data = test_data();
    let mut session = GameSession::new(&data);
    session.chronicle = vec![
        ChronicleEntry {
            year: 2,
            season: Season::Spring,
            title: "A loss".to_owned(),
            body: String::new(),
            site_id: None,
            importance: "major".to_owned(),
            tag: "crisis".to_owned(),
        },
        ChronicleEntry {
            year: 1,
            season: Season::Spring,
            title: "Another loss".to_owned(),
            body: String::new(),
            site_id: None,
            importance: "major".to_owned(),
            tag: "crisis".to_owned(),
        },
        ChronicleEntry {
            year: 5,
            season: Season::Spring,
            title: "A gain".to_owned(),
            body: String::new(),
            site_id: None,
            importance: "major".to_owned(),
            tag: "settlement".to_owned(),
        },
        ChronicleEntry {
            year: 4,
            season: Season::Spring,
            title: "Another gain".to_owned(),
            body: String::new(),
            site_id: None,
            importance: "major".to_owned(),
            tag: "settlement".to_owned(),
        },
    ];

    assert_eq!(session.chronicle_years(), (1, 4));
}
