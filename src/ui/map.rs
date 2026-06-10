//! Strategic terrain map rendering and site picking.

use super::{
    color_from_array, map_panel_rect, map_sites, style, virtual_icon_button, MapOverlay, UiAction,
    UiContext,
};
use crate::data::{RouteLevel, SiteDef};
use crate::state::{RouteCondition, RouteRuntimeState};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::RectExt;

pub(super) fn draw_map_panel(
    ctx: &UiContext<'_>,
    mouse: Vec2,
    input_enabled: bool,
    actions: &mut Vec<UiAction>,
) {
    let rect = map_panel_rect(ctx);
    draw_map_backdrop(rect);

    let map_rect = rect.inset(0.0);
    let view = MapView::new(ctx, map_rect);
    draw_terrain(ctx, &view);
    draw_map_overlay(ctx, &view);
    draw_region_labels(ctx, &view);
    draw_roads(ctx, &view);
    map_sites::draw_sites(ctx, &view);
    draw_map_vignette(map_rect);
    draw_overlay_tabs(ctx, rect, mouse, input_enabled, actions);

    if input_enabled && is_mouse_button_released(MouseButton::Left) {
        if let Some(site_id) = map_sites::picked_site_id(ctx, &view, mouse) {
            actions.push(UiAction::SelectSite(site_id));
        }
    }

    let mode_hint = match ctx.map_overlay {
        MapOverlay::Realm => {
            "Realm view: banners are yours, red marks are rivals, diamonds are independents."
        }
        MapOverlay::Supply => {
            "Supply view: green rings are connected to the capital, amber rings are isolated."
        }
        MapOverlay::Danger => "Danger view: red washes show wilderness and settlement pressure.",
    };
    style::draw_hover_tooltip("map_overlay_hint", map_rect, mode_hint, mouse);
}

fn draw_overlay_tabs(
    ctx: &UiContext<'_>,
    panel_rect: Rect,
    mouse: Vec2,
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
            overlay.label(),
            overlay_icon(*overlay),
            input_enabled,
            tone,
            mouse,
        ) {
            actions.push(UiAction::SetMapOverlay(*overlay));
        }
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

fn draw_terrain(ctx: &UiContext<'_>, view: &MapView) {
    draw_rectangle(
        view.rect.x,
        view.rect.y,
        view.rect.w,
        view.rect.h,
        Color::new(0.035, 0.056, 0.045, 1.0),
    );
    for y in 0..ctx.data.terrain.height {
        for x in 0..ctx.data.terrain.width {
            let Some(terrain) = ctx.data.terrain_at(x as i32, y as i32) else {
                continue;
            };
            let tile_rect = view.tile_rect(x as i32, y as i32);
            if !view.rect.overlaps(&tile_rect) {
                continue;
            }

            let color =
                textured_terrain_color(color_from_array(terrain.color), x, y, tile_rect, view);
            draw_rectangle(
                tile_rect.x,
                tile_rect.y,
                tile_rect.w + 0.5,
                tile_rect.h + 0.5,
                color,
            );
            draw_terrain_detail(terrain.id.as_str(), tile_rect, x, y);
        }
    }

    draw_river_glow(ctx, view);
}

