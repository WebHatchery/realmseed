//! Site markers, map labels, and picking for the strategic map.

use super::map::MapView;
use super::{map_site_sprites, style, MapOverlay, UiContext};
use crate::data::{SettlementTier, SiteCategory, SiteDef};
use crate::state::{SettlementRuntimeState, SettlementStatus};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;
use macroquad_toolkit::ui::RectExt;
use std::cmp::Ordering;

pub(super) fn draw_sites(ctx: &UiContext<'_>, view: &MapView) {
    let mut sites: Vec<&SiteDef> = ctx
        .data
        .sites
        .iter()
        .filter(|site| {
            ctx.session.is_known(&site.id)
                || ctx.session.is_adjacent_unknown(ctx.data, &site.id)
        })
        .collect();
    sites.sort_by(|left, right| {
        view.site_position(left)
            .y
            .partial_cmp(&view.site_position(right).y)
            .unwrap_or(Ordering::Equal)
    });

    for site in sites {
        if ctx.session.is_known(&site.id) {
            draw_known_site(ctx, view, site);
        } else if !ctx.sprite_showcase && ctx.session.is_adjacent_unknown(ctx.data, &site.id) {
            draw_unknown_site(ctx, view, site);
        }
    }
}

pub(super) fn picked_site_id(ctx: &UiContext<'_>, view: &MapView, mouse: Vec2) -> Option<String> {
    if !view.rect.contains_point(mouse) {
        return None;
    }

    ctx.data.sites.iter().rev().find_map(|site| {
        if !ctx.session.can_select_site(ctx.data, &site.id) {
            return None;
        }
        let position = view.site_position(site);
        let settlement = ctx.session.settlement_at_site(&site.id);
        let kind = marker_kind(ctx, site, settlement);
        let radius = map_site_sprites::hit_radius(
            &ctx.sprites,
            site,
            kind,
            view.scale(),
            marker_radius(site, settlement),
        );
        (position.distance(mouse) <= radius).then(|| site.id.clone())
    })
}

fn draw_known_site(ctx: &UiContext<'_>, view: &MapView, site: &SiteDef) {
    let position = view.site_position(site);
    let scale = view.scale();
    if !view.is_visible(position, 80.0 * scale) {
        return;
    }
    let selected = ctx.session.selected_site_id == site.id;
    let settlement = ctx.session.settlement_at_site(&site.id);
    let radius = marker_radius(site, settlement) * scale;
    let fill = marker_color(ctx, site, settlement);
    let kind = marker_kind(ctx, site, settlement);

    draw_landmark_silhouette(site, kind, position, scale, fill);
    map_site_sprites::draw_site_sprite(ctx, site, kind, position, scale);
    draw_circle(
        position.x,
        position.y,
        radius + 3.0,
        Color::new(0.03, 0.025, 0.015, 0.82),
    );
    if selected {
        draw_circle(
            position.x,
            position.y,
            radius + 17.0,
            Color::new(0.92, 0.68, 0.30, 0.18),
        );
        draw_circle_lines(
            position.x,
            position.y,
            radius + 13.0,
            1.4,
            Color::new(0.96, 0.78, 0.36, 0.62),
        );
    }
    if ctx.map_overlay == MapOverlay::Supply {
        draw_supply_ring(ctx, site, position, radius);
    }
    draw_marker_shape(kind, position, radius, fill);
    draw_marker_symbol(kind, position, radius);

    if selected {
        draw_circle_lines(position.x, position.y, radius + 8.0, 2.0, style::CYAN);
    }

    if selected
        || ctx.camera_zoom > 1.1
        || matches!(
            kind,
            MarkerKind::Capital
                | MarkerKind::PlayerSettlement
                | MarkerKind::Rival
                | MarkerKind::Independent
        )
    {
        draw_site_label(site, position, radius, selected);
    }
}

