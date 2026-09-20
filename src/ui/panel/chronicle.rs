//! Chronicle overlay listing recent realm history entries.

use crate::ui::{style, virtual_button, UiAction, UiContext};
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
        Some(&ctx.data.text("ui.chronicle_overlay")),
        &style,
        TextStyle::new(20.0, dark::TEXT_BRIGHT),
    );

    if virtual_button(
        Rect::new(rect.right() - 96.0, rect.y + 10.0, 76.0, 30.0),
        &ctx.data.text("ui.close"),
        ctx.action_review.is_none(),
        ButtonTone::Secondary,
        ctx.pointer,
    ) {
        actions.push(UiAction::ToggleChronicle);
    }

    let content = rect.inset(24.0);
    let entries_per_page = 5;
    let total_pages = ctx
        .session
        .chronicle
        .len()
        .div_ceil(entries_per_page)
        .max(1);
    let page = ctx.chronicle_page.min(total_pages - 1);
    let mut y = content.y + 48.0;
    for entry in ctx
        .session
        .chronicle
        .iter()
        .rev()
        .skip(page * entries_per_page)
        .take(entries_per_page)
    {
        let row_rect = Rect::new(content.x, y - 8.0, content.w, 62.0);
        if entry.site_id.is_some()
            && ctx.action_review.is_none()
            && ctx.pointer.released_on(touch_area(row_rect))
        {
            if let Some(site_id) = &entry.site_id {
                actions.push(UiAction::SelectSite(site_id.clone()));
                actions.push(UiAction::ToggleChronicle);
            }
        }
        if entry.site_id.is_some() && ctx.pointer.hovering_over(row_rect) {
            draw_rectangle(
                row_rect.x,
                row_rect.y,
                row_rect.w,
                row_rect.h,
                Color::new(0.95, 0.82, 0.50, 0.07),
            );
        }
        let season = entry.season.label().to_owned();
        let year = entry.year.to_string();
        let title = ctx.data.text_with(
            "ui.chronicle_entry",
            &[
                ("{tag}", &entry.tag),
                ("{season}", &season),
                ("{year}", &year),
                ("{title}", &entry.title),
            ],
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
            content.w - if entry.site_id.is_some() { 164.0 } else { 28.0 },
            44.0,
            15.0,
            3.0,
            dark::TEXT_DIM,
        );
        if entry.site_id.is_some() {
            draw_ui_text_ex(
                &ctx.data.text("ui.report_site_hint"),
                content.right() - 136.0,
                y + 31.0,
                TextStyle::new(12.0, style::GOLD).params(),
            );
        }
        y += 70.0;
    }

    if ctx.session.chronicle.is_empty() {
        draw_text_centered_in_box(
            &ctx.data.text("ui.no_chronicle"),
            content.x,
            content.y + 90.0,
            content.w,
            50.0,
            18.0,
            dark::TEXT_DIM,
        );
    }

    let page_label = ctx.data.text_with(
        "ui.page_of",
        &[
            ("{page}", &(page + 1).to_string()),
            ("{pages}", &total_pages.to_string()),
        ],
    );
    draw_ui_text_ex(
        &page_label,
        content.x,
        content.bottom() - 10.0,
        TextStyle::new(13.0, dark::TEXT_DIM).params(),
    );
    if virtual_button(
        Rect::new(
            content.right() - 230.0,
            content.bottom() - 34.0,
            104.0,
            30.0,
        ),
        &ctx.data.text("ui.previous"),
        ctx.action_review.is_none() && page > 0,
        ButtonTone::Secondary,
        ctx.pointer,
    ) {
        actions.push(UiAction::ChroniclePrevious);
    }
    if virtual_button(
        Rect::new(
            content.right() - 114.0,
            content.bottom() - 34.0,
            114.0,
            30.0,
        ),
        &ctx.data.text("ui.next"),
        ctx.action_review.is_none() && page + 1 < total_pages,
        ButtonTone::Secondary,
        ctx.pointer,
    ) {
        actions.push(UiAction::ChronicleNext);
    }
}
