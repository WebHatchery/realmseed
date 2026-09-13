//! Faction, independent settlement, and wilderness pressure overlay.

use super::{style, virtual_button, UiAction, UiContext};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;
use macroquad_toolkit::ui::RectExt;

pub(super) fn draw_faction_overlay(ctx: &UiContext<'_>, actions: &mut Vec<UiAction>) {
    let screen = super::screen_rect(ctx);
    draw_rectangle(
        screen.x,
        screen.y,
        screen.w,
        screen.h,
        Color::new(0.004, 0.010, 0.012, 0.70),
    );

    let rect = super::centered_modal_rect(ctx, 984.0, 650.0);
    style::draw_panel(rect);
    let content = rect.inset(24.0);

    style::draw_framed_icon(
        style::IconKind::Danger,
        vec2(content.x + 28.0, content.y + 26.0),
        48.0,
        Color::new(0.13, 0.08, 0.05, 0.96),
    );
    style::draw_panel_title("FRONTIER PRESSURE", content.x + 68.0, content.y + 17.0);
    draw_ui_text_ex(
        "Faction Pressure",
        content.x + 68.0,
        content.y + 48.0,
        TextStyle::new(25.0, style::TEXT_BRIGHT).params(),
    );
    if virtual_button(
        Rect::new(content.right() - 86.0, content.y + 14.0, 86.0, 32.0),
        "Close",
        true,
        ButtonTone::Secondary,
        ctx.pointer,
    ) {
        actions.push(UiAction::ToggleFactionPanel);
    }
    style::draw_divider(content.x, content.y + 76.0, content.w);

    let column_gap = 24.0;
    let column_w = (content.w - column_gap) * 0.5;
    let right_x = content.x + column_w + column_gap;
    let body_y = content.y + 102.0;
    let body_h = content.bottom() - body_y;
    draw_rival(ctx, Rect::new(content.x, body_y, column_w, body_h * 0.58));
    draw_campaign_controls(
        ctx,
        ctx.pointer,
        actions,
        Rect::new(content.x, body_y + body_h * 0.64, column_w, body_h * 0.32),
    );
    style::draw_vertical_divider(right_x - column_gap * 0.5, body_y, body_h - 10.0);
    draw_independents(
        ctx,
        Rect::new(right_x, body_y, column_w, (body_h * 0.42).max(178.0)),
    );
    draw_wilderness(
        ctx,
        Rect::new(right_x, body_y + body_h * 0.50, column_w, body_h * 0.44),
    );
}

fn draw_rival(ctx: &UiContext<'_>, rect: Rect) {
    let rival = &ctx.session.rival_faction;
    draw_section_heading("RIVAL CLAN", rect.x, rect.y, rect.w);
    draw_ui_text_ex(
        &rival.name,
        rect.x,
        rect.y + 32.0,
        TextStyle::new(22.0, style::TEXT_BRIGHT).params(),
    );
    draw_text_block(
        &format!(
            "{} personality. Goal: {}.",
            rival.personality,
            rival.current_goal.label()
        ),
        rect.x,
        rect.y + 48.0,
        rect.w,
        36.0,
        14.0,
        3.0,
        style::TEXT_DIM,
    );
    draw_rival_stats(
        rect.x,
        rect.y + 88.0,
        rect.w,
        [
            ("Conf", rival.confidence),
            ("Fear", rival.fear),
            ("Hostile", rival.hostility),
            ("Border", rival.border_pressure),
        ],
    );
    draw_ui_text_ex(
        "Recent Actions",
        rect.x,
        rect.y + 132.0,
        TextStyle::new(16.0, style::GOLD).params(),
    );
    let mut y = rect.y + 156.0;
    for entry in rival.action_log.iter().rev().take(2) {
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
            44.0,
            13.0,
            2.0,
            style::TEXT,
        );
        y += 48.0;
    }
    if rival.action_log.is_empty() {
        draw_text_block(
            "No rival actions have been logged yet.",
            rect.x,
            y - 4.0,
            rect.w,
            30.0,
            13.0,
            2.0,
            style::TEXT_DIM,
        );
    }
}

