//! Strategic terrain map rendering and site picking.

use super::{
    color_from_array, map_panel_rect, map_sites, map_terrain, style, virtual_button,
    virtual_icon_button, MapOverlay, UiAction, UiContext,
};
use crate::data::{RouteLevel, SiteDef};
use crate::state::{RouteCondition, RouteRuntimeState};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;
use macroquad_toolkit::ui::HoverTooltip;
use macroquad_toolkit::ui::RectExt;

pub(super) fn draw_map_panel(
    ctx: &UiContext<'_>,
    tooltip: &mut HoverTooltip,
    input_enabled: bool,
    actions: &mut Vec<UiAction>,
) {
    let rect = map_panel_rect(ctx);
    draw_map_backdrop(rect);

    let map_rect = rect.inset(0.0);
    let view = MapView::new(ctx, map_rect);
    map_terrain::draw_terrain(ctx, &view);
    draw_map_overlay(ctx, &view);
    draw_region_labels(ctx, &view);
    draw_roads(ctx, &view);
    map_sites::draw_sites(ctx, &view);
    draw_map_vignette(map_rect);
    draw_overlay_tabs(ctx, rect, ctx.pointer, input_enabled, actions);
    draw_map_controls(rect, ctx.pointer, input_enabled, actions);
    draw_map_caption(ctx, rect);

    if input_enabled && ctx.pointer.released {
        if let Some(site_id) = map_sites::picked_site_id(ctx, &view, ctx.pointer.position) {
            actions.push(UiAction::SelectSite(site_id));
        }
    }

    let mode_hint = match ctx.map_overlay {
        MapOverlay::Realm => ctx.data.text("ui.map_realm_hint"),
        MapOverlay::Supply => ctx.data.text("ui.map_supply_hint"),
        MapOverlay::Danger => ctx.data.text("ui.map_danger_hint"),
    };
    style::hover_tooltip(
        tooltip,
        "map_overlay_hint",
        map_rect,
        &mode_hint,
        ctx.pointer,
    );
}

fn draw_map_controls(
    panel_rect: Rect,
    pointer: Pointer,
    input_enabled: bool,
    actions: &mut Vec<UiAction>,
) {
    let button_size = 30.0;
    let gap = 4.0;
    let start_x = panel_rect.right() - button_size * 2.0 - gap - 16.0;
    let y = panel_rect.y + 16.0;
    if virtual_button(
        Rect::new(start_x, y, button_size, button_size),
        "−",
        input_enabled,
        ButtonTone::Secondary,
        pointer,
    ) {
        actions.push(UiAction::ZoomMapOut);
    }
    if virtual_button(
        Rect::new(start_x + button_size + gap, y, button_size, button_size),
        "+",
        input_enabled,
        ButtonTone::Secondary,
        pointer,
    ) {
        actions.push(UiAction::ZoomMapIn);
    }
}

fn draw_map_caption(ctx: &UiContext<'_>, panel_rect: Rect) {
    draw_ui_text_ex(
        &ctx.data.text("ui.strategic_view"),
        panel_rect.x + 18.0,
        panel_rect.y + 74.0,
        TextStyle::new(11.0, Color::new(0.86, 0.72, 0.42, 0.82)).params(),
    );
    draw_ui_text_ex(
        &ctx.data.text("ui.drag_map"),
        panel_rect.x + 18.0,
        panel_rect.bottom() - 14.0,
        TextStyle::new(11.5, Color::new(0.80, 0.84, 0.72, 0.68)).params(),
    );
}

fn draw_overlay_tabs(
    ctx: &UiContext<'_>,
    panel_rect: Rect,
    pointer: Pointer,
    input_enabled: bool,
    actions: &mut Vec<UiAction>,
) {
    let overlays = [MapOverlay::Realm, MapOverlay::Supply, MapOverlay::Danger];
    let tab_w = if panel_rect.w < 570.0 { 86.0 } else { 104.0 };
    let tab_h = 34.0;
    let start_x = panel_rect.x + (panel_rect.w - (tab_w + 1.0) * overlays.len() as f32) * 0.5;
    for (index, overlay) in overlays.iter().enumerate() {
        let rect = Rect::new(
            start_x + index as f32 * (tab_w + 1.0),
            panel_rect.y + 14.0,
            tab_w,
            tab_h,
        );
        let tone = if *overlay == ctx.map_overlay {
            ButtonTone::Primary
        } else {
            ButtonTone::Secondary
        };
        if virtual_icon_button(
            rect,
            &overlay_label(ctx, *overlay),
            overlay_icon(*overlay),
            input_enabled,
            tone,
            pointer,
        ) {
            actions.push(UiAction::SetMapOverlay(*overlay));
        }
    }
}

