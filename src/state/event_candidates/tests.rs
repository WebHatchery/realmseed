use super::*;
use crate::state::GameSession;

fn test_data() -> GameData {
    GameData::load().unwrap()
}

#[test]
fn independent_request_candidate_targets_independent_site() {
    let data = test_data();
    let mut session = GameSession::new(&data);
    let expected_site_id = session.independent_settlements[0].site_id.clone();
    session.independent_settlements[0].rival_pressure = 80;

    let family = data.event_family("independent_request").unwrap();
    let candidate = session.candidate_for_family(&data, family).unwrap();

    assert_eq!(candidate.target_site_id, expected_site_id);
    assert!(session
        .independent_settlements
        .iter()
        .any(|independent| independent.site_id == candidate.target_site_id));
}
