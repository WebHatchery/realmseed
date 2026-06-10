use super::*;
use crate::data::{GameData, RouteLevel};

fn test_data() -> GameData {
    GameData::load().unwrap()
}

fn found_two_settlements(data: &GameData) -> GameSession {
    let mut session = GameSession::new(data);
    assert!(session.select_site(data, "lowmeadow"));
    session.scout_selected_site(data).unwrap();
    session.found_selected_camp(data).unwrap();
    session.advance_season(data);
    assert!(session.select_site(data, "emberbrook"));
    session.scout_selected_site(data).unwrap();
    session.found_selected_camp(data).unwrap();
    session.advance_season(data);
    session
}

#[test]
fn starting_routes_track_authored_links() {
    let data = test_data();
    let session = GameSession::new(&data);
    let route = session
        .routes
        .iter()
        .find(|route| route.id == "road_06")
        .unwrap();

    assert_eq!(route.level, RouteLevel::None);
    assert!(!route.known);
}

#[test]
fn building_path_connects_settlement_to_capital_network() {
    let data = test_data();
    let mut session = found_two_settlements(&data);

    assert!(!session.is_site_in_capital_network(&data, "emberbrook"));
    let status = session.route_action_status(&data, "road_06");
    assert!(status.enabled, "{}", status.reason);
    session.build_or_upgrade_route(&data, "road_06").unwrap();

    assert!(session.is_site_in_capital_network(&data, "emberbrook"));
    assert_eq!(
        session
            .routes
            .iter()
            .find(|route| route.id == "road_06")
            .unwrap()
            .level,
        RouteLevel::Path
    );
}

#[test]
fn disconnected_settlement_takes_isolation_pressure() {
    let data = test_data();
    let mut session = found_two_settlements(&data);

    let report = session.advance_road_and_supply(&data);
    let settlement = session.settlement_at_site("emberbrook").unwrap();
    assert_eq!(report.isolated_settlements, 1);
    assert!(settlement
        .active_issue_ids
        .iter()
        .any(|issue| issue == "isolated"));
    assert!(settlement.autonomy_pressure > 0);
}

#[test]
fn disconnected_settlement_pressure_is_bounded() {
    let data = test_data();
    let mut session = found_two_settlements(&data);
    let settlement = session
        .settlements
        .iter_mut()
        .find(|settlement| settlement.location_id == "emberbrook")
        .unwrap();
    settlement.autonomy_pressure = 99;
    settlement.rival_pressure = 99;

    session.advance_road_and_supply(&data);

    let settlement = session.settlement_at_site("emberbrook").unwrap();
    assert_eq!(settlement.autonomy_pressure, 100);
    assert_eq!(settlement.rival_pressure, 100);
}
