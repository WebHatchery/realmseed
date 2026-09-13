//! Realm overview and council guidance surfaces.

use super::{section_label, style, UiAction, UiContext};
use crate::data::GameData;
use crate::state::{AdvisorPriority, SettlementStatus};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;
use macroquad_toolkit::ui::RectExt;

#[derive(Debug, Clone)]
struct Recommendation {
    title: String,
    detail: String,
}

pub(super) fn draw_realm_overview(ctx: &UiContext<'_>) {
    let rect = super::left_panel_rect(ctx);
    style::draw_panel(rect);

    let content = rect.inset(17.0);
    style::draw_panel_title(
        &ctx.data.text("ui.realm_overview"),
        content.x,
        content.y + 16.0,
    );
    draw_realm_crest(content.x + 38.0, content.y + 68.0);
    draw_text_block(
        &realm_subtitle(ctx),
        content.x + 80.0,
        content.y + 48.0,
        content.w - 82.0,
        48.0,
        14.0,
        2.0,
        style::TEXT,
    );

    let health = realm_health(ctx);
    let mut y = content.y + 132.0;
    draw_stat_bar(
        ctx,
        content.x,
        y,
        content.w,
        &ctx.data.text("ui.stability"),
        health.stability,
        Color::new(0.40, 0.70, 0.34, 1.0),
    );
    y += 31.0;
    draw_stat_bar(
        ctx,
        content.x,
        y,
        content.w,
        &ctx.data.text("ui.loyalty"),
        health.loyalty,
        dark::ACCENT,
    );
    y += 31.0;
    draw_stat_bar(
        ctx,
        content.x,
        y,
        content.w,
        &ctx.data.text("ui.supply"),
        health.supply,
        Color::new(0.75, 0.68, 0.43, 1.0),
    );
    y += 31.0;
    draw_stat_bar(
        ctx,
        content.x,
        y,
        content.w,
        &ctx.data.text("ui.danger"),
        health.danger,
        Color::new(0.86, 0.33, 0.22, 1.0),
    );

    y += 42.0;
    style::draw_divider(content.x, y, content.w);
    y += 22.0;
    let active_settlements = ctx
        .session
        .settlements
        .iter()
        .filter(|settlement| settlement.status == SettlementStatus::Active)
        .count();
    let known = ctx.session.known_site_count();
    let total_sites = ctx.data.sites.len();
    draw_fact_row(
        content.x,
        y,
        style::IconKind::Castle,
        &ctx.data.text_with(
            "ui.known_sites",
            &[
                ("{known}", &known.to_string()),
                ("{total}", &total_sites.to_string()),
            ],
        ),
    );
    y += 24.0;
    draw_fact_row(
        content.x,
        y,
        style::IconKind::Crown,
        &ctx.data.text_with(
            "ui.active_settlement",
            &[("{count}", &active_settlements.to_string())],
        ),
    );
    y += 24.0;
    draw_fact_row(
        content.x,
        y,
        style::IconKind::Danger,
        &ctx.data.text_with(
            "ui.active_issues",
            &[("{count}", &ctx.session.active_issues.len().to_string())],
        ),
    );

    y += 38.0;
    style::draw_divider(content.x, y, content.w);
    y += 26.0;
    let recommendation = recommendation_for(ctx);
    section_label(&ctx.data.text("ui.advisor_counsel"), content.x, y);
    let button_y = rect.bottom() - 64.0;
    draw_text_block(
        &recommendation.title,
        content.x,
        y + 20.0,
        content.w,
        22.0,
        15.0,
        2.0,
        style::TEXT_BRIGHT,
    );
    let detail_y = y + 42.0;
    let detail_h = (button_y - detail_y - 6.0).clamp(0.0, 50.0);
    if detail_h > 12.0 {
        draw_text_block(
            &recommendation.detail,
            content.x,
            detail_y,
            content.w,
            detail_h,
            12.5,
            2.0,
            style::TEXT_DIM,
        );
    }
    draw_compass_watermark(rect.center() + vec2(0.0, rect.h * 0.39));
}

