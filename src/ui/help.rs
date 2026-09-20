//! Reopenable, touch-readable controls and opening guidance.

use super::{style, UiAction, UiContext};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;
use macroquad_toolkit::ui::RectExt;

pub(super) fn draw_help_overlay(ctx: &UiContext<'_>, actions: &mut Vec<UiAction>) {
    let screen = super::screen_rect(ctx);
    draw_rectangle(
        screen.x,
        screen.y,
        screen.w,
        screen.h,
        Color::new(0.004, 0.010, 0.012, 0.74),
    );
    let rect = super::centered_modal_rect(ctx, 820.0, 520.0);
    style::draw_panel(rect);
    let content = rect.inset(24.0);
    style::draw_panel_title(&ctx.data.text("ui.help_title"), content.x, content.y + 16.0);
    draw_ui_text_ex(
        &ctx.data.text("ui.help_subtitle"),
        content.x,
        content.y + 50.0,
        TextStyle::new(23.0, style::TEXT_BRIGHT).params(),
    );
    if super::virtual_button(
        Rect::new(content.right() - 86.0, content.y + 2.0, 86.0, 32.0),
        &ctx.data.text("ui.close"),
        ctx.action_review.is_none(),
        ButtonTone::Secondary,
        ctx.pointer,
    ) {
        actions.push(UiAction::ToggleHelp);
    }
    style::draw_divider(content.x, content.y + 76.0, content.w);

    section(
        ctx,
        "ui.help_current",
        &ctx.session.guidance_text(ctx.data),
        content.x,
        content.y + 100.0,
        content.w,
    );
    section(
        ctx,
        "ui.help_map_title",
        "ui.help_map",
        content.x,
        content.y + 168.0,
        content.w,
    );
    section(
        ctx,
        "ui.help_decisions_title",
        "ui.help_decisions",
        content.x,
        content.y + 238.0,
        content.w,
    );
    section(
        ctx,
        "ui.help_views_title",
        "ui.help_views",
        content.x,
        content.y + 308.0,
        content.w,
    );
    draw_text_block(
        &ctx.data.text("ui.help_escape"),
        content.x,
        content.bottom() - 32.0,
        content.w,
        24.0,
        12.0,
        1.0,
        style::TEXT_DIM,
    );
}

fn section(ctx: &UiContext<'_>, title_id: &str, body: &str, x: f32, y: f32, width: f32) {
    draw_ui_text_ex(
        &ctx.data.text(title_id),
        x,
        y,
        TextStyle::new(14.0, style::GOLD).params(),
    );
    let body_text = if body.starts_with("ui.") {
        ctx.data.text(body)
    } else {
        body.to_owned()
    };
    draw_text_block(&body_text, x, y + 8.0, width, 42.0, 13.0, 2.0, style::TEXT);
}
