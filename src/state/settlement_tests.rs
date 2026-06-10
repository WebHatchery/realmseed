use super::*;
use crate::data::{GameData, SettlementTier};

fn test_data() -> GameData {
    GameData::load().unwrap()
}

fn scout_site(session: &mut GameSession, data: &GameData, site_id: &str) {
    assert!(session.select_site(data, site_id));
    session.scout_selected_site(data).unwrap();
}

#[test]
fn new_campaign_has_a_capital_settlement() {
    let data = test_data();
    let session = GameSession::new(&data);
    let capital = session.settlement_at_site("charter_hall").unwrap();

    assert_eq!(capital.tier, SettlementTier::Village);
    assert_eq!(capital.population, 140);
    assert_eq!(session.council_actions_remaining, 2);
}

#[test]
fn founding_camp_spends_resources_population_and_action() {
    let data = test_data();
    let mut session = GameSession::new(&data);

    scout_site(&mut session, &data, "lowmeadow");
    assert!(session.founding_status(&data).enabled);
    session.found_selected_camp(&data).unwrap();

    let capital = session.settlement_at_site("charter_hall").unwrap();
    let camp = session.settlement_at_site("lowmeadow").unwrap();
    assert_eq!(capital.population, 80);
    assert_eq!(capital.stored.timber, 95);
    assert_eq!(camp.population, 60);
    assert_eq!(session.council_actions_remaining, 0);
    assert!(session
        .chronicle
        .iter()
        .any(|entry| entry.site_id.as_deref() == Some("lowmeadow")));
}

#[test]
fn invalid_upgrade_explains_missing_requirements() {
    let data = test_data();
    let mut session = GameSession::new(&data);

    scout_site(&mut session, &data, "lowmeadow");
    session.found_selected_camp(&data).unwrap();
    session.advance_season(&data);

    let status = session.upgrade_status(&data);
    assert!(!status.enabled);
    assert!(status.reason.contains("population"));
}

#[test]
fn famine_can_collapse_neglected_settlement() {
    let data = test_data();
    let mut session = GameSession::new(&data);

    scout_site(&mut session, &data, "lowmeadow");
    session.found_selected_camp(&data).unwrap();
    let camp = session
        .settlements
        .iter_mut()
        .find(|settlement| settlement.location_id == "lowmeadow")
        .unwrap();
    camp.focus_id = "quarrying".to_owned();
    camp.stored.food = 0;
    camp.stability = 24;
    camp.loyalty = 22;

    for _ in 0..6 {
        session.advance_settlement_economy(&data);
    }

    let camp = session.settlement_at_site("lowmeadow").unwrap();
    assert_eq!(camp.status, SettlementStatus::Lost);
    assert!(session
        .chronicle
        .iter()
        .any(|entry| entry.title.contains("Abandoned")));
}
