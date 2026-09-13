//! Chronicle overlay listing recent realm history entries.

use crate::ui::{virtual_button, UiAction, UiContext};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;
use macroquad_toolkit::ui::RectExt;

pub(in crate::ui) fn draw_chronicle_overlay(ctx: &UiContext<'_>, actions: &mut Vec<UiAction>) {
    let shade = Color::new(0.02, 0.025, 0.02, 0.60);
    let screen = crate::ui::screen_rect(ctx);
    draw_rectangle(screen.x, screen.y, screen.w, screen.h, shade);

    let rect = crate::ui::centered_modal_rect(ctx, 936.0, 556.0);
    let style = SurfaceStyle::new(Color::new(0.075, 0.070, 0.055, 0.99))
        .with_border(1.0, Color::new(0.64, 0.55, 0.34, 0.85))
        .with_header(48.0, Color::new(0.10, 0.095, 0.075, 1.0))
        .with_header_divider(1.0, Color::new(0.64, 0.55, 0.34, 0.45));
    draw_surface_with_title(
        rect,
        Some("Chronicle"),
        &style,
        TextStyle::new(20.0, dark::TEXT_BRIGHT),
    );

    if virtual_button(
        Rect::new(rect.right() - 96.0, rect.y + 10.0, 76.0, 30.0),
        "Close",
        true,
        ButtonTone::Secondary,
        ctx.pointer,
    ) {
        actions.push(UiAction::ToggleChronicle);
    }

    let content = rect.inset(24.0);
    let mut y = content.y + 48.0;
    for entry in ctx.session.chronicle.iter().rev().take(8) {
        let title = format!(
            "[{}] {} Year {} - {}",
            entry.tag,
            entry.season.label(),
            entry.year,
            entry.title
        );
        draw_ui_text_ex(
            &title,
            content.x,
            y,
            TextStyle::new(17.0, dark::TEXT_BRIGHT).params(),
        );
        draw_text_block(
            &entry.body,
            content.x + 14.0,
            y + 10.0,
            content.w - 28.0,
            44.0,
            15.0,
            3.0,
            dark::TEXT_DIM,
        );
        y += 70.0;
    }

    if ctx.session.chronicle.is_empty() {
        draw_text_centered_in_box(
            "No chronicle entries yet.",
            content.x,
            content.y + 90.0,
            content.w,
            50.0,
            18.0,
            dark::TEXT_DIM,
        );
    }
}