fn draw_unknown_site(ctx: &UiContext<'_>, view: &MapView, site: &SiteDef) {
    let position = view.site_position(site);
    let scale = view.scale();
    if !view.is_visible(position, 48.0 * scale) {
        return;
    }
    let selected = ctx.session.selected_site_id == site.id;
    let radius = 7.0 * scale;
    draw_circle(
        position.x,
        position.y,
        radius + 5.0,
        Color::new(0.02, 0.02, 0.014, 0.70),
    );
    draw_circle(
        position.x,
        position.y,
        radius + 2.0,
        Color::new(0.12, 0.10, 0.06, 0.92),
    );
    draw_circle_lines(position.x, position.y, radius + 2.0, 1.4, style::GOLD);
    let pulse = (get_time() as f32 * 2.2).sin() * 1.5 + 3.0;
    draw_circle_lines(
        position.x,
        position.y,
        radius + pulse,
        1.4,
        Color::new(0.94, 0.82, 0.52, 0.42),
    );
    draw_text_centered(
        "?",
        position.x,
        position.y + 5.0,
        TextStyle::new(17.0, style::TEXT_BRIGHT),
    );

    if selected {
        draw_circle_lines(position.x, position.y, radius + 8.0, 2.0, style::CYAN);
        draw_ui_text_ex(
            "Scout",
            position.x + radius + 5.0,
            position.y - radius - 2.0,
            TextStyle::new(12.0, style::TEXT_BRIGHT).params(),
        );
    }
}

fn draw_landmark_silhouette(
    site: &SiteDef,
    kind: MarkerKind,
    position: Vec2,
    scale: f32,
    fill: Color,
) {
    let ground = position + vec2(0.0, 5.0 * scale);
    draw_ellipse(
        ground.x,
        ground.y + 5.0 * scale,
        23.0 * scale,
        7.0 * scale,
        0.0,
        Color::new(0.015, 0.018, 0.014, 0.42),
    );

    match kind {
        MarkerKind::Capital => draw_capital(ground, scale, fill),
        MarkerKind::PlayerSettlement => draw_settlement_cluster(ground, scale, fill),
        MarkerKind::Rival => draw_rival_hold(ground, scale, fill),
        MarkerKind::Independent => draw_independent_market(ground, scale, fill),
        MarkerKind::Landmark => draw_landmark(site, ground, scale, fill),
        MarkerKind::Settlement => draw_settlement_cluster(ground, scale, fill),
        MarkerKind::Lost => draw_ruined_marker(ground, scale, fill),
    }
}

fn draw_capital(ground: Vec2, scale: f32, fill: Color) {
    let wall = Color::new(fill.r * 0.78, fill.g * 0.72, fill.b * 0.52, 1.0);
    draw_rectangle(
        ground.x - 22.0 * scale,
        ground.y - 15.0 * scale,
        44.0 * scale,
        18.0 * scale,
        wall,
    );
    draw_rectangle(
        ground.x - 8.0 * scale,
        ground.y - 36.0 * scale,
        16.0 * scale,
        39.0 * scale,
        fill,
    );
    for offset in [-18.0, 18.0] {
        draw_rectangle(
            ground.x + (offset - 4.0) * scale,
            ground.y - 28.0 * scale,
            8.0 * scale,
            31.0 * scale,
            wall,
        );
        draw_triangle(
            vec2(ground.x + offset * scale, ground.y - 42.0 * scale),
            vec2(ground.x + (offset - 7.0) * scale, ground.y - 27.0 * scale),
            vec2(ground.x + (offset + 7.0) * scale, ground.y - 27.0 * scale),
            Color::new(0.35, 0.18, 0.10, 1.0),
        );
    }
    draw_triangle(
        vec2(ground.x, ground.y - 50.0 * scale),
        vec2(ground.x - 12.0 * scale, ground.y - 34.0 * scale),
        vec2(ground.x + 12.0 * scale, ground.y - 34.0 * scale),
        Color::new(0.56, 0.25, 0.12, 1.0),
    );
    draw_rectangle(
        ground.x - 3.0 * scale,
        ground.y - 14.0 * scale,
        6.0 * scale,
        12.0 * scale,
        Color::new(0.10, 0.12, 0.10, 1.0),
    );
    draw_line(
        ground.x,
        ground.y - 49.0 * scale,
        ground.x,
        ground.y - 63.0 * scale,
        1.0 * scale,
        style::GOLD,
    );
    draw_triangle(
        vec2(ground.x, ground.y - 63.0 * scale),
        vec2(ground.x + 10.0 * scale, ground.y - 59.0 * scale),
        vec2(ground.x, ground.y - 55.0 * scale),
        style::GOLD,
    );
}

