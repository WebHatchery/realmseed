//! Retrievable, prioritized result of the most recent season advance.

use super::{style, UiAction, UiContext};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;
use macroquad_toolkit::ui::RectExt;

pub(super) fn draw_season_report(ctx: &UiContext<'_>, actions: &mut Vec<UiAction>) {
    let screen = super::screen_rect(ctx);
    draw_rectangle(
        screen.x,
        screen.y,
        screen.w,
        screen.h,
        Color::new(0.004, 0.010, 0.012, 0.74),
    );
    let rect = super::centered_modal_rect(ctx, 880.0, 520.0);
    style::draw_panel(rect);
    let content = rect.inset(24.0);
    style::draw_panel_title(
        &ctx.data.text("ui.season_report"),
        content.x,
        content.y + 16.0,
    );
    let season = ctx.session.clock.season.label();
    let year = ctx.session.clock.year.to_string();
    draw_ui_text_ex(
        &ctx.data.text_with(
            "ui.season_report_title",
            &[("{season}", season), ("{year}", &year)],
        ),
        content.x,
        content.y + 52.0,
        TextStyle::new(24.0, style::TEXT_BRIGHT).params(),
    );
    if super::virtual_button(
        Rect::new(content.right() - 86.0, content.y + 2.0, 86.0, 32.0),
        &ctx.data.text("ui.close"),
        ctx.action_review.is_none(),
        ButtonTone::Secondary,
        ctx.pointer,
    ) {
        actions.push(UiAction::ToggleSeasonReport);
    }
    style::draw_divider(content.x, content.y + 76.0, content.w);

    if ctx.session.last_season_rows.is_empty() {
        draw_text_block(
            &ctx.data.text("state.no_season_summary"),
            content.x,
            content.y + 112.0,
            content.w,
            32.0,
            16.0,
            2.0,
            style::TEXT_DIM,
        );
        return;
    }

    let mut y = content.y + 94.0;
    for row in &ctx.session.last_season_rows {
        let row_rect = Rect::new(content.x, y, content.w, 54.0);
        let color = report_color(&row.tag);
        draw_surface(
            row_rect,
            &SurfaceStyle::new(Color::new(0.018, 0.032, 0.034, 0.72))
                .with_border(1.0, Color::new(color.r, color.g, color.b, 0.32)),
        );
        draw_ui_text_ex(
            &row.label,
            row_rect.x + 12.0,
            row_rect.y + 19.0,
            TextStyle::new(14.0, color).params(),
        );
        draw_text_block(
            &row.detail,
            row_rect.x + 12.0,
            row_rect.y + 25.0,
            row_rect.w - if row.site_id.is_some() { 150.0 } else { 24.0 },
            22.0,
            12.5,
            1.0,
            style::TEXT,
        );
        if let Some(site_id) = &row.site_id {
            if ctx.action_review.is_none() && ctx.pointer.released_on(touch_area(row_rect)) {
                actions.push(UiAction::SelectSite(site_id.clone()));
                actions.push(UiAction::ToggleSeasonReport);
            }
            draw_ui_text_ex(
                &ctx.data.text("ui.report_site_hint"),
                row_rect.right() - 132.0,
                row_rect.y + 31.0,
                TextStyle::new(11.0, style::GOLD).params(),
            );
        }
        y += 62.0;
    }
}

fn report_color(tag: &str) -> Color {
    match tag {
        "crisis" => style::RED,
        "road" => style::GOLD,
        "faction" => Color::new(0.68, 0.50, 0.86, 1.0),
        _ => style::CYAN,
    }
}
