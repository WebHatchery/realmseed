//! Prioritized feedback emitted after a season resolves.

use super::Game;
use crate::state::SeasonAdvanceReport;

pub(super) fn notify_season_result(game: &mut Game, report: &SeasonAdvanceReport) {
    let mut messages = Vec::new();
    let mut tone = SeasonTone::Info;
    push_count(
        game,
        &mut messages,
        &mut tone,
        report.settlements_lost,
        "game.settlement_lost",
        SeasonTone::Danger,
    );
    push_count(
        game,
        &mut messages,
        &mut tone,
        report.food_shortages,
        "game.food_shortage",
        SeasonTone::Warning,
    );
    push_count(
        game,
        &mut messages,
        &mut tone,
        report.issues_escalated,
        "game.issue_escalated",
        SeasonTone::Warning,
    );
    push_count(
        game,
        &mut messages,
        &mut tone,
        report.events_triggered,
        "game.event_pending",
        SeasonTone::Warning,
    );
    push_count(
        game,
        &mut messages,
        &mut tone,
        report.isolated_settlements,
        "game.isolated",
        SeasonTone::Warning,
    );
    push_count(
        game,
        &mut messages,
        &mut tone,
        report.road_warnings,
        "game.road_warning",
        SeasonTone::Warning,
    );
    push_count(
        game,
        &mut messages,
        &mut tone,
        report.independent_requests,
        "game.independent_request",
        SeasonTone::Warning,
    );
    if report.rival_actions > 0 {
        messages.push(game.data.text("game.rival_acted"));
    }
    push_count(
        game,
        &mut messages,
        &mut tone,
        report.wilderness_changes,
        "game.wilderness_changed",
        SeasonTone::Warning,
    );
    if report.unmanaged_strain > 0 {
        let strain = report.unmanaged_strain.to_string();
        messages.push(
            game.data
                .text_with("game.unmanaged_strain", &[("{count}", &strain)]),
        );
    }

    let season = game.session.clock.season.label().to_owned();
    let year = game.session.clock.year.to_string();
    let summary = if messages.is_empty() {
        game.data.text("game.season_clear")
    } else {
        messages.join(" · ")
    };
    let message = game.data.text_with(
        "game.season_result",
        &[
            ("{season}", &season),
            ("{year}", &year),
            ("{summary}", &summary),
        ],
    );
    match tone {
        SeasonTone::Info => game.notifications.info(message),
        SeasonTone::Warning => game.notifications.warning(message),
        SeasonTone::Danger => game.notifications.danger(message),
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum SeasonTone {
    Info,
    Warning,
    Danger,
}

fn push_count(
    game: &Game,
    messages: &mut Vec<String>,
    tone: &mut SeasonTone,
    count: usize,
    text_id: &str,
    message_tone: SeasonTone,
) {
    if count == 0 {
        return;
    }
    *tone = (*tone).max(message_tone);
    let count = count.to_string();
    messages.push(game.data.text_with(text_id, &[("{count}", &count)]));
}