fn draw_settlement_cluster(ground: Vec2, scale: f32, fill: Color) {
    let roof = Color::new(fill.r * 0.65, fill.g * 0.42, fill.b * 0.22, 1.0);
    for (offset, height) in [(-15.0, 16.0), (0.0, 23.0), (15.0, 13.0)] {
        draw_rectangle(
            ground.x + (offset - 6.0) * scale,
            ground.y - height * scale,
            12.0 * scale,
            height * scale,
            fill,
        );
        draw_triangle(
            vec2(
                ground.x + offset * scale,
                ground.y - (height + 10.0) * scale,
            ),
            vec2(
                ground.x + (offset - 9.0) * scale,
                ground.y - (height - 1.0) * scale,
            ),
            vec2(
                ground.x + (offset + 9.0) * scale,
                ground.y - (height - 1.0) * scale,
            ),
            roof,
        );
        draw_rectangle(
            ground.x + (offset - 2.0) * scale,
            ground.y - (height - 8.0) * scale,
            4.0 * scale,
            7.0 * scale,
            Color::new(0.14, 0.11, 0.07, 0.95),
        );
    }
    draw_line(
        ground.x - 25.0 * scale,
        ground.y + 1.0 * scale,
        ground.x + 24.0 * scale,
        ground.y + 1.0 * scale,
        2.0 * scale,
        Color::new(0.78, 0.66, 0.38, 0.52),
    );
}

fn draw_rival_hold(ground: Vec2, scale: f32, fill: Color) {
    let dark = Color::new(fill.r * 0.62, fill.g * 0.48, fill.b * 0.48, 1.0);
    draw_rectangle(
        ground.x - 17.0 * scale,
        ground.y - 27.0 * scale,
        34.0 * scale,
        29.0 * scale,
        dark,
    );
    draw_triangle(
        vec2(ground.x, ground.y - 42.0 * scale),
        vec2(ground.x - 20.0 * scale, ground.y - 25.0 * scale),
        vec2(ground.x + 20.0 * scale, ground.y - 25.0 * scale),
        fill,
    );
    draw_line(
        ground.x,
        ground.y - 42.0 * scale,
        ground.x,
        ground.y - 59.0 * scale,
        1.0 * scale,
        Color::new(0.96, 0.44, 0.28, 0.95),
    );
    draw_triangle(
        vec2(ground.x, ground.y - 59.0 * scale),
        vec2(ground.x + 10.0 * scale, ground.y - 55.0 * scale),
        vec2(ground.x, ground.y - 51.0 * scale),
        Color::new(0.78, 0.20, 0.16, 1.0),
    );
}

fn draw_independent_market(ground: Vec2, scale: f32, fill: Color) {
    draw_rectangle(
        ground.x - 22.0 * scale,
        ground.y - 16.0 * scale,
        44.0 * scale,
        15.0 * scale,
        fill,
    );
    draw_triangle(
        vec2(ground.x - 25.0 * scale, ground.y - 16.0 * scale),
        vec2(ground.x, ground.y - 34.0 * scale),
        vec2(ground.x + 25.0 * scale, ground.y - 16.0 * scale),
        Color::new(0.18, 0.30, 0.38, 1.0),
    );
    draw_rectangle(
        ground.x - 4.0 * scale,
        ground.y - 31.0 * scale,
        8.0 * scale,
        30.0 * scale,
        Color::new(0.40, 0.52, 0.58, 0.95),
    );
    draw_line(
        ground.x,
        ground.y - 40.0 * scale,
        ground.x,
        ground.y - 52.0 * scale,
        1.0 * scale,
        Color::new(0.66, 0.83, 0.86, 0.92),
    );
}