fn draw_campaign_controls(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    rect: Rect,
) {
    let compact = rect.h < 182.0;
    draw_section_heading("CAMPAIGN", rect.x, rect.y, rect.w);
    draw_ui_text_ex(
        "Ambition",
        rect.x,
        rect.y + 32.0,
        TextStyle::new(15.0, style::GOLD).params(),
    );
    let mut y = rect.y + if compact { 50.0 } else { 54.0 };
    if let Some(ambition_id) = &ctx.session.selected_ambition_id {
        let name = ctx
            .data
            .campaign_balance
            .ambition(ambition_id)
            .map(|ambition| ambition.name.as_str())
            .unwrap_or(ambition_id.as_str());
        draw_ui_text_ex(
            &format!(
                "{} progress {}",
                name,
                ctx.session.ambition_progress(ctx.data, ambition_id)
            ),
            rect.x,
            y,
            TextStyle::new(14.0, style::TEXT_BRIGHT).params(),
        );
        if !compact {
            draw_text_block(
                &ctx.session.ambition_objective_text(ctx.data, ambition_id),
                rect.x,
                y + 4.0,
                rect.w,
                28.0,
                12.0,
                2.0,
                style::TEXT_DIM,
            );
            y += 40.0;
        } else {
            y += 22.0;
        }
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
                pointer,
            ) {
                actions.push(UiAction::SelectAmbition(ambition.id.clone()));
            }
        }
        y += if compact { 28.0 } else { 34.0 };
    }

    style::draw_divider(rect.x, y + 2.0, rect.w);
    y += if compact { 16.0 } else { 22.0 };

    if ctx.session.last_season_rows.is_empty() {
        draw_text_block(
            &ctx.session.last_season_summary,
            rect.x,
            y - 8.0,
            rect.w,
            34.0,
            12.0,
            2.0,
            style::TEXT_DIM,
        );
        y += if compact { 24.0 } else { 34.0 };
    } else {
        let row_limit = if compact { 1 } else { 2 };
        for row in ctx.session.last_season_rows.iter().take(row_limit) {
            let label = format!("{}: {}", row.label, row.detail);
            if let Some(site_id) = &row.site_id {
                if virtual_button(
                    Rect::new(rect.x, y - 14.0, rect.w, 22.0),
                    &label,
                    true,
                    ButtonTone::Secondary,
                    pointer,
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
                    style::TEXT_DIM,
                );
            }
            y += if compact { 22.0 } else { 24.0 };
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
            pointer,
        ) {
            actions.push(UiAction::CompleteProject(project.id.clone()));
        }
    }
    y += if compact { 30.0 } else { 34.0 };

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
            pointer,
        ) {
            actions.push(UiAction::ActivateInstitution(institution.id.clone()));
        }
    }
}

fn draw_independents(ctx: &UiContext<'_>, rect: Rect) {
    draw_section_heading("INDEPENDENTS", rect.x, rect.y, rect.w);
    let mut y = rect.y + 32.0;
    for independent in &ctx.session.independent_settlements {
        let name = ctx
            .data
            .site(&independent.site_id)
            .map(|site| site.name.as_str())
            .unwrap_or(independent.site_id.as_str());
        draw_ui_text_ex(
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
            TextStyle::new(14.0, style::TEXT).params(),
        );
        y += 28.0;
    }
}

fn draw_wilderness(ctx: &UiContext<'_>, rect: Rect) {
    draw_section_heading("WILDERNESS", rect.x, rect.y, rect.w);
    let mut y = rect.y + 34.0;
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
            style::TEXT,
        );
        draw_ui_text_ex(
            &format!(
                "{}  Pressure {} ({:+})",
                region_name, pressure.pressure, pressure.last_delta
            ),
            rect.x + 126.0,
            y,
            TextStyle::new(14.0, style::TEXT).params(),
        );
        y += 30.0;
    }
}

fn draw_section_heading(text: &str, x: f32, y: f32, width: f32) {
    style::draw_panel_title(text, x, y + 14.0);
    style::draw_divider(x, y + 26.0, width);
}

fn draw_rival_stats(x: f32, y: f32, width: f32, stats: [(&str, i32); 4]) {
    let gap = 8.0;
    let item_w = (width - gap * 3.0) / 4.0;
    for (index, (label, value)) in stats.iter().enumerate() {
        let rect = Rect::new(x + index as f32 * (item_w + gap), y, item_w, 30.0);
        style::draw_button_frame(rect, ButtonTone::Muted, true, false, false);
        draw_text_centered_in_box_ex(
            &format!("{} {}", label, value),
            rect.x + 6.0,
            rect.y,
            rect.w - 12.0,
            rect.h,
            TextStyle::new(11.5, style::TEXT),
        );
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
