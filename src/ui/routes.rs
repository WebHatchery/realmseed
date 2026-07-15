//! Compact route and supply controls for the selected-site panel.

use super::{style, virtual_icon_button, UiAction, UiContext};
use crate::state::{RouteCondition, RouteRuntimeState};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;
use macroquad_toolkit::ui::HoverTooltip;
use macroquad_toolkit::ui::RectExt;

pub(super) fn draw_route_section(
    ctx: &UiContext<'_>,
    tooltip: &mut HoverTooltip,
    mouse: Vec2,
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
    super::section_label("ROUTES & SUPPLY", content.x, y + 10.0);
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
            tooltip,
            mouse,
            input_enabled,
            actions,
            content,
            next_y,
            route,
            &site.id,
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
        "Wardens",
        style::IconKind::Road,
        input_enabled && project_status.enabled,
        ButtonTone::Secondary,
        mouse,
    ) {
        actions.push(UiAction::CompleteRegionalProject(site.region_id.clone()));
    }
    style::hover_tooltip(
        tooltip,
        "regional_project_wardens",
        project_rect,
        &regional_project_tooltip(ctx, &project_status.reason, project_status.enabled),
        mouse,
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
        "Ready.".to_owned()
    } else {
        format!("Blocked: {}", status_reason)
    };
    format!(
        "{}: {} Cost: {} action and {}. {}",
        project.name,
        project.description,
        project.action_cost,
        project.cost.cost_text(),
        status
    )
}

fn draw_route_row(
    ctx: &UiContext<'_>,
    tooltip: &mut HoverTooltip,
    mouse: Vec2,
    input_enabled: bool,
    actions: &mut Vec<UiAction>,
    content: Rect,
    y: f32,
    route: &RouteRuntimeState,
    selected_site_id: &str,
) {
    let row_rect = Rect::new(content.x, y - 1.0, content.w, 26.0);
    if row_rect.contains_point(mouse) {
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
        .map(|site| site.name.as_str())
        .unwrap_or("Unknown");
    style::draw_icon(
        style::IconKind::Road,
        vec2(content.x + 13.0, y + 12.0),
        17.0,
        style::GOLD,
    );
    draw_text_block(
        other_name,
        content.x + 28.0,
        y + 8.0,
        112.0,
        20.0,
        13.0,
        1.0,
        style::TEXT,
    );

    draw_ui_text_ex(
        route.level.label(),
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
        route_status_label(route.condition),
        content.right() - 52.0,
        y + 16.0,
        TextStyle::new(12.0, style::TEXT_DIM).params(),
    );

    let status = ctx.session.route_action_status(ctx.data, &route.id);
    if input_enabled
        && status.enabled
        && row_rect.contains_point(mouse)
        && is_mouse_button_released(MouseButton::Left)
    {
        actions.push(UiAction::BuildOrUpgradeRoute(route.id.clone()));
    }
    let tooltip_id = format!("route_{}", route.id);
    style::hover_tooltip(
        tooltip,
        &tooltip_id,
        row_rect,
        &format!(
            "{} route. {}: {}",
            route.condition.label(),
            ctx.session.route_action_label(ctx.data, &route.id),
            status.reason
        ),
        mouse,
    );
}

fn route_status_label(condition: RouteCondition) -> &'static str {
    match condition {
        RouteCondition::Clear => "Good",
        RouteCondition::Damaged => "Damaged",
        RouteCondition::Blocked => "Blocked",
    }
}

fn route_condition_color(condition: RouteCondition) -> Color {
    match condition {
        RouteCondition::Clear => Color::new(0.18, 0.24, 0.17, 1.0),
        RouteCondition::Damaged => Color::new(0.34, 0.20, 0.12, 1.0),
        RouteCondition::Blocked => Color::new(0.34, 0.12, 0.10, 1.0),
    }
}