fn draw_landmark(site: &SiteDef, ground: Vec2, scale: f32, fill: Color) {
    match site.site_type.as_str() {
        "ruin" => draw_ruin(ground, scale, fill),
        "resource" => draw_resource_node(ground, scale, fill),
        "pass" => draw_pass_gate(ground, scale, fill),
        "ford" => draw_ford(ground, scale, fill),
        "old_road" => draw_milestone(ground, scale, fill),
        "hazard" => draw_hazard(ground, scale, fill),
        _ => draw_milestone(ground, scale, fill),
    }
}

fn draw_ruin(ground: Vec2, scale: f32, fill: Color) {
    draw_rectangle(
        ground.x - 21.0 * scale,
        ground.y - 17.0 * scale,
        42.0 * scale,
        17.0 * scale,
        Color::new(0.24, 0.22, 0.18, 1.0),
    );
    draw_rectangle(
        ground.x - 15.0 * scale,
        ground.y - 35.0 * scale,
        8.0 * scale,
        35.0 * scale,
        fill,
    );
    draw_rectangle(
        ground.x + 7.0 * scale,
        ground.y - 27.0 * scale,
        8.0 * scale,
        27.0 * scale,
        fill,
    );
    draw_triangle(
        vec2(ground.x - 11.0 * scale, ground.y - 43.0 * scale),
        vec2(ground.x - 22.0 * scale, ground.y - 31.0 * scale),
        vec2(ground.x - 2.0 * scale, ground.y - 31.0 * scale),
        Color::new(0.46, 0.38, 0.24, 1.0),
    );
}

fn draw_resource_node(ground: Vec2, scale: f32, fill: Color) {
    for (x, y, size) in [(-15.0, -9.0, 13.0), (0.0, -19.0, 19.0), (16.0, -8.0, 11.0)] {
        draw_triangle(
            vec2(ground.x + x * scale, ground.y + (y - size) * scale),
            vec2(ground.x + (x - size * 0.70) * scale, ground.y + y * scale),
            vec2(ground.x + (x + size * 0.70) * scale, ground.y + y * scale),
            fill,
        );
    }
    draw_line(
        ground.x - 24.0 * scale,
        ground.y,
        ground.x + 24.0 * scale,
        ground.y,
        2.0 * scale,
        Color::new(0.92, 0.74, 0.30, 0.62),
    );
}

fn draw_pass_gate(ground: Vec2, scale: f32, fill: Color) {
    draw_triangle(
        vec2(ground.x, ground.y - 51.0 * scale),
        vec2(ground.x - 30.0 * scale, ground.y),
        vec2(ground.x + 30.0 * scale, ground.y),
        Color::new(0.24, 0.25, 0.27, 1.0),
    );
    draw_rectangle(
        ground.x - 17.0 * scale,
        ground.y - 22.0 * scale,
        34.0 * scale,
        22.0 * scale,
        fill,
    );
    draw_rectangle(
        ground.x - 6.0 * scale,
        ground.y - 18.0 * scale,
        12.0 * scale,
        18.0 * scale,
        Color::new(0.06, 0.09, 0.09, 1.0),
    );
    draw_line(
        ground.x,
        ground.y - 45.0 * scale,
        ground.x,
        ground.y - 60.0 * scale,
        1.0 * scale,
        style::GOLD,
    );
}

fn draw_ford(ground: Vec2, scale: f32, fill: Color) {
    draw_line(
        ground.x - 25.0 * scale,
        ground.y - 5.0 * scale,
        ground.x + 25.0 * scale,
        ground.y + 5.0 * scale,
        7.0 * scale,
        Color::new(0.12, 0.42, 0.52, 0.86),
    );
    draw_line(
        ground.x - 22.0 * scale,
        ground.y - 8.0 * scale,
        ground.x + 22.0 * scale,
        ground.y + 2.0 * scale,
        3.0 * scale,
        fill,
    );
    draw_rectangle(
        ground.x - 3.0 * scale,
        ground.y - 31.0 * scale,
        6.0 * scale,
        31.0 * scale,
        fill,
    );
    draw_triangle(
        vec2(ground.x, ground.y - 42.0 * scale),
        vec2(ground.x - 8.0 * scale, ground.y - 30.0 * scale),
        vec2(ground.x + 8.0 * scale, ground.y - 30.0 * scale),
        Color::new(0.86, 0.70, 0.34, 1.0),
    );
}

