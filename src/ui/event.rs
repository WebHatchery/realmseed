//! Event modal rendering and choice intents.

use super::{virtual_button, UiAction, UiContext};
use crate::state::fill_event_text;
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::RectExt;

pub(super) fn draw_event_modal(ctx: &UiContext<'_>, mouse: Vec2, actions: &mut Vec<UiAction>) {
    let Some(pending) = &ctx.session.pending_event else {
        return;
    };
    let Some(template) = ctx.session.pending_event_template(ctx.data) else {
        return;
    };
    let site_name = ctx.session.pending_event_site_name(ctx.data);

    let screen = super::screen_rect(ctx);
    draw_rectangle(
        screen.x,
        screen.y,
        screen.w,
        screen.h,
        Color::new(0.02, 0.025, 0.02, 0.66),
    );

    let rect = super::centered_modal_rect(ctx, 844.0, 548.0);
    draw_surface_with_title(
        rect,
        Some("Event"),
        &SurfaceStyle::new(Color::new(0.075, 0.070, 0.055, 0.99))
            .with_border(1.0, Color::new(0.64, 0.55, 0.34, 0.85))
            .with_header(48.0, Color::new(0.10, 0.095, 0.075, 1.0))
            .with_header_divider(1.0, Color::new(0.64, 0.55, 0.34, 0.45)),
        TextStyle::new(20.0, dark::TEXT_BRIGHT),
    );

    let content = rect.inset(24.0);
    let mut y = content.y + 52.0;
    draw_text_ex(
        &fill_event_text(&template.title, &site_name),
        content.x,
        y,
        TextStyle::new(25.0, dark::TEXT_BRIGHT).params(),
    );
    draw_badge(
        Rect::new(content.right() - 116.0, y - 24.0, 116.0, 28.0),
        &format!("Severity {}", pending.severity),
        Color::new(0.28, 0.18, 0.12, 1.0),
        dark::TEXT,
    );
    y += 24.0;

    draw_text_block(
        &fill_event_text(&template.narrative, &site_name),
        content.x,
        y,
        content.w,
        74.0,
        17.0,
        4.0,
        dark::TEXT,
    );
    y += 84.0;

    draw_event_detail("Cause", &pending.cause, content.x, y, content.w);
    y += 48.0;
    draw_event_detail(
        "Visible",
        &template.visible_consequences,
        content.x,
        y,
        content.w,
    );
    y += 48.0;
    draw_event_detail(
        "Uncertain",
        &template.hidden_consequences,
        content.x,
        y,
        content.w,
    );
    y += 58.0;

    for choice in &template.choices {
        let status = ctx.session.event_choice_status(ctx.data, choice);
        if virtual_button(
            Rect::new(content.x, y, 238.0, 34.0),
            &choice.label,
            status.enabled,
            ButtonTone::Primary,
            mouse,
        ) {
            actions.push(UiAction::ResolveEventChoice(choice.id.clone()));
        }
        draw_text_block(
            &choice.visible_consequence,
            content.x + 252.0,
            y + 2.0,
            content.w - 252.0,
            34.0,
            14.0,
            2.0,
            if status.enabled {
                dark::TEXT_DIM
            } else {
                dark::WARNING
            },
        );
        y += 44.0;
    }

    if template.allow_defer
        && virtual_button(
            Rect::new(
                content.right() - 150.0,
                content.bottom() - 34.0,
                150.0,
                32.0,
            ),
            "Defer",
            true,
            ButtonTone::Secondary,
            mouse,
        )
    {
        actions.push(UiAction::DeferEvent);
    }
}

fn draw_event_detail(label: &str, text: &str, x: f32, y: f32, width: f32) {
    draw_text_ex(
        label,
        x,
        y,
        TextStyle::new(14.0, dark::TEXT_BRIGHT).params(),
    );
    draw_text_block(
        text,
        x + 76.0,
        y - 15.0,
        width - 76.0,
        42.0,
        14.0,
        2.0,
        dark::TEXT_DIM,
    );
}
