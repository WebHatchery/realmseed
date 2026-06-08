//! Strategic terrain map rendering and site picking.

use super::{color_from_array, map_panel_rect, UiAction, UiContext};
use crate::data::{RouteLevel, SettlementTier, SiteCategory, SiteDef};
use crate::state::{RouteCondition, RouteRuntimeState, SettlementRuntimeState, SettlementStatus};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::RectExt;

pub(super) fn draw_map_panel(
    ctx: &UiContext<'_>,
    mouse: Vec2,
    input_enabled: bool,
    actions: &mut Vec<UiAction>,
) {
    let rect = map_panel_rect();
    let style = SurfaceStyle::new(Color::new(0.055, 0.060, 0.050, 0.98))
        .with_border(1.0, Color::new(0.52, 0.47, 0.32, 0.70))
        .with_inner_border(4.0, 1.0, Color::new(1.0, 0.92, 0.72, 0.05))
        .with_header(42.0, Color::new(0.085, 0.090, 0.075, 1.0))
        .with_header_divider(1.0, Color::new(0.52, 0.47, 0.32, 0.35));
    draw_surface_with_title(
        rect,
        Some("Greenvale Charter Map"),
        &style,
        TextStyle::new(18.0, dark::TEXT),
    );

    let map_rect = Rect::new(rect.x + 24.0, rect.y + 62.0, rect.w - 48.0, rect.h - 88.0);
    let view = MapView::new(ctx, map_rect);
    draw_terrain(ctx, &view);
    draw_region_labels(ctx, &view);
    draw_roads(ctx, &view);
    draw_sites(ctx, &view);

    if input_enabled && is_mouse_button_released(MouseButton::Left) {
        if let Some(site_id) = picked_site_id(ctx, &view, mouse) {
            actions.push(UiAction::SelectSite(site_id));
        }
    }

    if map_rect.contains_point(mouse) {
        draw_tooltip(
            "Known sites use solid markers. Question markers are adjacent sites that can be scouted.",
            mouse,
        );
    }
}

fn draw_terrain(ctx: &UiContext<'_>, view: &MapView) {
    for y in 0..ctx.data.terrain.height {
        for x in 0..ctx.data.terrain.width {
            let Some(terrain) = ctx.data.terrain_at(x as i32, y as i32) else {
                continue;
            };
            let tile_rect = view.tile_rect(x as i32, y as i32);
            if !view.rect.overlaps(&tile_rect) {
                continue;
            }

            let color = color_from_array(terrain.color);
            draw_rectangle(
                tile_rect.x,
                tile_rect.y,
                tile_rect.w + 0.5,
                tile_rect.h + 0.5,
                color,
            );
        }
    }

    draw_rectangle_lines(
        view.rect.x,
        view.rect.y,
        view.rect.w,
        view.rect.h,
        1.0,
        Color::new(0.83, 0.73, 0.47, 0.25),
    );
}

