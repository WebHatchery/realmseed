//! Realm overview and council guidance surfaces.

use super::{section_label, style, virtual_icon_button, UiAction, UiContext};
use crate::data::SiteCategory;
use crate::state::SettlementStatus;
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;
use macroquad_toolkit::ui::RectExt;

#[derive(Debug, Clone)]
struct Recommendation {
    title: String,
    detail: String,
    button_label: String,
    action: UiAction,
    enabled: bool,
    show_button: bool,
    tone: ButtonTone,
}

pub(super) fn draw_realm_overview(
    ctx: &UiContext<'_>,
    _mouse: Vec2,
    _input_enabled: bool,
    _actions: &mut Vec<UiAction>,
) {
    let rect = super::left_panel_rect(ctx);
    style::draw_panel(rect);

    let content = rect.inset(17.0);
    style::draw_panel_title("REALM OVERVIEW", content.x, content.y + 16.0);
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
        content.x,
        y,
        content.w,
        "Stability",
        health.stability,
        Color::new(0.40, 0.70, 0.34, 1.0),
    );
    y += 31.0;
    draw_stat_bar(
        content.x,
        y,
        content.w,
        "Loyalty",
        health.loyalty,
        dark::ACCENT,
    );
    y += 31.0;
    draw_stat_bar(
        content.x,
        y,
        content.w,
        "Supply",
        health.supply,
        Color::new(0.75, 0.68, 0.43, 1.0),
    );
    y += 31.0;
    draw_stat_bar(
        content.x,
        y,
        content.w,
        "Danger",
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
        &format!("{} of {} sites known", known, total_sites),
    );
    y += 24.0;
    draw_fact_row(
        content.x,
        y,
        style::IconKind::Crown,
        &format!("{} active settlement", active_settlements),
    );
    y += 24.0;
    draw_fact_row(
        content.x,
        y,
        style::IconKind::Danger,
        &format!("{} active issues", ctx.session.active_issues.len()),
    );

    y += 38.0;
    style::draw_divider(content.x, y, content.w);
    y += 26.0;
    let recommendation = recommendation_for(ctx);
    section_label("ADVISOR'S COUNSEL", content.x, y);
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
    mouse: Vec2,
    input_enabled: bool,
    actions: &mut Vec<UiAction>,
) {
    let rect = super::footer_rect(ctx);
    style::draw_panel(rect);

    let recommendation = recommendation_for(ctx);
    let fallback_guidance = ctx.session.guidance_text();
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
        "COUNCIL GUIDANCE",
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
        "SEASON LOG",
        center.x,
        center.y + 26.0,
        TextStyle::new(14.5, style::GOLD).params(),
    );
    draw_log_lines(ctx, center.x, center.y + 44.0, center.w);

    if recommendation.show_button {
        if virtual_icon_button(
            Rect::new(right.x, right.y + 10.0, right.w, 38.0),
            &recommendation.button_label.to_uppercase(),
            recommendation_icon(&recommendation.action),
            input_enabled && recommendation.enabled,
            recommendation.tone,
            mouse,
        ) {
            actions.push(recommendation.action);
        }
    } else {
        draw_text_block(
            &recommendation.title,
            right.x,
            right.y + 12.0,
            right.w,
            36.0,
            14.0,
            2.0,
            style::TEXT_BRIGHT,
        );
    }
    draw_ui_text_ex(
        &format!(
            "{} Actions Remaining",
            ctx.session.council_actions_remaining
        ),
        right.x + right.w * 0.5 - 58.0,
        right.y + 66.0,
        TextStyle::new(12.5, style::TEXT_DIM).params(),
    );
}

fn recommendation_icon(action: &UiAction) -> style::IconKind {
    match action {
        UiAction::ScoutSelectedSite | UiAction::SelectSite(_) => style::IconKind::Compass,
        UiAction::FoundCamp | UiAction::UpgradeSelectedSettlement => style::IconKind::Castle,
        UiAction::AdvanceSeason => style::IconKind::Actions,
        UiAction::ToggleChronicle => style::IconKind::Crown,
        UiAction::SetMapOverlay(_) => style::IconKind::Road,
        _ => style::IconKind::Crown,
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
        draw_text_block(
            &format!("* {} - {}", row.label, row.detail),
            x,
            line_y,
            width,
            24.0,
            12.0,
            1.0,
            style::TEXT_DIM,
        );
        line_y += 22.0;
    }
}

