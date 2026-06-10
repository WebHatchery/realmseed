//! Site markers, map labels, and picking for the strategic map.

use super::map::MapView;
use super::{style, MapOverlay, UiContext};
use crate::data::{SettlementTier, SiteCategory, SiteDef};
use crate::state::{SettlementRuntimeState, SettlementStatus};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::RectExt;

pub(super) fn draw_sites(ctx: &UiContext<'_>, view: &MapView) {
    for site in &ctx.data.sites {
        if ctx.session.is_known(&site.id) {
            draw_known_site(ctx, view, site);
        } else if ctx.session.is_adjacent_unknown(ctx.data, &site.id) {
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
        let radius = marker_radius(site, ctx.session.settlement_at_site(&site.id)).max(8.0) + 7.0;
        (position.distance(mouse) <= radius).then(|| site.id.clone())
    })
}

fn draw_known_site(ctx: &UiContext<'_>, view: &MapView, site: &SiteDef) {
    let position = view.site_position(site);
    let selected = ctx.session.selected_site_id == site.id;
    let settlement = ctx.session.settlement_at_site(&site.id);
    let radius = marker_radius(site, settlement) * ctx.camera_zoom.clamp(0.9, 1.3);
    let fill = marker_color(ctx, site, settlement);
    let kind = marker_kind(ctx, site, settlement);

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
    let selected = ctx.session.selected_site_id == site.id;
    let radius = 7.0 * ctx.camera_zoom.clamp(0.9, 1.25);
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
        draw_text_ex(
            "Scout",
            position.x + radius + 5.0,
            position.y - radius - 2.0,
            TextStyle::new(12.0, style::TEXT_BRIGHT).params(),
        );
    }
}

#[derive(Debug, Clone, Copy)]
enum MarkerKind {
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
    draw_text_ex(
        &site.name,
        x + 1.0,
        y + 1.0,
        TextStyle::new(13.0, Color::new(0.02, 0.018, 0.012, 0.80)).params(),
    );
    draw_text_ex(&site.name, x, y, TextStyle::new(13.0, color).params());
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