fn draw_milestone(ground: Vec2, scale: f32, fill: Color) {
    draw_rectangle(
        ground.x - 6.0 * scale,
        ground.y - 26.0 * scale,
        12.0 * scale,
        26.0 * scale,
        fill,
    );
    draw_triangle(
        vec2(ground.x, ground.y - 35.0 * scale),
        vec2(ground.x - 8.0 * scale, ground.y - 25.0 * scale),
        vec2(ground.x + 8.0 * scale, ground.y - 25.0 * scale),
        fill,
    );
    draw_line(
        ground.x - 23.0 * scale,
        ground.y + 1.0 * scale,
        ground.x + 23.0 * scale,
        ground.y + 1.0 * scale,
        2.0 * scale,
        Color::new(0.78, 0.64, 0.34, 0.62),
    );
}

fn draw_hazard(ground: Vec2, scale: f32, fill: Color) {
    draw_ellipse(
        ground.x,
        ground.y - 5.0 * scale,
        25.0 * scale,
        13.0 * scale,
        0.0,
        Color::new(0.09, 0.18, 0.16, 1.0),
    );
    for offset in [-13.0, 0.0, 13.0] {
        draw_line(
            ground.x + offset * scale,
            ground.y - 5.0 * scale,
            ground.x + (offset + 3.0) * scale,
            ground.y - 29.0 * scale,
            1.0 * scale,
            fill,
        );
    }
}

fn draw_ruined_marker(ground: Vec2, scale: f32, fill: Color) {
    draw_rectangle(
        ground.x - 13.0 * scale,
        ground.y - 15.0 * scale,
        26.0 * scale,
        15.0 * scale,
        fill,
    );
    draw_line(
        ground.x - 11.0 * scale,
        ground.y - 19.0 * scale,
        ground.x + 10.0 * scale,
        ground.y - 36.0 * scale,
        3.0 * scale,
        Color::new(0.70, 0.30, 0.24, 0.90),
    );
}

#[derive(Debug, Clone, Copy)]
pub(super) enum MarkerKind {
    Capital,
    PlayerSettlement,
    Rival,
    Independent,
    Landmark,
    Settlement,
    Lost,
}

fn marker_kind(
    ctx: &UiContext<'_>,
    site: &SiteDef,
    settlement: Option<&SettlementRuntimeState>,
) -> MarkerKind {
    if let Some(settlement) = settlement {
        if settlement.status == SettlementStatus::Lost {
            return MarkerKind::Lost;
        }
        if site.site_type == "capital"
            || settlement
                .memory_tags
                .iter()
                .any(|tag| tag == "charter_seat")
        {
            return MarkerKind::Capital;
        }
        return MarkerKind::PlayerSettlement;
    }
    if ctx.session.rival_controls_site(&site.id) {
        return MarkerKind::Rival;
    }
    if site.category == SiteCategory::Independent {
        return MarkerKind::Independent;
    }
    if site.category == SiteCategory::Landmark {
        return MarkerKind::Landmark;
    }
    MarkerKind::Settlement
}