fn recommendation_for(ctx: &UiContext<'_>) -> Recommendation {
    if ctx.session.pending_event.is_some() {
        return Recommendation {
            title: "Answer the council matter".to_owned(),
            detail:
                "A live petition is waiting for a decision before the realm can breathe easily."
                    .to_owned(),
            button_label: "Review Petition".to_owned(),
            action: UiAction::DeferEvent,
            enabled: false,
            show_button: true,
            tone: ButtonTone::Primary,
        };
    }

    if ctx.session.council_actions_remaining <= 0 {
        return Recommendation {
            title: "Council actions spent".to_owned(),
            detail: "End the season to resolve production, roads, rival moves, and new warnings."
                .to_owned(),
            button_label: "End Season".to_owned(),
            action: UiAction::AdvanceSeason,
            enabled: true,
            show_button: true,
            tone: ButtonTone::Positive,
        };
    }

    if ctx
        .session
        .is_adjacent_unknown(ctx.data, &ctx.session.selected_site_id)
    {
        let scout_status = ctx.session.scout_status(ctx.data);
        let site_name = ctx
            .session
            .selected_site(ctx.data)
            .map(|site| site.name.as_str())
            .unwrap_or("the rumor");
        return Recommendation {
            title: format!("Scout {}", site_name),
            detail: scout_status.reason.clone(),
            button_label: "Scout Site".to_owned(),
            action: UiAction::ScoutSelectedSite,
            enabled: scout_status.enabled,
            show_button: false,
            tone: ButtonTone::Primary,
        };
    }

    if let Some(site) = ctx
        .data
        .sites
        .iter()
        .find(|site| ctx.session.is_adjacent_unknown(ctx.data, &site.id))
    {
        return Recommendation {
            title: "Reveal the next frontier".to_owned(),
            detail: "Question markers show places your scouts can reach from known roads."
                .to_owned(),
            button_label: "Select Scout Target".to_owned(),
            action: UiAction::SelectSite(site.id.clone()),
            enabled: true,
            show_button: true,
            tone: ButtonTone::Primary,
        };
    }

    if ctx
        .session
        .settlements
        .iter()
        .filter(|settlement| settlement.status == SettlementStatus::Active)
        .count()
        < 2
    {
        if let Some(site) = ctx.data.sites.iter().find(|site| {
            site.category == SiteCategory::Settlement
                && ctx.session.is_known(&site.id)
                && site.owner.is_none()
                && ctx.session.settlement_at_site(&site.id).is_none()
        }) {
            let selected = ctx.session.selected_site_id == site.id;
            return Recommendation {
                title: "Found the first outpost".to_owned(),
                detail: "A second settlement turns Greenvale from a seat into a realm.".to_owned(),
                button_label: if selected {
                    "Found Camp"
                } else {
                    "Select Site"
                }
                .to_owned(),
                action: if selected {
                    UiAction::FoundCamp
                } else {
                    UiAction::SelectSite(site.id.clone())
                },
                enabled: true,
                show_button: true,
                tone: ButtonTone::Positive,
            };
        }
    }

    if ctx.session.routes.iter().any(|route| {
        route.known
            && route.level == crate::data::RouteLevel::None
            && ctx.session.route_action_status(ctx.data, &route.id).enabled
    }) {
        return Recommendation {
            title: "Bind the realm with roads".to_owned(),
            detail: "A built path reduces isolation and lets the capital support frontier sites."
                .to_owned(),
            button_label: "Open Routes".to_owned(),
            action: UiAction::SetMapOverlay(super::MapOverlay::Supply),
            enabled: true,
            show_button: true,
            tone: ButtonTone::Primary,
        };
    }

    if ctx.session.active_issues.is_empty() {
        Recommendation {
            title: "Let the season turn".to_owned(),
            detail: "Production, roads, rivals, and wilderness pressure will all resolve into the chronicle.".to_owned(),
            button_label: "Advance Season".to_owned(),
            action: UiAction::AdvanceSeason,
            enabled: true,
            show_button: true,
            tone: ButtonTone::Positive,
        }
    } else {
        Recommendation {
            title: "Stabilize active issues".to_owned(),
            detail: "Warnings tied to settlements and roads can grow if seasons pass unattended."
                .to_owned(),
            button_label: "Open Chronicle".to_owned(),
            action: UiAction::ToggleChronicle,
            enabled: true,
            show_button: true,
            tone: ButtonTone::Danger,
        }
    }
}

fn realm_subtitle(ctx: &UiContext<'_>) -> String {
    if let Some(ambition_id) = &ctx.session.selected_ambition_id {
        if let Some(ambition) = ctx.data.campaign_balance.ambition(ambition_id) {
            return ambition.name.clone();
        }
    }
    "Frontier charter under the crown".to_owned()
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

fn draw_stat_bar(x: f32, y: f32, width: f32, label: &str, value: i32, color: Color) {
    draw_ui_text_ex(label, x, y, TextStyle::new(13.0, style::TEXT).params());
    draw_ui_text_ex(
        &format!("{}/100", value.clamp(0, 100)),
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