fn overlay_label(ctx: &UiContext<'_>, overlay: MapOverlay) -> String {
    match overlay {
        MapOverlay::Realm => ctx.data.text("ui.overlay_realm"),
        MapOverlay::Supply => ctx.data.text("ui.overlay_supply"),
        MapOverlay::Danger => ctx.data.text("ui.overlay_danger"),
    }
}

fn overlay_icon(overlay: MapOverlay) -> style::IconKind {
    match overlay {
        MapOverlay::Realm => style::IconKind::Crown,
        MapOverlay::Supply => style::IconKind::Road,
        MapOverlay::Danger => style::IconKind::Danger,
    }
}

fn draw_map_backdrop(rect: Rect) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.020, 0.036, 0.032, 1.0),
    );
    draw_line(
        rect.x,
        rect.y,
        rect.right(),
        rect.y,
        1.0,
        Color::new(0.82, 0.66, 0.36, 0.22),
    );
    draw_line(
        rect.x,
        rect.bottom(),
        rect.right(),
        rect.bottom(),
        1.0,
        Color::new(0.82, 0.66, 0.36, 0.20),
    );
}

fn draw_map_overlay(ctx: &UiContext<'_>, view: &MapView) {
    match ctx.map_overlay {
        MapOverlay::Realm => {}
        MapOverlay::Supply => draw_supply_overlay(ctx, view),
        MapOverlay::Danger => draw_danger_overlay(ctx, view),
    }
}

fn draw_supply_overlay(ctx: &UiContext<'_>, view: &MapView) {
    for route in &ctx.session.routes {
        if !route.known {
            continue;
        }
        let (Some(from), Some(to)) = (ctx.data.site(&route.site_a), ctx.data.site(&route.site_b))
        else {
            continue;
        };
        let start = view.site_position(from);
        let end = view.site_position(to);
        let connected = ctx
            .session
            .is_site_in_capital_network(ctx.data, &route.site_a)
            && ctx
                .session
                .is_site_in_capital_network(ctx.data, &route.site_b);
        let color = if connected {
            Color::new(0.36, 0.72, 0.34, 0.24)
        } else if route.level.is_built() {
            Color::new(0.86, 0.58, 0.22, 0.23)
        } else {
            Color::new(0.55, 0.50, 0.38, 0.10)
        };
        draw_line(
            start.x,
            start.y,
            end.x,
            end.y,
            route_thickness(route) + 7.0,
            color,
        );
    }
}

fn draw_danger_overlay(ctx: &UiContext<'_>, view: &MapView) {
    for pressure in &ctx.session.wilderness_pressure {
        let Some(region) = ctx.data.region(&pressure.region_id) else {
            continue;
        };
        let position = view.tile_center(region.label_position.x, region.label_position.y);
        let radius = (44.0 + pressure.pressure as f32 * 0.9) * ctx.camera_zoom.clamp(0.85, 1.25);
        let alpha = (0.06 + pressure.pressure as f32 / 360.0).clamp(0.08, 0.30);
        draw_circle(
            position.x,
            position.y,
            radius,
            Color::new(0.70, 0.16, 0.12, alpha),
        );
    }

    for settlement in &ctx.session.settlements {
        let Some(site) = ctx.data.site(&settlement.location_id) else {
            continue;
        };
        let position = view.site_position(site);
        let radius = (14.0 + settlement.danger as f32 * 0.35) * ctx.camera_zoom.clamp(0.9, 1.25);
        draw_circle(
            position.x,
            position.y,
            radius,
            Color::new(0.90, 0.25, 0.12, 0.11),
        );
    }
}