fn textured_terrain_color(base: Color, x: usize, y: usize, rect: Rect, view: &MapView) -> Color {
    let hash = ((x as u32).wrapping_mul(37) ^ (y as u32).wrapping_mul(53)) % 17;
    let variation = 0.84 + hash as f32 * 0.014;
    let center = vec2(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
    let dx = ((center.x - view.rect.x) / view.rect.w - 0.5).abs();
    let dy = ((center.y - view.rect.y) / view.rect.h - 0.5).abs();
    let vignette = 1.0 - (dx.max(dy) * 0.72).clamp(0.0, 0.42);
    Color::new(
        (base.r * variation * vignette).clamp(0.0, 1.0),
        (base.g * variation * vignette).clamp(0.0, 1.0),
        (base.b * variation * vignette).clamp(0.0, 1.0),
        base.a,
    )
}

fn draw_terrain_detail(terrain_id: &str, rect: Rect, x: usize, y: usize) {
    if rect.w < 7.0 {
        return;
    }
    let hash = ((x as u32 * 11 + y as u32 * 23) % 5) as f32;
    let center = vec2(
        rect.x + rect.w * (0.36 + hash * 0.04),
        rect.y + rect.h * 0.55,
    );
    match terrain_id {
        "forest" if (x + y) % 3 == 0 => {
            draw_circle(
                center.x,
                center.y,
                rect.w * 0.22,
                Color::new(0.02, 0.10, 0.05, 0.20),
            );
            draw_circle(
                center.x + rect.w * 0.16,
                center.y + rect.h * 0.10,
                rect.w * 0.18,
                Color::new(0.04, 0.17, 0.08, 0.24),
            );
        }
        "hills" if (x + y) % 4 == 0 => {
            draw_line(
                rect.x + 2.0,
                rect.y + rect.h - 3.0,
                rect.x + rect.w * 0.5,
                rect.y + 3.0,
                1.0,
                Color::new(0.22, 0.20, 0.13, 0.24),
            );
            draw_line(
                rect.x + rect.w * 0.5,
                rect.y + 3.0,
                rect.x + rect.w - 2.0,
                rect.y + rect.h - 3.0,
                1.0,
                Color::new(0.22, 0.20, 0.13, 0.24),
            );
        }
        "mountain" if (x + y) % 2 == 0 => {
            draw_triangle(
                vec2(rect.x + rect.w * 0.5, rect.y + 2.0),
                vec2(rect.x + 2.0, rect.y + rect.h - 2.0),
                vec2(rect.x + rect.w - 2.0, rect.y + rect.h - 2.0),
                Color::new(0.62, 0.62, 0.58, 0.15),
            );
        }
        "river" => {
            draw_line(
                rect.x + 1.0,
                rect.y + rect.h * 0.48,
                rect.x + rect.w - 1.0,
                rect.y + rect.h * 0.32,
                1.5,
                Color::new(0.58, 0.88, 0.95, 0.32),
            );
        }
        "coast" if y % 3 == 0 => {
            draw_line(
                rect.x + 1.0,
                rect.y + rect.h * 0.65,
                rect.x + rect.w - 1.0,
                rect.y + rect.h * 0.55,
                1.0,
                Color::new(0.76, 0.82, 0.70, 0.16),
            );
        }
        "marsh" if (x + y) % 3 == 1 => {
            draw_circle(center.x, center.y, 1.2, Color::new(0.58, 0.68, 0.40, 0.18));
        }
        _ => {}
    }
}

fn draw_river_glow(ctx: &UiContext<'_>, view: &MapView) {
    for y in 0..ctx.data.terrain.height {
        for x in 0..ctx.data.terrain.width {
            let Some(terrain) = ctx.data.terrain_at(x as i32, y as i32) else {
                continue;
            };
            if terrain.id != "river" && terrain.id != "coast" {
                continue;
            }
            let rect = view.tile_rect(x as i32, y as i32);
            draw_rectangle(
                rect.x,
                rect.y,
                rect.w + 0.8,
                rect.h + 0.8,
                if terrain.id == "river" {
                    Color::new(0.16, 0.48, 0.58, 0.18)
                } else {
                    Color::new(0.10, 0.31, 0.42, 0.16)
                },
            );
        }
    }
}

#[allow(dead_code)]
fn draw_map_grid(ctx: &UiContext<'_>, view: &MapView) {
    let grid_color = Color::new(0.95, 0.84, 0.56, 0.055);
    for x in (0..=ctx.data.terrain.width).step_by(5) {
        let px = view.origin.x + x as f32 * view.tile_size;
        draw_line(
            px,
            view.rect.y,
            px,
            view.rect.y + view.rect.h,
            1.0,
            grid_color,
        );
    }
    for y in (0..=ctx.data.terrain.height).step_by(5) {
        let py = view.origin.y + y as f32 * view.tile_size;
        draw_line(
            view.rect.x,
            py,
            view.rect.x + view.rect.w,
            py,
            1.0,
            grid_color,
        );
    }
    draw_rectangle_lines(
        view.rect.x + 2.0,
        view.rect.y + 2.0,
        view.rect.w - 4.0,
        view.rect.h - 4.0,
        1.0,
        Color::new(0.03, 0.025, 0.016, 0.45),
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
        let position = view.tile_center(region.label_position.x, region.label_position.y);
        let tint = color_from_array(region.tint);
        draw_circle(
            position.x,
            position.y,
            46.0 * ctx.camera_zoom,
            Color::new(tint.r, tint.g, tint.b, 0.11),
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
        draw_line(
            start.x,
            start.y,
            end.x,
            end.y,
            route_thickness(route) + 2.2,
            Color::new(0.035, 0.030, 0.020, 0.58),
        );
        draw_line(
            start.x,
            start.y,
            end.x,
            end.y,
            route_thickness(route),
            route_color(ctx, route),
        );
        if route.condition != RouteCondition::Clear {
            draw_route_condition_marker(start, end, route.condition);
        }
    }
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
}

impl MapView {
    fn new(ctx: &UiContext<'_>, rect: Rect) -> Self {
        let base_tile_size =
            (rect.w / ctx.data.terrain.width as f32).min(rect.h / ctx.data.terrain.height as f32);
        let tile_size = (base_tile_size * ctx.camera_zoom).clamp(7.0, 32.0);
        let map_size = vec2(
            ctx.data.terrain.width as f32 * tile_size,
            ctx.data.terrain.height as f32 * tile_size,
        );
        let origin = vec2(
            rect.x + (rect.w - map_size.x) * 0.5 - ctx.camera_target.x * 0.12,
            rect.y + (rect.h - map_size.y) * 0.5 - ctx.camera_target.y * 0.12,
        );
        Self {
            rect,
            origin,
            tile_size,
        }
    }

    pub(super) fn tile_rect(self, x: i32, y: i32) -> Rect {
        Rect::new(
            self.origin.x + x as f32 * self.tile_size,
            self.origin.y + y as f32 * self.tile_size,
            self.tile_size,
            self.tile_size,
        )
    }

    pub(super) fn tile_center(self, x: i32, y: i32) -> Vec2 {
        vec2(
            self.origin.x + (x as f32 + 0.5) * self.tile_size,
            self.origin.y + (y as f32 + 0.5) * self.tile_size,
        )
    }

    pub(super) fn site_position(self, site: &SiteDef) -> Vec2 {
        self.tile_center(site.position.x, site.position.y)
    }
}
