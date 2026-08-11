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
    assert!(data.event_families.len() >= 12);
    assert!(data.event_templates.len() >= 60);
    assert!(data.chronicle_templates.len() >= 60);
    assert_eq!(data.faction_balance.rival.id, "ashthorn_clan");
    assert_eq!(data.campaign_balance.campaign_turns, 80);
    assert_eq!(
        data.sites
            .iter()
            .filter(|site| site.initially_visible)
            .count(),
        8
    );
}