fn draw_region_labels(ctx: &UiContext<'_>, view: &MapView) {
    for region in &ctx.data.regions {
        let position = view.tile_center(region.label_position.x, region.label_position.y)
            + vec2(0.0, -view.tile_size() * 2.0);
        if !view.is_visible(position, 120.0) {
            continue;
        }
        let tint = color_from_array(region.tint);
        draw_circle(
            position.x,
            position.y,
            view.tile_size() * 2.5,
            Color::new(tint.r, tint.g, tint.b, 0.11),
        );
        draw_line(
            position.x - 44.0,
            position.y + 9.0,
            position.x + 44.0,
            position.y + 9.0,
            1.0,
            Color::new(tint.r, tint.g, tint.b, 0.38),
        );
        draw_text_centered(
            &region.name,
            position.x + 1.4,
            position.y + 1.4,
            TextStyle::new(15.0, Color::new(0.02, 0.018, 0.012, 0.70)),
        );
        draw_text_centered(
            &region.name,
            position.x,
            position.y,
            TextStyle::new(15.0, Color::new(0.92, 0.87, 0.70, 0.86)),
        );
    }
}

fn draw_map_vignette(rect: Rect) {
    for i in 0..8 {
        let alpha = 0.045 + i as f32 * 0.018;
        let inset = i as f32 * 5.0;
        draw_rectangle_lines(
            rect.x + inset,
            rect.y + inset,
            (rect.w - inset * 2.0).max(0.0),
            (rect.h - inset * 2.0).max(0.0),
            5.0,
            Color::new(0.0, 0.0, 0.0, alpha),
        );
    }
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        1.0,
        Color::new(0.74, 0.58, 0.32, 0.20),
    );
}

fn draw_roads(ctx: &UiContext<'_>, view: &MapView) {
    for route in &ctx.session.routes {
        if !route.known {
            continue;
        }
        let (Some(from), Some(to)) = (ctx.data.site(&route.site_a), ctx.data.site(&route.site_b))
        else {
            continue;
        };
        let start = view.site_position(from);
        let end = view.site_position(to);
        let thickness = route_thickness(route) * view.scale().clamp(0.9, 1.45);
        let bend_code = ((from.position.x + to.position.y) % 3 - 1) as f32;
        draw_road_curve(
            start,
            end,
            bend_code * view.tile_size() * 0.42,
            thickness,
            route_color(ctx, route),
        );
        if route.condition != RouteCondition::Clear {
            draw_route_condition_marker(start, end, route.condition);
        }
    }
}

fn draw_road_curve(start: Vec2, end: Vec2, bend: f32, thickness: f32, color: Color) {
    let delta = end - start;
    let normal = if delta.length_squared() > 0.01 {
        vec2(-delta.y, delta.x).normalize()
    } else {
        Vec2::ZERO
    };
    let control = (start + end) * 0.5 + normal * bend;
    let segments = 8;
    for segment in 0..segments {
        let t0 = segment as f32 / segments as f32;
        let t1 = (segment + 1) as f32 / segments as f32;
        let from = quadratic_point(start, control, end, t0);
        let to = quadratic_point(start, control, end, t1);
        draw_line(
            from.x,
            from.y,
            to.x,
            to.y,
            thickness + 2.6,
            Color::new(0.035, 0.025, 0.014, 0.68),
        );
        draw_line(from.x, from.y, to.x, to.y, thickness, color);
    }
}

fn quadratic_point(start: Vec2, control: Vec2, end: Vec2, t: f32) -> Vec2 {
    let inverse = 1.0 - t;
    start * inverse * inverse + control * 2.0 * inverse * t + end * t * t
}