fn draw_marker_shape(kind: MarkerKind, position: Vec2, radius: f32, fill: Color) {
    match kind {
        MarkerKind::Capital => {
            draw_poly(position.x, position.y, 6, radius + 3.0, 0.0, fill);
            draw_poly_lines(
                position.x,
                position.y,
                6,
                radius + 3.0,
                0.0,
                1.8,
                Color::new(0.96, 0.86, 0.50, 0.92),
            );
        }
        MarkerKind::PlayerSettlement => {
            draw_poly(position.x, position.y, 6, radius + 2.2, 0.0, fill);
            draw_poly_lines(
                position.x,
                position.y,
                6,
                radius + 2.2,
                0.0,
                1.4,
                Color::new(0.92, 0.86, 0.58, 0.82),
            );
        }
        MarkerKind::Rival | MarkerKind::Independent => {
            draw_diamond(position, radius + 2.0, fill);
            draw_circle_lines(
                position.x,
                position.y,
                radius + 2.0,
                1.4,
                Color::new(0.96, 0.84, 0.62, 0.72),
            );
        }
        MarkerKind::Landmark => {
            draw_triangle(
                vec2(position.x, position.y - radius - 2.0),
                vec2(position.x + radius + 2.0, position.y + radius + 1.0),
                vec2(position.x - radius - 2.0, position.y + radius + 1.0),
                fill,
            );
            draw_triangle_lines(
                vec2(position.x, position.y - radius - 2.0),
                vec2(position.x + radius + 2.0, position.y + radius + 1.0),
                vec2(position.x - radius - 2.0, position.y + radius + 1.0),
                1.4,
                Color::new(0.94, 0.82, 0.56, 0.78),
            );
        }
        MarkerKind::Settlement => {
            draw_circle(position.x, position.y, radius, fill);
            draw_circle_lines(
                position.x,
                position.y,
                radius,
                1.4,
                Color::new(0.94, 0.86, 0.60, 0.78),
            );
        }
        MarkerKind::Lost => {
            draw_circle(position.x, position.y, radius, fill);
            draw_line(
                position.x - radius,
                position.y - radius,
                position.x + radius,
                position.y + radius,
                2.0,
                Color::new(0.86, 0.42, 0.32, 0.95),
            );
            draw_line(
                position.x + radius,
                position.y - radius,
                position.x - radius,
                position.y + radius,
                2.0,
                Color::new(0.86, 0.42, 0.32, 0.95),
            );
        }
    }
}

fn draw_diamond(position: Vec2, radius: f32, fill: Color) {
    let top = vec2(position.x, position.y - radius);
    let right = vec2(position.x + radius, position.y);
    let bottom = vec2(position.x, position.y + radius);
    let left = vec2(position.x - radius, position.y);
    draw_triangle(top, right, bottom, fill);
    draw_triangle(top, bottom, left, fill);
}

fn draw_marker_symbol(kind: MarkerKind, position: Vec2, radius: f32) {
    let icon = match kind {
        MarkerKind::Capital => Some(style::IconKind::Castle),
        MarkerKind::PlayerSettlement => Some(style::IconKind::Tree),
        MarkerKind::Rival => Some(style::IconKind::Danger),
        MarkerKind::Independent => Some(style::IconKind::Road),
        MarkerKind::Landmark | MarkerKind::Settlement | MarkerKind::Lost => None,
    };
    if let Some(icon) = icon {
        style::draw_icon(
            icon,
            position,
            (radius * 1.55).clamp(15.0, 24.0),
            Color::new(0.03, 0.025, 0.015, 0.92),
        );
    }
}

fn draw_supply_ring(ctx: &UiContext<'_>, site: &SiteDef, position: Vec2, radius: f32) {
    let connected = ctx.session.is_site_in_capital_network(ctx.data, &site.id);
    let color = if connected {
        Color::new(0.42, 0.86, 0.36, 0.62)
    } else {
        Color::new(0.94, 0.56, 0.20, 0.74)
    };
    draw_circle_lines(position.x, position.y, radius + 10.0, 2.2, color);
}

fn draw_site_label(site: &SiteDef, position: Vec2, radius: f32, selected: bool) {
    let x = position.x + radius + 6.0;
    let y = position.y - radius - 3.0;
    let color = if selected {
        style::TEXT_BRIGHT
    } else {
        Color::new(0.89, 0.84, 0.69, 0.88)
    };
    draw_ui_text_ex(
        &site.name,
        x + 1.0,
        y + 1.0,
        TextStyle::new(13.0, Color::new(0.02, 0.018, 0.012, 0.80)).params(),
    );
    draw_ui_text_ex(&site.name, x, y, TextStyle::new(13.0, color).params());
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

fn marker_color(
    ctx: &UiContext<'_>,
    site: &SiteDef,
    settlement: Option<&SettlementRuntimeState>,
) -> Color {
    if let Some(settlement) = settlement {
        if site.site_type == "capital"
            || settlement
                .memory_tags
                .iter()
                .any(|tag| tag == "charter_seat")
        {
            return Color::new(0.86, 0.67, 0.26, 1.0);
        }
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
    if ctx.session.rival_controls_site(&site.id) {
        return Color::new(0.72, 0.24, 0.18, 1.0);
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
