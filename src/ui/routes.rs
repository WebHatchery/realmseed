//! Compact route and supply controls for the selected-site panel.

use super::{virtual_button, UiAction, UiContext};
use crate::state::{RouteCondition, RouteRuntimeState};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;

pub(super) fn draw_route_section(
    ctx: &UiContext<'_>,
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

    draw_text_ex(
        "Routes & Supply",
        content.x,
        y,
        TextStyle::new(18.0, dark::TEXT_BRIGHT).params(),
    );
    draw_text_block(
        &ctx.session.settlement_supply_summary(ctx.data, &site.id),
        content.x,
        y + 12.0,
        content.w,
        28.0,
        13.0,
        2.0,
        dark::TEXT_DIM,
    );

    let mut next_y = y + 42.0;
    for route in ctx
        .session
        .routes_for_site(&site.id)
        .filter(|route| route.known)
        .take(3)
    {
        draw_route_row(
            ctx,
            mouse,
            input_enabled,
            actions,
            content,
            next_y,
            route,
            &site.id,
        );
        next_y += 30.0;
    }

    let project_status = ctx
        .session
        .regional_project_status(ctx.data, &site.region_id);
    let project_w = 132.0;
    if virtual_button(
        Rect::new(content.right() - project_w, y, project_w, 28.0),
        "Trail Wardens",
        input_enabled && project_status.enabled,
        ButtonTone::Secondary,
        mouse,
    ) {
        actions.push(UiAction::CompleteRegionalProject(site.region_id.clone()));
    }

    next_y + 4.0
}

fn draw_route_row(
    ctx: &UiContext<'_>,
    mouse: Vec2,
    input_enabled: bool,
    actions: &mut Vec<UiAction>,
    content: Rect,
    y: f32,
    route: &RouteRuntimeState,
    selected_site_id: &str,
) {
    let other_name = route
        .other_end(selected_site_id)
        .and_then(|site_id| ctx.data.site(site_id))
        .map(|site| site.name.as_str())
        .unwrap_or("Unknown");
    draw_text_block(
        other_name,
        content.x,
        y + 4.0,
        128.0,
        20.0,
        13.0,
        1.0,
        dark::TEXT,
    );

    draw_badge(
        Rect::new(content.x + 134.0, y, 82.0, 24.0),
        route.level.label(),
        route_level_color(route),
        dark::TEXT,
    );
    draw_badge(
        Rect::new(content.x + 222.0, y, 72.0, 24.0),
        route.condition.label(),
        route_condition_color(route.condition),
        dark::TEXT,
    );

    let status = ctx.session.route_action_status(ctx.data, &route.id);
    if virtual_button(
        Rect::new(content.x + 300.0, y, content.w - 300.0, 24.0),
        &ctx.session.route_action_label(ctx.data, &route.id),
        input_enabled && status.enabled,
        ButtonTone::Primary,
        mouse,
    ) {
        actions.push(UiAction::BuildOrUpgradeRoute(route.id.clone()));
    }
}

fn route_level_color(route: &RouteRuntimeState) -> Color {
    match route.level {
        crate::data::RouteLevel::None => Color::new(0.16, 0.15, 0.13, 1.0),
        crate::data::RouteLevel::Path => Color::new(0.22, 0.20, 0.15, 1.0),
        crate::data::RouteLevel::Road => Color::new(0.28, 0.22, 0.14, 1.0),
        crate::data::RouteLevel::StoneRoad => Color::new(0.30, 0.30, 0.27, 1.0),
    }
}

fn route_condition_color(condition: RouteCondition) -> Color {
    match condition {
        RouteCondition::Clear => Color::new(0.18, 0.24, 0.17, 1.0),
        RouteCondition::Damaged => Color::new(0.34, 0.20, 0.12, 1.0),
        RouteCondition::Blocked => Color::new(0.34, 0.12, 0.10, 1.0),
    }
}