fn route_color(ctx: &UiContext<'_>, route: &RouteRuntimeState) -> Color {
    if route.condition == RouteCondition::Blocked {
        return Color::new(0.66, 0.18, 0.12, 0.90);
    }
    if route.condition == RouteCondition::Damaged {
        return Color::new(0.86, 0.48, 0.18, 0.90);
    }
    if route.level == RouteLevel::None {
        return Color::new(0.50, 0.46, 0.36, 0.26);
    }

    let in_network = ctx
        .session
        .is_site_in_capital_network(ctx.data, &route.site_a)
        && ctx
            .session
            .is_site_in_capital_network(ctx.data, &route.site_b);
    match (route.level, in_network) {
        (RouteLevel::Path, true) => Color::new(0.74, 0.66, 0.42, 0.86),
        (RouteLevel::Path, false) => Color::new(0.54, 0.49, 0.36, 0.72),
        (RouteLevel::Road, true) => Color::new(0.86, 0.70, 0.38, 0.95),
        (RouteLevel::Road, false) => Color::new(0.62, 0.54, 0.38, 0.82),
        (RouteLevel::StoneRoad, true) => Color::new(0.88, 0.84, 0.70, 1.0),
        (RouteLevel::StoneRoad, false) => Color::new(0.68, 0.64, 0.56, 0.90),
        (RouteLevel::None, _) => Color::new(0.50, 0.46, 0.36, 0.26),
    }
}

fn route_thickness(route: &RouteRuntimeState) -> f32 {
    match route.level {
        RouteLevel::None => 1.0,
        RouteLevel::Path => 2.0,
        RouteLevel::Road => 3.2,
        RouteLevel::StoneRoad => 4.5,
    }
}

fn draw_route_condition_marker(start: Vec2, end: Vec2, condition: RouteCondition) {
    let mid = (start + end) * 0.5;
    let color = match condition {
        RouteCondition::Clear => return,
        RouteCondition::Damaged => Color::new(0.95, 0.60, 0.22, 1.0),
        RouteCondition::Blocked => Color::new(0.90, 0.20, 0.14, 1.0),
    };
    draw_circle(mid.x, mid.y, 5.0, color);
    draw_circle_lines(mid.x, mid.y, 7.0, 1.5, dark::BACKGROUND);
}

#[derive(Debug, Clone, Copy)]
pub(super) struct MapView {
    pub(super) rect: Rect,
    origin: Vec2,
    tile_size: f32,
    axis_x: Vec2,
    axis_y: Vec2,
}

impl MapView {
    fn new(ctx: &UiContext<'_>, rect: Rect) -> Self {
        let tile_size = (rect.h / 39.0 * ctx.camera_zoom).clamp(11.0, 25.0);
        let axis_x = vec2(tile_size * 0.88, tile_size * 0.17);
        let axis_y = vec2(-tile_size * 0.28, tile_size * 0.86);
        let map_center = axis_x * (ctx.data.terrain.width as f32 * 0.5)
            + axis_y * (ctx.data.terrain.height as f32 * 0.5);
        let pan = -ctx.camera_target * ctx.camera_zoom * 0.92 + vec2(0.0, tile_size * 1.4);
        let origin = rect.center() - map_center + pan;
        Self {
            rect,
            origin,
            tile_size,
            axis_x,
            axis_y,
        }
    }

    pub(super) fn tile_quad(self, x: i32, y: i32) -> [Vec2; 4] {
        let center = self.tile_center(x, y);
        let half_x = self.axis_x * 0.5;
        let half_y = self.axis_y * 0.5;
        [
            center - half_x - half_y,
            center + half_x - half_y,
            center + half_x + half_y,
            center - half_x + half_y,
        ]
    }

    pub(super) fn tile_center(self, x: i32, y: i32) -> Vec2 {
        vec2(
            self.origin.x + (x as f32 + 0.5) * self.axis_x.x + (y as f32 + 0.5) * self.axis_y.x,
            self.origin.y + (x as f32 + 0.5) * self.axis_x.y + (y as f32 + 0.5) * self.axis_y.y,
        )
    }

    pub(super) fn site_position(self, site: &SiteDef) -> Vec2 {
        self.tile_center(site.position.x, site.position.y)
    }

    pub(super) fn is_visible(self, position: Vec2, margin: f32) -> bool {
        position.x >= self.rect.x - margin
            && position.x <= self.rect.right() + margin
            && position.y >= self.rect.y - margin
            && position.y <= self.rect.bottom() + margin
    }

    pub(super) fn scale(self) -> f32 {
        (self.tile_size / 16.0).clamp(0.82, 1.75)
    }

    pub(super) fn tile_size(self) -> f32 {
        self.tile_size
    }
}
