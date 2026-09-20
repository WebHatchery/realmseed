//! Faction, independent settlement, and wilderness pressure overlay.

use super::{inspectable_button, style, virtual_button, ActionReview, UiAction, UiContext};
use crate::data::FactionGoal;
use crate::state::{IntegrationState, WildernessPressureState};
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
    let input_enabled = ctx.action_review.is_none();

    style::draw_framed_icon(
        style::IconKind::Danger,
        vec2(content.x + 28.0, content.y + 26.0),
        48.0,
        Color::new(0.13, 0.08, 0.05, 0.96),
    );
    style::draw_panel_title(
        &ctx.data.text("ui.frontier_pressure"),
        content.x + 68.0,
        content.y + 17.0,
    );
    draw_ui_text_ex(
        &ctx.data.text("ui.faction_pressure"),
        content.x + 68.0,
        content.y + 48.0,
        TextStyle::new(25.0, style::TEXT_BRIGHT).params(),
    );
    if virtual_button(
        Rect::new(content.right() - 86.0, content.y + 14.0, 86.0, 32.0),
        &ctx.data.text("ui.close"),
        input_enabled,
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
        input_enabled,
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
    draw_section_heading(ctx, "ui.rival_clan", rect.x, rect.y, rect.w);
    draw_ui_text_ex(
        &rival.name,
        rect.x,
        rect.y + 32.0,
        TextStyle::new(22.0, style::TEXT_BRIGHT).params(),
    );
    draw_text_block(
        &ctx.data.text_with(
            "ui.rival_description",
            &[
                ("{personality}", &rival.personality),
                ("{goal}", &goal_label(ctx, rival.current_goal)),
            ],
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
            (ctx.data.text("ui.confidence"), rival.confidence),
            (ctx.data.text("ui.fear"), rival.fear),
            (ctx.data.text("ui.hostile"), rival.hostility),
            (ctx.data.text("ui.border"), rival.border_pressure),
        ],
    );
    draw_ui_text_ex(
        &ctx.data.text("ui.recent_actions"),
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
            .map(|site| site.name.clone())
            .unwrap_or_else(|| ctx.data.text("state.frontier"));
        let season = entry.season.label().to_owned();
        let year = entry.year.to_string();
        draw_text_block(
            &ctx.data.text_with(
                "ui.rival_action_log",
                &[
                    ("{season}", &season),
                    ("{year}", &year),
                    ("{action}", &entry.action),
                    ("{target}", &target),
                    ("{reason}", &entry.reason),
                    ("{result}", &entry.result),
                ],
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
            &ctx.data.text("ui.no_rival_actions"),
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
    input_enabled: bool,
    actions: &mut Vec<UiAction>,
    rect: Rect,
) {
    let compact = rect.h < 182.0;
    draw_section_heading(ctx, "ui.campaign", rect.x, rect.y, rect.w);
    draw_ui_text_ex(
        &ctx.data.text("ui.ambition"),
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
            &ctx.data.text_with(
                "ui.campaign_progress",
                &[
                    ("{name}", name),
                    (
                        "{progress}",
                        &ctx.session
                            .ambition_progress(ctx.data, ambition_id)
                            .to_string(),
                    ),
                ],
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
                input_enabled,
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

    if virtual_button(
        Rect::new(rect.x, y - 12.0, rect.w, 30.0),
        &ctx.data.text("ui.season_report"),
        input_enabled,
        ButtonTone::Secondary,
        pointer,
    ) {
        actions.push(UiAction::ToggleSeasonReport);
    }
    y += if compact { 32.0 } else { 36.0 };

    let project_w = (rect.w - 12.0) / 3.0;
    for (index, project) in ctx.data.campaign_balance.projects.iter().enumerate() {
        let status = ctx.session.project_status(ctx.data, &project.id);
        if inspectable_button(
            Rect::new(
                rect.x + index as f32 * (project_w + 6.0),
                y,
                project_w,
                26.0,
            ),
            &project.name,
            status.enabled,
            input_enabled,
            ButtonTone::Secondary,
            pointer,
        ) {
            actions.push(UiAction::OpenActionReview(ActionReview::CompleteProject(
                project.id.clone(),
            )));
        }
    }
    y += if compact { 30.0 } else { 34.0 };

    let institution_w = (rect.w - 8.0) / 2.0;
    for (index, institution) in ctx.data.campaign_balance.institutions.iter().enumerate() {
        let status = ctx.session.institution_status(ctx.data, &institution.id);
        if inspectable_button(
            Rect::new(
                rect.x + index as f32 * (institution_w + 8.0),
                y,
                institution_w,
                26.0,
            ),
            &institution.name,
            status.enabled,
            input_enabled,
            ButtonTone::Positive,
            pointer,
        ) {
            actions.push(UiAction::OpenActionReview(
                ActionReview::ActivateInstitution(institution.id.clone()),
            ));
        }
    }
}

fn draw_independents(ctx: &UiContext<'_>, rect: Rect) {
    draw_section_heading(ctx, "ui.independents", rect.x, rect.y, rect.w);
    let mut y = rect.y + 32.0;
    for independent in &ctx.session.independent_settlements {
        let name = ctx
            .data
            .site(&independent.site_id)
            .map(|site| site.name.clone())
            .unwrap_or_else(|| ctx.data.text("state.frontier"));
        let state = integration_label(ctx, independent.integration_state);
        draw_ui_text_ex(
            &ctx.data.text_with(
                "ui.independent_line",
                &[
                    ("{name}", &name),
                    ("{trust}", &independent.trust.to_string()),
                    ("{autonomy}", &independent.autonomy.to_string()),
                    ("{rival}", &independent.rival_pressure.to_string()),
                    ("{state}", &state),
                ],
            ),
            rect.x,
            y,
            TextStyle::new(14.0, style::TEXT).params(),
        );
        y += 28.0;
    }
}

fn draw_wilderness(ctx: &UiContext<'_>, rect: Rect) {
    draw_section_heading(ctx, "ui.wilderness", rect.x, rect.y, rect.w);
    let mut y = rect.y + 34.0;
    for pressure in &ctx.session.wilderness_pressure {
        let region_name = ctx
            .data
            .region(&pressure.region_id)
            .map(|region| region.name.clone())
            .unwrap_or_else(|| pressure.region_id.clone());
        let band = wilderness_band(ctx, pressure);
        draw_badge(
            Rect::new(rect.x, y - 18.0, 114.0, 24.0),
            &band,
            wilderness_color(pressure.pressure),
            style::TEXT,
        );
        draw_ui_text_ex(
            &ctx.data.text_with(
                "ui.wilderness_line",
                &[
                    ("{region}", &region_name),
                    ("{pressure}", &pressure.pressure.to_string()),
                    ("{delta}", &format!("{:+}", pressure.last_delta)),
                ],
            ),
            rect.x + 126.0,
            y,
            TextStyle::new(14.0, style::TEXT).params(),
        );
        y += 30.0;
    }
}

fn draw_section_heading(ctx: &UiContext<'_>, text_id: &str, x: f32, y: f32, width: f32) {
    style::draw_panel_title(&ctx.data.text(text_id), x, y + 14.0);
    style::draw_divider(x, y + 26.0, width);
}

fn goal_label(ctx: &UiContext<'_>, goal: FactionGoal) -> String {
    let text_id = match goal {
        FactionGoal::Expand => "ui.goal_expand",
        FactionGoal::Fortify => "ui.goal_fortify",
        FactionGoal::Raid => "ui.goal_raid",
        FactionGoal::Trade => "ui.goal_trade",
        FactionGoal::Influence => "ui.goal_influence",
        FactionGoal::Recover => "ui.goal_recover",
        FactionGoal::Confront => "ui.goal_confront",
        FactionGoal::Appease => "ui.goal_appease",
    };
    ctx.data.text(text_id)
}

fn integration_label(ctx: &UiContext<'_>, state: IntegrationState) -> String {
    let text_id = match state {
        IntegrationState::Independent => "ui.integration_independent",
        IntegrationState::Trading => "ui.integration_trading",
        IntegrationState::Integrating => "ui.integration_integrating",
        IntegrationState::Integrated => "ui.integration_integrated",
        IntegrationState::Resistant => "ui.integration_resistant",
    };
    ctx.data.text(text_id)
}

fn wilderness_band(ctx: &UiContext<'_>, pressure: &WildernessPressureState) -> String {
    let text_id = match pressure.pressure {
        0..=24 => "ui.wilderness_quiet",
        25..=49 => "ui.wilderness_watchful",
        50..=74 => "ui.wilderness_dangerous",
        _ => "ui.wilderness_lawless",
    };
    ctx.data.text(text_id)
}

fn draw_rival_stats(x: f32, y: f32, width: f32, stats: [(String, i32); 4]) {
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
