use realmseed::data::GameData;
use realmseed::state::GameSession;

#[test]
fn independent_request_candidate_targets_independent_site() {
    let data = GameData::load().unwrap();
    let session = GameSession::new(&data);
    let target = session
        .independent_settlements
        .first()
        .map(|independent| independent.site_id.as_str())
        .unwrap();

    assert!(data
        .event_families
        .iter()
        .any(|family| family.id == "independent_request"));
    assert!(data.sites.iter().any(|site| site.id == target));
}