pub(super) fn draw_council_footer(
    ctx: &UiContext<'_>,
    input_enabled: bool,
    actions: &mut Vec<UiAction>,
) {
    let rect = super::footer_rect(ctx);
    style::draw_panel(rect);

    let recommendation = recommendation_for(ctx);
    let fallback_guidance = ctx.session.guidance_text(ctx.data);
    let content = rect.inset(16.0);
    let left_w = content.w * 0.38;
    let center_w = content.w * 0.32;
    let action_w = content.w - left_w - center_w - 36.0;
    let left = Rect::new(content.x, content.y, left_w, content.h);
    let center = Rect::new(left.right() + 18.0, content.y, center_w, content.h);
    let right = Rect::new(center.right() + 18.0, content.y, action_w, content.h);

    style::draw_icon(
        style::IconKind::Compass,
        vec2(left.x + 28.0, left.y + 32.0),
        52.0,
        style::GOLD,
    );
    draw_ui_text_ex(
        &ctx.data.text("ui.council_guidance"),
        left.x + 70.0,
        left.y + 26.0,
        TextStyle::new(14.5, style::GOLD).params(),
    );
    let latest = ctx
        .session
        .last_season_rows
        .first()
        .map(|row| row.detail.as_str())
        .unwrap_or(fallback_guidance.as_str());
    draw_text_block(
        &format!(
            "{} - {}\nLast season: {}",
            recommendation.title, recommendation.detail, latest
        ),
        left.x + 70.0,
        left.y + 36.0,
        left.w - 74.0,
        left.h - 36.0,
        13.0,
        3.0,
        style::TEXT_DIM,
    );

    style::draw_vertical_divider(center.x - 9.0, center.y + 4.0, center.h - 8.0);
    draw_ui_text_ex(
        &ctx.data.text("ui.season_log"),
        center.x,
        center.y + 26.0,
        TextStyle::new(14.5, style::GOLD).params(),
    );
    draw_log_lines(ctx, center.x, center.y + 44.0, center.w);

    draw_quick_actions(ctx.data, right, input_enabled, ctx.pointer, actions);
    draw_ui_text_ex(
        &ctx.data.text_with(
            "ui.actions_remaining",
            &[(
                "{count}",
                &ctx.session.council_actions_remaining.to_string(),
            )],
        ),
        right.x + right.w * 0.5 - 58.0,
        right.y + 66.0,
        TextStyle::new(12.5, style::TEXT_DIM).params(),
    );
}

fn draw_quick_actions(
    data: &GameData,
    rect: Rect,
    input_enabled: bool,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let gap = 6.0;
    let button_w = (rect.w - gap * 3.0) / 4.0;
    let controls = [
        (data.text("ui.pause"), UiAction::OpenPauseMenu),
        (data.text("ui.chronicle"), UiAction::ToggleChronicle),
        (data.text("ui.factions"), UiAction::ToggleFactionPanel),
        (data.text("ui.advance"), UiAction::AdvanceSeason),
    ];
    for (index, (label, action)) in controls.into_iter().enumerate() {
        let button = Rect::new(
            rect.x + index as f32 * (button_w + gap),
            rect.y + 8.0,
            button_w,
            42.0,
        );
        if super::virtual_button(
            button,
            &label,
            input_enabled,
            if matches!(action, UiAction::AdvanceSeason) {
                ButtonTone::Primary
            } else {
                ButtonTone::Secondary
            },
            pointer,
        ) {
            actions.push(action);
        }
    }
}

fn draw_realm_crest(x: f32, y: f32) {
    let top = vec2(x, y - 28.0);
    let right = vec2(x + 28.0, y - 16.0);
    let bottom = vec2(x, y + 30.0);
    let left = vec2(x - 28.0, y - 16.0);
    draw_triangle(top, right, bottom, Color::new(0.05, 0.22, 0.17, 0.94));
    draw_triangle(top, bottom, left, Color::new(0.04, 0.17, 0.14, 0.94));
    draw_line(top.x, top.y, right.x, right.y, 1.3, style::GOLD);
    draw_line(right.x, right.y, bottom.x, bottom.y, 1.3, style::GOLD);
    draw_line(bottom.x, bottom.y, left.x, left.y, 1.3, style::GOLD);
    draw_line(left.x, left.y, top.x, top.y, 1.3, style::GOLD);
    style::draw_icon(style::IconKind::Tree, vec2(x, y - 3.0), 38.0, style::GOLD);
}

fn draw_fact_row(x: f32, y: f32, icon: style::IconKind, text: &str) {
    style::draw_icon(icon, vec2(x + 8.0, y - 4.0), 18.0, style::GOLD);
    draw_ui_text_ex(
        text,
        x + 24.0,
        y,
        TextStyle::new(13.0, style::TEXT).params(),
    );
}

fn draw_compass_watermark(center: Vec2) {
    draw_circle_lines(
        center.x,
        center.y,
        33.0,
        1.0,
        Color::new(0.74, 0.58, 0.32, 0.18),
    );
    style::draw_icon(
        style::IconKind::Compass,
        center,
        70.0,
        Color::new(0.74, 0.58, 0.32, 0.18),
    );
}

fn draw_log_lines(ctx: &UiContext<'_>, x: f32, y: f32, width: f32) {
    let mut line_y = y;
    if ctx.session.last_season_rows.is_empty() {
        draw_text_block(
            &ctx.session.last_season_summary,
            x,
            line_y,
            width,
            38.0,
            12.5,
            2.0,
            style::TEXT_DIM,
        );
        return;
    }
    for row in ctx.session.last_season_rows.iter().take(2) {
        let detail = ctx.data.text_with(
            "ui.advisor_log_line",
            &[("{label}", &row.label), ("{detail}", &row.detail)],
        );
        draw_text_block(&detail, x, line_y, width, 24.0, 12.0, 1.0, style::TEXT_DIM);
        line_y += 22.0;
    }
}

