//! Compact route and supply controls for the selected-site panel.

use super::{style, virtual_icon_button, UiAction, UiContext};
use crate::data::RouteLevel;
use crate::state::{RouteCondition, RouteRuntimeState};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;
use macroquad_toolkit::ui::HoverTooltip;

pub(super) fn draw_route_section(
    ctx: &UiContext<'_>,
    tooltip: &mut HoverTooltip,
    pointer: Pointer,
    input_enabled: bool,
    actions: &mut Vec<UiAction>,
    content: Rect,
    y: f32,
) -> f32 {
    let Some(site) = ctx.session.selected_site(ctx.data) else {
        return y;
    };
    if !ctx.session.is_known(&site.id) {
        return y;
    }

    style::draw_divider(content.x, y - 6.0, content.w);
    super::section_label(&ctx.data.text("ui.routes_supply"), content.x, y + 10.0);
    draw_text_block(
        &ctx.session.settlement_supply_summary(ctx.data, &site.id),
        content.x,
        y + 20.0,
        content.w - 118.0,
        22.0,
        12.0,
        2.0,
        style::TEXT_DIM,
    );

    let mut next_y = y + 46.0;
    for route in ctx
        .session
        .routes_for_site(&site.id)
        .filter(|route| route.known)
        .take(2)
    {
        draw_route_row(
            ctx,
            RouteRowContext {
                tooltip,
                pointer,
                input_enabled,
                actions,
                content,
                selected_site_id: &site.id,
            },
            next_y,
            route,
        );
        next_y += 24.0;
    }

    let project_status = ctx
        .session
        .regional_project_status(ctx.data, &site.region_id);
    let project_w = 132.0;
    let project_rect = Rect::new(content.right() - project_w, y + 2.0, project_w, 25.0);
    if virtual_icon_button(
        project_rect,
        &ctx.data.text("ui.wardens"),
        style::IconKind::Road,
        input_enabled && project_status.enabled,
        ButtonTone::Secondary,
        pointer,
    ) {
        actions.push(UiAction::CompleteRegionalProject(site.region_id.clone()));
    }
    style::hover_tooltip(
        tooltip,
        "regional_project_wardens",
        project_rect,
        &regional_project_tooltip(ctx, &project_status.reason, project_status.enabled),
        pointer,
    );

    next_y + 2.0
}

fn regional_project_tooltip(ctx: &UiContext<'_>, status_reason: &str, enabled: bool) -> String {
    let Some(project) = ctx
        .data
        .road_balance
        .regional_project(&ctx.data.road_balance.regional_project_id)
    else {
        return status_reason.to_owned();
    };

    let status = if enabled {
        ctx.data.text("ui.ready")
    } else {
        ctx.data
            .text_with("ui.blocked", &[("{reason}", status_reason)])
    };
    let actions = project.action_cost.to_string();
    let cost = project.cost.cost_text_with(ctx.data);
    ctx.data.text_with(
        "ui.regional_tooltip",
        &[
            ("{name}", &project.name),
            ("{description}", &project.description),
            ("{actions}", &actions),
            ("{cost}", &cost),
            ("{status}", &status),
        ],
    )
}

struct RouteRowContext<'a> {
    tooltip: &'a mut HoverTooltip,
    pointer: Pointer,
    input_enabled: bool,
    actions: &'a mut Vec<UiAction>,
    content: Rect,
    selected_site_id: &'a str,
}

fn draw_route_row(
    ctx: &UiContext<'_>,
    row: RouteRowContext<'_>,
    y: f32,
    route: &RouteRuntimeState,
) {
    let RouteRowContext {
        tooltip,
        pointer,
        input_enabled,
        actions,
        content,
        selected_site_id,
    } = row;
    let row_rect = Rect::new(content.x, y - 1.0, content.w, 26.0);
    if pointer.hovering_over(row_rect) {
        draw_rectangle(
            row_rect.x,
            row_rect.y,
            row_rect.w,
            row_rect.h,
            Color::new(0.95, 0.82, 0.50, 0.07),
        );
    }
    draw_surface(
        row_rect,
        &SurfaceStyle::new(Color::new(0.018, 0.032, 0.034, 0.54))
            .with_border(1.0, Color::new(0.58, 0.45, 0.25, 0.18)),
    );
    let other_name = route
        .other_end(selected_site_id)
        .and_then(|site_id| ctx.data.site(site_id))
        .map(|site| site.name.clone())
        .unwrap_or_else(|| ctx.data.text("ui.unknown"));
    style::draw_icon(
        style::IconKind::Road,
        vec2(content.x + 13.0, y + 12.0),
        17.0,
        style::GOLD,
    );
    draw_text_block(
        &other_name,
        content.x + 28.0,
        y + 8.0,
        112.0,
        20.0,
        13.0,
        1.0,
        style::TEXT,
    );

    let level = route_level_label(ctx, route.level);
    draw_ui_text_ex(
        &level,
        content.x + 148.0,
        y + 16.0,
        TextStyle::new(12.0, style::TEXT_DIM).params(),
    );
    draw_circle(
        content.right() - 62.0,
        y + 12.0,
        3.5,
        route_condition_color(route.condition),
    );
    draw_ui_text_ex(
        &route_status_label(ctx, route.condition),
        content.right() - 52.0,
        y + 16.0,
        TextStyle::new(12.0, style::TEXT_DIM).params(),
    );

    let status = ctx.session.route_action_status(ctx.data, &route.id);
    if input_enabled && status.enabled && pointer.released_on(touch_area(row_rect)) {
        actions.push(UiAction::BuildOrUpgradeRoute(route.id.clone()));
    }
    let tooltip_id = format!("route_{}", route.id);
    let route_action = ctx.session.route_action_label(ctx.data, &route.id);
    let tooltip_text = ctx.data.text_with(
        "ui.route_tooltip",
        &[
            ("{condition}", &route_status_label(ctx, route.condition)),
            ("{action}", &route_action),
            ("{reason}", &status.reason),
        ],
    );
    style::hover_tooltip(tooltip, &tooltip_id, row_rect, &tooltip_text, pointer);
}

fn route_level_label(ctx: &UiContext<'_>, level: RouteLevel) -> String {
    let text_id = match level {
        RouteLevel::None => "ui.route_none",
        RouteLevel::Path => "ui.route_path",
        RouteLevel::Road => "ui.route_road",
        RouteLevel::StoneRoad => "ui.route_stone",
    };
    ctx.data.text(text_id)
}

fn route_status_label(ctx: &UiContext<'_>, condition: RouteCondition) -> String {
    match condition {
        RouteCondition::Clear => ctx.data.text("ui.route_good"),
        RouteCondition::Damaged => ctx.data.text("ui.route_damaged"),
        RouteCondition::Blocked => ctx.data.text("ui.route_blocked"),
    }
}

fn route_condition_color(condition: RouteCondition) -> Color {
    match condition {
        RouteCondition::Clear => Color::new(0.18, 0.24, 0.17, 1.0),
        RouteCondition::Damaged => Color::new(0.34, 0.20, 0.12, 1.0),
        RouteCondition::Blocked => Color::new(0.34, 0.12, 0.10, 1.0),
    }
}