fn draw_region_labels(ctx: &UiContext<'_>, view: &MapView) {
    for region in &ctx.data.regions {
        let position = view.tile_center(region.label_position.x, region.label_position.y);
        let tint = color_from_array(region.tint);
        draw_circle(position.x, position.y, 42.0 * ctx.camera_zoom, tint);
        draw_text_centered(
            &region.name,
            position.x,
            position.y,
            TextStyle::new(13.0, Color::new(0.90, 0.86, 0.70, 0.58)),
        );
    }
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

fn draw_sites(ctx: &UiContext<'_>, view: &MapView) {
    for site in &ctx.data.sites {
        if ctx.session.is_known(&site.id) {
            draw_known_site(ctx, view, site);
        } else if ctx.session.is_adjacent_unknown(ctx.data, &site.id) {
            draw_unknown_site(ctx, view, site);
        }
    }
}

fn draw_known_site(ctx: &UiContext<'_>, view: &MapView, site: &SiteDef) {
    let position = view.site_position(site);
    let selected = ctx.session.selected_site_id == site.id;
    let settlement = ctx.session.settlement_at_site(&site.id);
    let radius = marker_radius(site, settlement) * ctx.camera_zoom.clamp(0.9, 1.3);
    let fill = marker_color(site, settlement);

    draw_circle(
        position.x,
        position.y,
        radius + 3.0,
        Color::new(0.03, 0.025, 0.015, 0.75),
    );
    draw_circle(position.x, position.y, radius, fill);
    draw_circle_lines(
        position.x,
        position.y,
        radius,
        1.5,
        Color::new(0.94, 0.86, 0.60, 0.80),
    );

    if selected {
        draw_circle_lines(position.x, position.y, radius + 7.0, 3.0, dark::ACCENT);
    }

    if selected || ctx.camera_zoom > 1.1 {
        draw_text_ex(
            &site.name,
            position.x + radius + 5.0,
            position.y - radius - 3.0,
            TextStyle::new(13.0, dark::TEXT_BRIGHT).params(),
        );
    }
}

fn draw_unknown_site(ctx: &UiContext<'_>, view: &MapView, site: &SiteDef) {
    let position = view.site_position(site);
    let selected = ctx.session.selected_site_id == site.id;
    let radius = 7.0 * ctx.camera_zoom.clamp(0.9, 1.25);
    let diamond = [
        vec2(position.x, position.y - radius),
        vec2(position.x + radius, position.y),
        vec2(position.x, position.y + radius),
        vec2(position.x - radius, position.y),
    ];

    draw_triangle(
        diamond[0],
        diamond[1],
        diamond[2],
        Color::new(0.72, 0.66, 0.48, 0.85),
    );
    draw_triangle(
        diamond[0],
        diamond[2],
        diamond[3],
        Color::new(0.72, 0.66, 0.48, 0.85),
    );
    draw_text_centered(
        "?",
        position.x,
        position.y + 5.0,
        TextStyle::new(16.0, dark::BACKGROUND),
    );

    if selected {
        draw_circle_lines(position.x, position.y, radius + 7.0, 3.0, dark::ACCENT);
    }
}

fn picked_site_id(ctx: &UiContext<'_>, view: &MapView, mouse: Vec2) -> Option<String> {
    if !view.rect.contains_point(mouse) {
        return None;
    }

    ctx.data.sites.iter().rev().find_map(|site| {
        if !ctx.session.can_select_site(ctx.data, &site.id) {
            return None;
        }
        let position = view.site_position(site);
        let radius = marker_radius(site, ctx.session.settlement_at_site(&site.id)).max(8.0) + 7.0;
        (position.distance(mouse) <= radius).then(|| site.id.clone())
    })
}

fn marker_radius(site: &SiteDef, settlement: Option<&SettlementRuntimeState>) -> f32 {
    if let Some(settlement) = settlement {
        return match settlement.status {
            SettlementStatus::Lost => 6.0,
            SettlementStatus::Active => match settlement.tier {
                SettlementTier::Camp => 7.0,
                SettlementTier::Village => 8.5,
                SettlementTier::Town => 10.0,
                SettlementTier::City => 12.0,
            },
        };
    }

    match site.site_type.as_str() {
        "capital" => 9.0,
        "independent_settlement" => 7.5,
        _ => match site.category {
            SiteCategory::Settlement => 6.5,
            SiteCategory::Independent => 7.5,
            SiteCategory::Landmark => 5.8,
        },
    }
}

fn marker_color(site: &SiteDef, settlement: Option<&SettlementRuntimeState>) -> Color {
    if let Some(settlement) = settlement {
        return match settlement.status {
            SettlementStatus::Lost => Color::new(0.22, 0.18, 0.16, 1.0),
            SettlementStatus::Active => match settlement.tier {
                SettlementTier::Camp => Color::new(0.66, 0.76, 0.42, 1.0),
                SettlementTier::Village => Color::new(0.78, 0.70, 0.36, 1.0),
                SettlementTier::Town => Color::new(0.82, 0.55, 0.30, 1.0),
                SettlementTier::City => Color::new(0.88, 0.42, 0.28, 1.0),
            },
        };
    }

    match site.site_type.as_str() {
        "capital" => Color::new(0.86, 0.67, 0.26, 1.0),
        "independent_settlement" => Color::new(0.36, 0.58, 0.76, 1.0),
        _ => match site.category {
            SiteCategory::Settlement => Color::new(0.55, 0.76, 0.42, 1.0),
            SiteCategory::Independent => Color::new(0.36, 0.58, 0.76, 1.0),
            SiteCategory::Landmark => Color::new(0.78, 0.60, 0.34, 1.0),
        },
    }
}

#[derive(Debug, Clone, Copy)]
struct MapView {
    rect: Rect,
    origin: Vec2,
    tile_size: f32,
}

impl MapView {
    fn new(ctx: &UiContext<'_>, rect: Rect) -> Self {
        let base_tile_size =
            (rect.w / ctx.data.terrain.width as f32).min(rect.h / ctx.data.terrain.height as f32);
        let tile_size = (base_tile_size * ctx.camera_zoom).clamp(7.0, 20.0);
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

    fn tile_rect(self, x: i32, y: i32) -> Rect {
        Rect::new(
            self.origin.x + x as f32 * self.tile_size,
            self.origin.y + y as f32 * self.tile_size,
            self.tile_size,
            self.tile_size,
        )
    }

    fn tile_center(self, x: i32, y: i32) -> Vec2 {
        vec2(
            self.origin.x + (x as f32 + 0.5) * self.tile_size,
            self.origin.y + (y as f32 + 0.5) * self.tile_size,
        )
    }

    fn site_position(self, site: &SiteDef) -> Vec2 {
        self.tile_center(site.position.x, site.position.y)
    }
}