fn recommendation_for(ctx: &UiContext<'_>) -> Recommendation {
    match ctx.session.advisor_priority(ctx.data) {
        AdvisorPriority::AnswerEvent => Recommendation {
            title: ctx.data.text("ui.advisor_answer_title"),
            detail: ctx.data.text("ui.advisor_scouting"),
        },
        AdvisorPriority::SpendActions => Recommendation {
            title: ctx.data.text("ui.advisor_actions_title"),
            detail: ctx.data.text("ui.advisor_actions_spent"),
        },
        AdvisorPriority::ScoutSelected => {
            let scout_status = ctx.session.scout_status(ctx.data);
            let site_name = ctx
                .session
                .selected_site(ctx.data)
                .map(|site| site.name.clone())
                .unwrap_or_else(|| ctx.data.text("ui.rumor"));
            Recommendation {
                title: ctx
                    .data
                    .text_with("ui.scout_title", &[("{site}", &site_name)]),
                detail: scout_status.reason.clone(),
            }
        }
        AdvisorPriority::RevealFrontier => Recommendation {
            title: ctx.data.text("ui.advisor_reveal_title"),
            detail: ctx.data.text("ui.advisor_reveal"),
        },
        AdvisorPriority::FoundCamp => Recommendation {
            title: ctx.data.text("ui.advisor_founding_title"),
            detail: ctx.data.text("ui.advisor_founding"),
        },
        AdvisorPriority::BuildRoad => Recommendation {
            title: ctx.data.text("ui.advisor_roads_title"),
            detail: ctx.data.text("ui.advisor_roads"),
        },
        AdvisorPriority::AdvanceSeason => Recommendation {
            title: ctx.data.text("ui.advisor_season_title"),
            detail: ctx.data.text("ui.advisor_season"),
        },
        AdvisorPriority::ResolveIssues => Recommendation {
            title: ctx.data.text("ui.advisor_issues_title"),
            detail: ctx.data.text("ui.advisor_issues"),
        },
    }
}

fn realm_subtitle(ctx: &UiContext<'_>) -> String {
    if let Some(ambition_id) = &ctx.session.selected_ambition_id {
        if let Some(ambition) = ctx.data.campaign_balance.ambition(ambition_id) {
            return ambition.name.clone();
        }
    }
    ctx.data.text("ui.frontier_charter")
}

#[derive(Debug, Clone, Copy)]
struct RealmHealth {
    stability: i32,
    loyalty: i32,
    supply: i32,
    danger: i32,
}

fn realm_health(ctx: &UiContext<'_>) -> RealmHealth {
    let active = ctx
        .session
        .settlements
        .iter()
        .filter(|settlement| settlement.status == SettlementStatus::Active)
        .collect::<Vec<_>>();
    if active.is_empty() {
        return RealmHealth {
            stability: 0,
            loyalty: 0,
            supply: 0,
            danger: 0,
        };
    }

    let count = active.len() as i32;
    let stability = active
        .iter()
        .map(|settlement| settlement.stability)
        .sum::<i32>()
        / count;
    let loyalty = active
        .iter()
        .map(|settlement| settlement.loyalty)
        .sum::<i32>()
        / count;
    let danger = active
        .iter()
        .map(|settlement| settlement.danger)
        .sum::<i32>()
        / count;
    let connected = active
        .iter()
        .filter(|settlement| {
            ctx.session
                .is_site_in_capital_network(ctx.data, &settlement.location_id)
        })
        .count() as i32;
    RealmHealth {
        stability,
        loyalty,
        supply: (connected * 100 / count).clamp(0, 100),
        danger,
    }
}

fn draw_stat_bar(
    ctx: &UiContext<'_>,
    x: f32,
    y: f32,
    width: f32,
    label: &str,
    value: i32,
    color: Color,
) {
    draw_ui_text_ex(label, x, y, TextStyle::new(13.0, style::TEXT).params());
    let value_text = ctx.data.text_with(
        "ui.stat_value",
        &[("{value}", &value.clamp(0, 100).to_string())],
    );
    draw_ui_text_ex(
        &value_text,
        x + width - 56.0,
        y,
        TextStyle::new(12.5, style::TEXT_BRIGHT).params(),
    );
    let bar = Rect::new(x, y + 7.0, width, 7.0);
    draw_rectangle(
        bar.x,
        bar.y,
        bar.w,
        bar.h,
        Color::new(0.006, 0.014, 0.016, 0.78),
    );
    draw_rectangle(
        bar.x,
        bar.y,
        bar.w * (value.clamp(0, 100) as f32 / 100.0),
        bar.h,
        color,
    );
}
