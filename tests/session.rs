use realmseed::data::GameData;
use realmseed::state::migrate_save_value;
use realmseed::state::{GameSession, Season};
use serde_json::json;

fn test_data() -> GameData {
    GameData::load().unwrap()
}

#[test]
fn new_campaign_starts_with_only_the_charter_site_and_chronicle() {
    let data = test_data();
    let session = GameSession::new(&data);

    assert_eq!(session.known_site_count(), 1);
    assert_eq!(session.selected_site_id, "charter_hall");
    assert_eq!(session.clock.season, Season::Spring);
    assert_eq!(session.clock.year, 1);
    assert_eq!(session.chronicle.len(), 1);
}

#[test]
fn scouting_reveals_adjacent_unknown_sites() {
    let data = test_data();
    let mut session = GameSession::new(&data);
    let starting_stores = session.settlement_at_site("charter_hall").unwrap().stored;
    let scout_cost = data.settlement_balance.scouting.cost;

    assert!(session.select_site(&data, "old_king_road"));
    assert!(session.scout_status(&data).enabled);
    let message = session.scout_selected_site(&data).unwrap();
    assert!(message.starts_with("Scouted Old King Road."));
    assert!(message.contains("Scout report:"));
    assert!(session.is_known("old_king_road"));
    assert_eq!(session.known_site_count(), 2);
    let capital = session.settlement_at_site("charter_hall").unwrap();
    assert_eq!(capital.stored.food, starting_stores.food - scout_cost.food);
    assert_eq!(
        capital.stored.wealth,
        starting_stores.wealth - scout_cost.wealth
    );
    assert_eq!(
        session.council_actions_remaining,
        data.settlement_balance.council_actions_per_season
            - data.settlement_balance.scouting.action_cost
    );
    assert_eq!(session.chronicle.len(), 2);
}

#[test]
fn scouting_is_limited_by_council_actions() {
    let data = test_data();
    let mut session = GameSession::new(&data);

    for site_id in ["old_king_road", "lowmeadow"] {
        assert!(session.select_site(&data, site_id));
        session.scout_selected_site(&data).unwrap();
    }

    assert_eq!(session.council_actions_remaining, 0);
    assert!(session.select_site(&data, "redford_crossing"));
    let status = session.scout_status(&data);
    assert!(!status.enabled);
    assert!(status.reason.contains("council action"));
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

#[test]
fn migration_rejects_corrupt_and_unsupported_saves() {
    let data = test_data();

    let corrupt = migrate_save_value(None, json!({ "data": { "clock": "broken" } }), &data)
        .expect_err("corrupt save should not become a new campaign");
    assert!(corrupt.contains("Could not read save"));

    let unsupported = migrate_save_value(Some("9.9.9".to_owned()), json!({}), &data)
        .expect_err("future save versions should be rejected");
    assert!(unsupported.contains("Unsupported save version"));
}
