//! Endgame legacy summary modal.

use super::{virtual_button, UiAction, UiContext};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;
use macroquad_toolkit::ui::RectExt;

pub(super) fn draw_endgame_summary(ctx: &UiContext<'_>, actions: &mut Vec<UiAction>) {
    let Some(summary) = &ctx.session.endgame_summary else {
        return;
    };
    let screen = super::screen_rect(ctx);
    draw_rectangle(
        screen.x,
        screen.y,
        screen.w,
        screen.h,
        Color::new(0.02, 0.025, 0.02, 0.70),
    );
    let rect = super::centered_modal_rect(ctx, 868.0, 520.0);
    draw_surface_with_title(
        rect,
        Some(&ctx.data.text("ui.endgame_title")),
        &SurfaceStyle::new(Color::new(0.075, 0.070, 0.055, 0.99))
            .with_border(1.0, Color::new(0.64, 0.55, 0.34, 0.85))
            .with_header(48.0, Color::new(0.10, 0.095, 0.075, 1.0))
            .with_header_divider(1.0, Color::new(0.64, 0.55, 0.34, 0.45)),
        TextStyle::new(20.0, dark::TEXT_BRIGHT),
    );
    let content = rect.inset(26.0);
    let score = ctx.data.text_with(
        "ui.endgame_score",
        &[
            ("{band}", &summary.ending_band),
            ("{score}", &summary.legacy_score.to_string()),
        ],
    );
    draw_ui_text_ex(
        &score,
        content.x,
        content.y + 54.0,
        TextStyle::new(24.0, dark::TEXT_BRIGHT).params(),
    );
    draw_text_block(
        &summary.summary_text,
        content.x,
        content.y + 80.0,
        content.w,
        190.0,
        17.0,
        4.0,
        dark::TEXT,
    );
    let details = ctx.data.text_with(
        "state.summary_details",
        &[
            ("{identity}", &summary.strongest_identity),
            ("{largest}", &summary.largest_settlement),
            ("{worst}", &summary.worst_year.to_string()),
            ("{golden}", &summary.golden_year.to_string()),
            ("{event}", &summary.defining_event),
            ("{arcs}", &summary.arcs.join(", ")),
        ],
    );
    draw_text_block(
        &details,
        content.x,
        content.y + 286.0,
        content.w,
        150.0,
        16.0,
        4.0,
        dark::TEXT_DIM,
    );
    let button_y = content.bottom() - 42.0;
    let button_w = (content.w - 14.0) * 0.5;
    if virtual_button(
        Rect::new(content.x, button_y, button_w, 38.0),
        &ctx.data.text("ui.restart_campaign"),
        true,
        ButtonTone::Primary,
        ctx.pointer,
    ) {
        actions.push(UiAction::EndgameNewGame);
    }
    if virtual_button(
        Rect::new(content.x + button_w + 14.0, button_y, button_w, 38.0),
        &ctx.data.text("ui.return_title"),
        true,
        ButtonTone::Secondary,
        ctx.pointer,
    ) {
        actions.push(UiAction::EndgameReturnToTitle);
    }
}
