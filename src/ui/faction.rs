//! Faction, independent settlement, and wilderness pressure overlay.

use super::{virtual_button, UiAction, UiContext};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::RectExt;

pub(super) fn draw_faction_overlay(ctx: &UiContext<'_>, mouse: Vec2, actions: &mut Vec<UiAction>) {
    draw_rectangle(
        0.0,
        0.0,
        super::LOGICAL_WIDTH,
        super::LOGICAL_HEIGHT,
        Color::new(0.02, 0.025, 0.02, 0.62),
    );
    let rect = Rect::new(158.0, 72.0, 964.0, 584.0);
    draw_surface_with_title(
        rect,
        Some("Faction Pressure"),
        &SurfaceStyle::new(Color::new(0.072, 0.068, 0.055, 0.99))
            .with_border(1.0, Color::new(0.64, 0.55, 0.34, 0.85))
            .with_header(48.0, Color::new(0.10, 0.095, 0.075, 1.0))
            .with_header_divider(1.0, Color::new(0.64, 0.55, 0.34, 0.45)),
        TextStyle::new(20.0, dark::TEXT_BRIGHT),
    );
    if virtual_button(
        Rect::new(rect.right() - 96.0, rect.y + 10.0, 76.0, 30.0),
        "Close",
        true,
        ButtonTone::Secondary,
        mouse,
    ) {
        actions.push(UiAction::ToggleFactionPanel);
    }

    let content = rect.inset(24.0);
    draw_rival(ctx, Rect::new(content.x, content.y + 48.0, 430.0, 486.0));
    draw_independents(
        ctx,
        Rect::new(content.x + 456.0, content.y + 48.0, 430.0, 210.0),
    );
    draw_wilderness(
        ctx,
        Rect::new(content.x + 456.0, content.y + 282.0, 430.0, 252.0),
    );
}

fn draw_rival(ctx: &UiContext<'_>, rect: Rect) {
    let rival = &ctx.session.rival_faction;
    draw_text_ex(
        &rival.name,
        rect.x,
        rect.y,
        TextStyle::new(22.0, dark::TEXT_BRIGHT).params(),
    );
    draw_text_block(
        &format!(
            "{} personality. Goal: {}. Confidence {} Fear {} Hostility {} Border {}.",
            rival.personality,
            rival.current_goal.label(),
            rival.confidence,
            rival.fear,
            rival.hostility,
            rival.border_pressure
        ),
        rect.x,
        rect.y + 14.0,
        rect.w,
        60.0,
        14.0,
        3.0,
        dark::TEXT_DIM,
    );
    draw_text_ex(
        "Recent Actions",
        rect.x,
        rect.y + 94.0,
        TextStyle::new(17.0, dark::TEXT_BRIGHT).params(),
    );
    let mut y = rect.y + 118.0;
    for entry in rival.action_log.iter().rev().take(6) {
        let target = entry
            .target_site_id
            .as_deref()
            .and_then(|site_id| ctx.data.site(site_id))
            .map(|site| site.name.as_str())
            .unwrap_or("the frontier");
        draw_text_block(
            &format!(
                "{} Y{}: {} at {}. Reason: {} Result: {}",
                entry.season.label(),
                entry.year,
                entry.action,
                target,
                entry.reason,
                entry.result
            ),
            rect.x,
            y,
            rect.w,
            54.0,
            13.0,
            2.0,
            dark::TEXT,
        );
        y += 60.0;
    }
}

fn draw_independents(ctx: &UiContext<'_>, rect: Rect) {
    draw_text_ex(
        "Independents",
        rect.x,
        rect.y,
        TextStyle::new(18.0, dark::TEXT_BRIGHT).params(),
    );
    let mut y = rect.y + 24.0;
    for independent in &ctx.session.independent_settlements {
        let name = ctx
            .data
            .site(&independent.site_id)
            .map(|site| site.name.as_str())
            .unwrap_or(independent.site_id.as_str());
        draw_text_ex(
            &format!(
                "{}  Trust {}  Autonomy {}  Rival {}  {}",
                name,
                independent.trust,
                independent.autonomy,
                independent.rival_pressure,
                independent.integration_state.label()
            ),
            rect.x,
            y,
            TextStyle::new(14.0, dark::TEXT).params(),
        );
        y += 28.0;
    }
}

fn draw_wilderness(ctx: &UiContext<'_>, rect: Rect) {
    draw_text_ex(
        "Wilderness",
        rect.x,
        rect.y,
        TextStyle::new(18.0, dark::TEXT_BRIGHT).params(),
    );
    let mut y = rect.y + 24.0;
    for pressure in &ctx.session.wilderness_pressure {
        let region_name = ctx
            .data
            .region(&pressure.region_id)
            .map(|region| region.name.as_str())
            .unwrap_or(pressure.region_id.as_str());
        draw_badge(
            Rect::new(rect.x, y - 18.0, 114.0, 24.0),
            pressure.band(),
            wilderness_color(pressure.pressure),
            dark::TEXT,
        );
        draw_text_ex(
            &format!(
                "{}  Pressure {} ({:+})",
                region_name, pressure.pressure, pressure.last_delta
            ),
            rect.x + 126.0,
            y,
            TextStyle::new(14.0, dark::TEXT).params(),
        );
        y += 30.0;
    }
}

fn wilderness_color(pressure: i32) -> Color {
    match pressure {
        0..=24 => Color::new(0.16, 0.24, 0.18, 1.0),
        25..=49 => Color::new(0.24, 0.22, 0.14, 1.0),
        50..=74 => Color::new(0.34, 0.20, 0.12, 1.0),
        _ => Color::new(0.34, 0.12, 0.10, 1.0),
    }
}
