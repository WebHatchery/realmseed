use realmseed::data::{FactionGoal, GameData};
use realmseed::state::GameSession;

fn test_data() -> GameData {
    GameData::load().unwrap()
}

fn scout_site(session: &mut GameSession, data: &GameData, site_id: &str) {
    assert!(session.select_site(data, site_id));
    session.scout_selected_site(data).unwrap();
}

#[test]
fn new_campaign_has_rival_independents_and_wilderness() {
    let data = test_data();
    let session = GameSession::new(&data);

    assert_eq!(session.rival_faction.id, "ashthorn_clan");
    assert_eq!(session.independent_settlements.len(), 4);
    assert_eq!(session.wilderness_pressure.len(), data.regions.len());
}

#[test]
fn rival_executes_visible_action() {
    let data = test_data();
    let mut session = GameSession::new(&data);

    let report = session.advance_faction_systems(&data);
    assert_eq!(report.rival_actions, 1);
    assert_eq!(session.rival_faction.action_log.len(), 1);
}

#[test]
fn rival_pressure_changes_are_bounded() {
    let data = test_data();
    let mut session = GameSession::new(&data);
    session.settlements[0].rival_pressure = 98;
    session.execute_rival_action(&data, FactionGoal::Influence, "charter_hall");
    assert_eq!(session.settlements[0].rival_pressure, 100);

    let independent_site_id = session.independent_settlements[0].site_id.clone();
    session.independent_settlements[0].rival_pressure = 99;
    session.execute_rival_action(&data, FactionGoal::Trade, &independent_site_id);
    assert_eq!(session.independent_settlements[0].rival_pressure, 100);
}

#[test]
fn independent_trade_and_integration_progress() {
    let data = test_data();
    let mut session = GameSession::new(&data);

    scout_site(&mut session, &data, "lowmeadow");
    scout_site(&mut session, &data, "briarford");
    session.open_trade_with_selected_independent(&data).unwrap();
    {
        let independent = session
            .independent_settlements
            .iter_mut()
            .find(|state| state.site_id == "briarford")
            .unwrap();
        independent.trust = 50;
    }
    session.begin_selected_integration(&data).unwrap();
    let independent = session.selected_independent().unwrap();
    assert!(independent.integration_progress > 0);
}
