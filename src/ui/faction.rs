//! Faction, independent settlement, and wilderness pressure overlay.

use super::{virtual_button, UiAction, UiContext};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::RectExt;

pub(super) fn draw_faction_overlay(ctx: &UiContext<'_>, mouse: Vec2, actions: &mut Vec<UiAction>) {
    let screen = super::screen_rect(ctx);
    draw_rectangle(
        screen.x,
        screen.y,
        screen.w,
        screen.h,
        Color::new(0.02, 0.025, 0.02, 0.62),
    );
    let rect = super::centered_modal_rect(ctx, 964.0, 584.0);
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
    let column_gap = 24.0;
    let column_w = (content.w - column_gap) * 0.5;
    let right_x = content.x + column_w + column_gap;
    draw_rival(
        ctx,
        Rect::new(content.x, content.y + 48.0, column_w, content.h * 0.54),
    );
    draw_campaign_controls(
        ctx,
        mouse,
        actions,
        Rect::new(
            content.x,
            content.y + content.h * 0.66,
            column_w,
            content.h * 0.28,
        ),
    );
    draw_independents(ctx, Rect::new(right_x, content.y + 48.0, column_w, 210.0));
    draw_wilderness(
        ctx,
        Rect::new(
            right_x,
            content.y + content.h * 0.48,
            column_w,
            content.h * 0.46,
        ),
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
    for entry in rival.action_log.iter().rev().take(3) {
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

fn draw_campaign_controls(
    ctx: &UiContext<'_>,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
    rect: Rect,
) {
    draw_text_ex(
        "Campaign",
        rect.x,
        rect.y,
        TextStyle::new(18.0, dark::TEXT_BRIGHT).params(),
    );
    let mut y = rect.y + 24.0;
    if let Some(ambition_id) = &ctx.session.selected_ambition_id {
        let name = ctx
            .data
            .campaign_balance
            .ambition(ambition_id)
            .map(|ambition| ambition.name.as_str())
            .unwrap_or(ambition_id.as_str());
        draw_text_ex(
            &format!(
                "{} progress {}",
                name,
                ctx.session.ambition_progress(ctx.data, ambition_id)
            ),
            rect.x,
            y,
            TextStyle::new(14.0, dark::TEXT).params(),
        );
        draw_text_block(
            &ctx.session.ambition_objective_text(ctx.data, ambition_id),
            rect.x,
            y + 4.0,
            rect.w,
            30.0,
            12.0,
            2.0,
            dark::TEXT_DIM,
        );
        y += 42.0;
    } else {
        let button_w = (rect.w - 12.0) / 3.0;
        for (index, ambition) in ctx.data.campaign_balance.ambitions.iter().enumerate() {
            if virtual_button(
                Rect::new(
                    rect.x + index as f32 * (button_w + 6.0),
                    y - 16.0,
                    button_w,
                    28.0,
                ),
                &ambition.name.replace(" Charter", ""),
                true,
                ButtonTone::Primary,
                mouse,
            ) {
                actions.push(UiAction::SelectAmbition(ambition.id.clone()));
            }
        }
        y += 34.0;
    }

    if ctx.session.last_season_rows.is_empty() {
        draw_text_block(
            &ctx.session.last_season_summary,
            rect.x,
            y - 8.0,
            rect.w,
            34.0,
            12.0,
            2.0,
            dark::TEXT_DIM,
        );
        y += 34.0;
    } else {
        for row in ctx.session.last_season_rows.iter().take(3) {
            let label = format!("{}: {}", row.label, row.detail);
            if let Some(site_id) = &row.site_id {
                if virtual_button(
                    Rect::new(rect.x, y - 14.0, rect.w, 22.0),
                    &label,
                    true,
                    ButtonTone::Secondary,
                    mouse,
                ) {
                    actions.push(UiAction::SelectSite(site_id.clone()));
                }
            } else {
                draw_text_block(
                    &label,
                    rect.x,
                    y - 18.0,
                    rect.w,
                    22.0,
                    11.0,
                    1.0,
                    dark::TEXT_DIM,
                );
            }
            y += 24.0;
        }
    }

    let project_w = (rect.w - 12.0) / 3.0;
    for (index, project) in ctx.data.campaign_balance.projects.iter().enumerate() {
        let status = ctx.session.project_status(ctx.data, &project.id);
        if virtual_button(
            Rect::new(
                rect.x + index as f32 * (project_w + 6.0),
                y,
                project_w,
                26.0,
            ),
            &project.name,
            status.enabled,
            ButtonTone::Secondary,
            mouse,
        ) {
            actions.push(UiAction::CompleteProject(project.id.clone()));
        }
    }
    y += 34.0;

    let institution_w = (rect.w - 8.0) / 2.0;
    for (index, institution) in ctx.data.campaign_balance.institutions.iter().enumerate() {
        let status = ctx.session.institution_status(ctx.data, &institution.id);
        if virtual_button(
            Rect::new(
                rect.x + index as f32 * (institution_w + 8.0),
                y,
                institution_w,
                26.0,
            ),
            &institution.name,
            status.enabled,
            ButtonTone::Positive,
            mouse,
        ) {
            actions.push(UiAction::ActivateInstitution(institution.id.clone()));
        }
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
