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
        Some("20-Year Chronicle Summary"),
        &SurfaceStyle::new(Color::new(0.075, 0.070, 0.055, 0.99))
            .with_border(1.0, Color::new(0.64, 0.55, 0.34, 0.85))
            .with_header(48.0, Color::new(0.10, 0.095, 0.075, 1.0))
            .with_header_divider(1.0, Color::new(0.64, 0.55, 0.34, 0.45)),
        TextStyle::new(20.0, dark::TEXT_BRIGHT),
    );
    let content = rect.inset(26.0);
    draw_ui_text_ex(
        &format!("{} - {} points", summary.ending_band, summary.legacy_score),
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
    draw_text_block(
        &format!(
            "Strongest identity: {}\nLargest settlement: {}\nWorst year: {}\nGolden year: {}\nDefining event: {}\nGrouped arcs: {}",
            summary.strongest_identity,
            summary.largest_settlement,
            summary.worst_year,
            summary.golden_year,
            summary.defining_event,
            summary.arcs.join(", ")
        ),
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
        "Restart Campaign",
        true,
        ButtonTone::Primary,
        ctx.pointer,
    ) {
        actions.push(UiAction::EndgameNewGame);
    }
    if virtual_button(
        Rect::new(content.x + button_w + 14.0, button_y, button_w, 38.0),
        "Return to Title",
        true,
        ButtonTone::Secondary,
        ctx.pointer,
    ) {
        actions.push(UiAction::EndgameReturnToTitle);
    }
}
