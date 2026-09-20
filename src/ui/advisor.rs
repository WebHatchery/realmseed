//! Realm overview and council guidance surfaces.

use super::{section_label, style, UiAction, UiContext};
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

pub(super) fn draw_realm_summary_overlay(ctx: &UiContext<'_>, actions: &mut Vec<UiAction>) {
    let screen = super::screen_rect(ctx);
    draw_rectangle(
        screen.x,
        screen.y,
        screen.w,
        screen.h,
        Color::new(0.004, 0.010, 0.012, 0.72),
    );
    let rect = super::centered_modal_rect(ctx, 720.0, 520.0);
    style::draw_panel(rect);

    let content = rect.inset(17.0);
    style::draw_panel_title(
        &ctx.data.text("ui.realm_overview"),
        content.x,
        content.y + 16.0,
    );
    if super::virtual_button(
        Rect::new(content.right() - 82.0, content.y, 82.0, 32.0),
        &ctx.data.text("ui.close"),
        true,
        ButtonTone::Secondary,
        ctx.pointer,
    ) {
        actions.push(UiAction::ToggleRealmSummary);
    }
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

    y += 30.0;
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

    y += 26.0;
    style::draw_divider(content.x, y, content.w);
    y += 26.0;
    let recommendation = recommendation_for(ctx);
    section_label(&ctx.data.text("ui.advisor_counsel"), content.x, y);
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
    draw_text_block(
        &recommendation.detail,
        content.x,
        y + 42.0,
        content.w,
        42.0,
        12.5,
        2.0,
        style::TEXT_DIM,
    );
}

pub(super) fn draw_council_footer(
    ctx: &UiContext<'_>,
    input_enabled: bool,
    actions: &mut Vec<UiAction>,
) {
    let rect = super::footer_rect(ctx);
    style::draw_panel(rect);

    let recommendation = recommendation_for(ctx);
    let content = rect.inset(16.0);
    let advance_w = if ctx.ui.logical_width < 1040.0 {
        190.0
    } else {
        226.0
    };
    let utility_w = (content.w * 0.47).min(content.w - advance_w - 190.0);
    let decision_w = (content.w - utility_w - advance_w - 24.0).max(140.0);
    let utility = Rect::new(content.x, content.y, utility_w, content.h);
    let decision = Rect::new(utility.right() + 12.0, content.y, decision_w, content.h);
    let advance = Rect::new(decision.right() + 12.0, content.y, advance_w, content.h);

    draw_quick_actions(ctx, utility, input_enabled, actions);
    style::draw_vertical_divider(decision.x - 6.0, decision.y + 3.0, decision.h - 6.0);
    draw_ui_text_ex(
        &recommendation.title,
        decision.x,
        decision.y + 18.0,
        TextStyle::new(13.5, style::TEXT_BRIGHT).params(),
    );
    let detail = if ctx.session.pending_event.is_some() {
        ctx.data.text("ui.resolve_event_first")
    } else {
        ctx.data.text_with(
            "ui.actions_remaining",
            &[(
                "{count}",
                &ctx.session.council_actions_remaining.to_string(),
            )],
        )
    };
    draw_text_block(
        &detail,
        decision.x,
        decision.y + 25.0,
        decision.w,
        decision.h - 25.0,
        11.5,
        2.0,
        style::TEXT_DIM,
    );

    let advance_enabled = input_enabled && ctx.session.pending_event.is_none();
    let advance_tone = if matches!(
        ctx.session.advisor_priority(ctx.data),
        AdvisorPriority::AdvanceSeason
    ) || ctx.session.council_actions_remaining == 0
    {
        ButtonTone::Primary
    } else {
        ButtonTone::Secondary
    };
    if super::virtual_icon_button(
        Rect::new(advance.x, advance.y + 2.0, advance.w, 42.0),
        &ctx.data.text("ui.advance_season"),
        style::IconKind::Compass,
        advance_enabled,
        advance_tone,
        ctx.pointer,
    ) {
        actions.push(UiAction::AdvanceSeason);
    }
}

fn draw_quick_actions(
    ctx: &UiContext<'_>,
    rect: Rect,
    input_enabled: bool,
    actions: &mut Vec<UiAction>,
) {
    let gap = 5.0;
    let button_w = (rect.w - gap * 3.0) / 4.0;
    let controls = [
        (
            ctx.data.text("ui.pause"),
            style::IconKind::Actions,
            UiAction::OpenPauseMenu,
        ),
        (
            ctx.data.text("ui.realm_summary"),
            style::IconKind::Crown,
            UiAction::ToggleRealmSummary,
        ),
        (
            ctx.data.text("ui.chronicle"),
            style::IconKind::Compass,
            UiAction::ToggleChronicle,
        ),
        (
            ctx.data.text("ui.factions"),
            style::IconKind::Danger,
            UiAction::ToggleFactionPanel,
        ),
    ];
    for (index, (label, icon, action)) in controls.into_iter().enumerate() {
        let button = Rect::new(
            rect.x + index as f32 * (button_w + gap),
            rect.y + 2.0,
            button_w,
            42.0,
        );
        let activated = if ctx.ui.logical_width < 1040.0 {
            super::virtual_button(
                button,
                &label,
                input_enabled,
                ButtonTone::Secondary,
                ctx.pointer,
            )
        } else {
            super::virtual_icon_button(
                button,
                &label,
                icon,
                input_enabled,
                ButtonTone::Secondary,
                ctx.pointer,
            )
        };
        if activated {
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
