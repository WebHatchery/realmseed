use realmseed::data::GameData;

fn assert_rejected(data: GameData, expected: &str) {
    let error = data
        .validate()
        .expect_err("malformed data should be rejected");
    assert!(
        error.contains(expected),
        "expected `{expected}` in validation error, got `{error}`"
    );
}

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

#[test]
fn validation_rejects_duplicate_and_malformed_world_records() {
    let data = GameData::load().unwrap();

    let mut duplicate_site = data.clone();
    duplicate_site.sites[1].id = duplicate_site.sites[0].id.clone();
    assert_rejected(duplicate_site, "duplicate site id");

    let mut invalid_site_metadata = data.clone();
    invalid_site_metadata.sites[0].region_id = "missing_region".to_owned();
    assert_rejected(invalid_site_metadata, "unknown region");

    let mut invalid_site_owner = data.clone();
    invalid_site_owner.sites[0].owner = Some("ashthorn_clan".to_owned());
    assert_rejected(invalid_site_owner, "invalid owner");
}

#[test]
fn validation_rejects_bad_road_records() {
    let data = GameData::load().unwrap();

    let mut invalid_level = data.clone();
    invalid_level.roads[0].level = 3;
    assert_rejected(invalid_level, "unsupported level");

    let mut invalid_route_type = data;
    invalid_route_type.roads[0].route_type = "skyway".to_owned();
    assert_rejected(invalid_route_type, "unsupported route type");
}

#[test]
fn validation_rejects_bad_faction_and_event_references() {
    let data = GameData::load().unwrap();

    let mut invalid_faction_site = data.clone();
    invalid_faction_site
        .faction_balance
        .rival
        .controlled_locations
        .push("missing_site".to_owned());
    assert_rejected(invalid_faction_site, "unknown controlled site");

    let mut invalid_event_stage = data;
    let resolution_id = invalid_event_stage.event_families[0]
        .resolution_template_id
        .clone();
    invalid_event_stage.event_families[0].opening_template_id = resolution_id;
    assert_rejected(invalid_event_stage, "inconsistent stage reference");
}
