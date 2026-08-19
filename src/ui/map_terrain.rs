//! Layered terrain painting for the close strategic map view.

use super::map::MapView;
use super::{color_from_array, UiContext};
use macroquad::prelude::*;

pub(super) fn draw_terrain(ctx: &UiContext<'_>, view: &MapView) {
    draw_rectangle(
        view.rect.x,
        view.rect.y,
        view.rect.w,
        view.rect.h,
        Color::new(0.025, 0.075, 0.085, 1.0),
    );
    draw_world_ground(ctx, view);

    for y in 0..ctx.data.terrain.height {
        for x in 0..ctx.data.terrain.width {
            let Some(terrain) = ctx.data.terrain_at(x as i32, y as i32) else {
                continue;
            };
            let quad = view.tile_quad(x as i32, y as i32);
            if !view.is_visible(view.tile_center(x as i32, y as i32), view.tile_size() * 1.2) {
                continue;
            }

            let mut color = terrain_color(color_from_array(terrain.color), x, y);
            color.a = terrain_fill_alpha(terrain.id.as_str());
            if matches!(terrain.id.as_str(), "hills" | "mountain") {
                draw_tile_shadow(quad, view.tile_size());
            }
            draw_tile(quad, color);
            draw_tile_edge(quad, terrain.id.as_str(), view.tile_size());
            draw_terrain_detail(
                terrain.id.as_str(),
                quad,
                view.tile_center(x as i32, y as i32),
                x,
                y,
                view,
            );
        }
    }

    draw_landscape_landforms(ctx, view);
    draw_environmental_sprite_anchors(ctx, view);
    draw_region_washes(ctx, view);
    draw_river_network(ctx, view);
    draw_coastlines(ctx, view);
}

fn draw_environmental_sprite_anchors(ctx: &UiContext<'_>, view: &MapView) {
    if let Some(grove) = ctx.sprites.grove {
        for (x, y, width, height) in [
            (5, 8, 78.0, 76.0),
            (15, 12, 70.0, 68.0),
            (11, 27, 74.0, 72.0),
        ] {
            if !ctx
                .data
                .terrain_at(x, y)
                .is_some_and(|terrain| terrain.id == "forest")
            {
                continue;
            }
            let position = view.tile_center(x, y);
            if !view.is_visible(position, width) {
                continue;
            }
            draw_grounded_sprite(
                grove,
                position,
                vec2(width * view.scale(), height * view.scale()),
                0.72,
            );
        }
    }

    if let Some(gate) = ctx.sprites.gate {
        let (x, y) = (38, 10);
        if ctx
            .data
            .terrain_at(x, y)
            .is_some_and(|terrain| terrain.id == "mountain" || terrain.id == "hills")
        {
            let position = view.tile_center(x, y);
            if view.is_visible(position, 90.0) {
                draw_grounded_sprite(
                    gate,
                    position,
                    vec2(78.0 * view.scale(), 82.0 * view.scale()),
                    0.58,
                );
            }
        }
    }
}

fn draw_grounded_sprite(texture: &Texture2D, position: Vec2, size: Vec2, alpha: f32) {
    draw_ellipse(
        position.x,
        position.y + size.y * 0.30,
        size.x * 0.34,
        size.y * 0.10,
        0.0,
        Color::new(0.01, 0.018, 0.014, alpha * 0.42),
    );
    draw_texture_ex(
        texture,
        position.x - size.x * 0.5,
        position.y - size.y * 0.72,
        Color::new(1.0, 1.0, 1.0, alpha),
        DrawTextureParams {
            dest_size: Some(size),
            ..Default::default()
        },
    );
}

fn draw_world_ground(ctx: &UiContext<'_>, view: &MapView) {
    let top_left = view.tile_quad(0, 0)[0];
    let top_right = view.tile_quad(ctx.data.terrain.width as i32 - 1, 0)[1];
    let bottom_right = view.tile_quad(
        ctx.data.terrain.width as i32 - 1,
        ctx.data.terrain.height as i32 - 1,
    )[2];
    let bottom_left = view.tile_quad(0, ctx.data.terrain.height as i32 - 1)[3];
    let ground = Color::new(0.20, 0.29, 0.16, 1.0);
    draw_triangle(top_left, top_right, bottom_right, ground);
    draw_triangle(top_left, bottom_right, bottom_left, ground);
}

fn terrain_fill_alpha(terrain_id: &str) -> f32 {
    match terrain_id {
        "plains" => 0.68,
        "forest" => 0.74,
        "hills" => 0.78,
        "mountain" => 0.92,
        "river" | "coast" => 0.96,
        "marsh" => 0.82,
        _ => 0.86,
    }
}

fn draw_landscape_landforms(ctx: &UiContext<'_>, view: &MapView) {
    for (x, y, variant) in [(9, 10, 0), (14, 15, 1), (10, 22, 2), (17, 8, 3)] {
        if ctx
            .data
            .terrain_at(x, y)
            .is_some_and(|terrain| terrain.id == "forest")
        {
            draw_forest_mass(view.tile_center(x, y), view.tile_size(), variant);
        }
    }

    for (x, y, variant) in [(33, 8, 0), (36, 9, 1), (39, 10, 2), (42, 12, 3)] {
        if ctx
            .data
            .terrain_at(x, y)
            .is_some_and(|terrain| terrain.id == "mountain" || terrain.id == "hills")
        {
            draw_big_peak(view.tile_center(x, y), view.tile_size(), variant);
        }
    }

    for (x, y, width) in [(23, 18, 2.0), (29, 23, 2.6), (34, 21, 1.8)] {
        if ctx
            .data
            .terrain_at(x, y)
            .is_some_and(|terrain| terrain.id == "plains")
        {
            draw_field_patch(view.tile_center(x, y), view.tile_size(), width);
        }
    }
}

fn draw_forest_mass(center: Vec2, size: f32, variant: usize) {
    if !center.is_finite() {
        return;
    }
    draw_circle(
        center.x,
        center.y,
        size * 2.0,
        Color::new(0.015, 0.075, 0.04, 0.42),
    );
    let offsets = [
        vec2(-size * 1.25, size * 0.35),
        vec2(-size * 0.42, -size * 0.72),
        vec2(size * 0.48, -size * 0.35),
        vec2(size * 1.18, size * 0.42),
    ];
    for (index, offset) in offsets.iter().enumerate() {
        draw_tree(
            center + *offset,
            size * (0.72 + (index + variant) as f32 * 0.04),
            index as u32 + variant as u32,
        );
    }
}

fn draw_big_peak(center: Vec2, size: f32, variant: usize) {
    let shift = (variant as f32 - 1.5) * size * 0.18;
    let ground = center + vec2(shift, size * 1.15);
    let peak = center + vec2(shift, -size * 3.0);
    let left = ground + vec2(-size * 2.15, 0.0);
    let right = ground + vec2(size * 2.15, 0.0);
    draw_triangle(
        peak + vec2(size * 0.18, size * 0.28),
        left + vec2(0.0, size * 0.34),
        right + vec2(0.0, size * 0.34),
        Color::new(0.035, 0.045, 0.05, 0.56),
    );
    draw_triangle(
        peak,
        left,
        center + vec2(0.0, size * 0.70),
        Color::new(0.43, 0.45, 0.45, 1.0),
    );
    draw_triangle(
        peak,
        center + vec2(0.0, size * 0.70),
        right,
        Color::new(0.22, 0.24, 0.27, 1.0),
    );
    draw_triangle(
        peak + vec2(0.0, size * 0.16),
        peak + vec2(-size * 0.42, size * 0.82),
        peak + vec2(size * 0.34, size * 0.66),
        Color::new(0.86, 0.87, 0.83, 0.80),
    );
}

fn draw_field_patch(center: Vec2, size: f32, width: f32) {
    draw_ellipse(
        center.x,
        center.y + size * 0.30,
        size * width,
        size * 0.50,
        0.0,
        Color::new(0.60, 0.58, 0.25, 0.18),
    );
    for index in 0..4 {
        let y = center.y - size * 0.10 + index as f32 * size * 0.16;
        draw_line(
            center.x - size * (width - 0.25),
            y,
            center.x + size * (width - 0.25),
            y - size * 0.05,
            1.0,
            Color::new(0.82, 0.73, 0.34, 0.38),
        );
    }
}

fn terrain_color(base: Color, x: usize, y: usize) -> Color {
    let hash = ((x as u32).wrapping_mul(37) ^ (y as u32).wrapping_mul(53)) % 19;
    let variation = 0.99 + hash as f32 * 0.001;
    Color::new(
        (base.r * variation).clamp(0.0, 1.0),
        (base.g * variation).clamp(0.0, 1.0),
        (base.b * variation).clamp(0.0, 1.0),
        base.a,
    )
}

fn draw_tile_shadow(quad: [Vec2; 4], tile_size: f32) {
    let offset = vec2(0.0, tile_size * 0.16);
    let shadow = quad.map(|point| point + offset);
    draw_triangle(
        shadow[0],
        shadow[1],
        shadow[2],
        Color::new(0.01, 0.018, 0.018, 0.34),
    );
    draw_triangle(
        shadow[0],
        shadow[2],
        shadow[3],
        Color::new(0.01, 0.018, 0.018, 0.34),
    );
}

fn draw_tile(quad: [Vec2; 4], color: Color) {
    let center = quad
        .iter()
        .copied()
        .fold(Vec2::ZERO, |sum, point| sum + point)
        * 0.25;
    let filled = quad.map(|point| center + (point - center) * 1.045);
    draw_triangle(filled[0], filled[1], filled[2], color);
    draw_triangle(filled[0], filled[2], filled[3], color);
}

fn draw_tile_edge(quad: [Vec2; 4], terrain_id: &str, tile_size: f32) {
    if !matches!(terrain_id, "mountain" | "river" | "coast") {
        return;
    }
    let edge = match terrain_id {
        "mountain" => Color::new(0.06, 0.065, 0.07, 0.14),
        "river" | "coast" => Color::new(0.38, 0.76, 0.82, 0.12),
        _ => Color::new(0.03, 0.045, 0.035, 0.055),
    };
    for index in 0..4 {
        let start = quad[index];
        let end = quad[(index + 1) % 4];
        draw_line(
            start.x,
            start.y,
            end.x,
            end.y,
            (tile_size * 0.035).clamp(0.35, 0.8),
            edge,
        );
    }
}

fn draw_terrain_detail(
    terrain_id: &str,
    quad: [Vec2; 4],
    center: Vec2,
    x: usize,
    y: usize,
    view: &MapView,
) {
    let size = view.tile_size();
    let seed = (x as u32 * 11 + y as u32 * 23) % 7;
    match terrain_id {
        "plains" => draw_plains_detail(center, quad, size, seed),
        "forest" => draw_forest_detail(center, size, seed),
        "hills" => draw_hill_detail(center, quad, size, seed),
        "mountain" => draw_mountain_detail(center, quad, size, seed),
        "river" => draw_water_detail(center, size, seed),
        "coast" => draw_coast_detail(center, quad, size, seed),
        "marsh" => draw_marsh_detail(center, size, seed),
        _ => {}
    }
}

fn draw_plains_detail(center: Vec2, quad: [Vec2; 4], size: f32, seed: u32) {
    let tint = if seed.is_multiple_of(3) {
        Color::new(0.72, 0.70, 0.34, 0.22)
    } else {
        Color::new(0.55, 0.64, 0.29, 0.20)
    };
    draw_circle(
        center.x - size * 0.18,
        center.y + size * 0.06,
        size * 0.22,
        tint,
    );
    draw_circle(
        center.x + size * 0.22,
        center.y - size * 0.08,
        size * 0.14,
        tint,
    );
    if seed % 2 == 0 {
        draw_line(
            quad[3].x + size * 0.12,
            quad[3].y - size * 0.12,
            quad[2].x - size * 0.14,
            quad[2].y - size * 0.12,
            (size * 0.07).clamp(0.8, 1.5),
            Color::new(0.82, 0.73, 0.38, 0.28),
        );
    }
    draw_grass_tuft(center + vec2(-size * 0.28, size * 0.11), size * 0.18);
}

fn draw_forest_detail(center: Vec2, size: f32, seed: u32) {
    draw_circle(
        center.x,
        center.y - size * 0.08,
        size * 0.68,
        if seed.is_multiple_of(3) {
            Color::new(0.025, 0.14, 0.07, 0.86)
        } else {
            Color::new(0.02, 0.11, 0.06, 0.84)
        },
    );
    let cluster = if seed.is_multiple_of(3) { 2 } else { 1 };
    for index in 0..cluster {
        let offset = match index {
            0 => vec2(-size * 0.12, size * 0.06),
            1 => vec2(size * 0.16, -size * 0.10),
            _ => vec2(size * 0.22, size * 0.06),
        };
        draw_tree(
            center + offset,
            size * (0.48 + index as f32 * 0.06),
            seed + index,
        );
    }
}

fn draw_tree(position: Vec2, size: f32, seed: u32) {
    draw_ellipse(
        position.x,
        position.y + size * 0.25,
        size * 0.34,
        size * 0.14,
        0.0,
        Color::new(0.015, 0.035, 0.022, 0.32),
    );
    draw_rectangle(
        position.x - size * 0.06,
        position.y + size * 0.02,
        size * 0.12,
        size * 0.32,
        Color::new(0.16, 0.12, 0.07, 0.88),
    );
    let dark = if seed.is_multiple_of(3) {
        Color::new(0.035, 0.16, 0.085, 1.0)
    } else {
        Color::new(0.025, 0.12, 0.065, 1.0)
    };
    let mid = Color::new(0.08, 0.28, 0.12, 1.0);
    draw_circle(position.x, position.y - size * 0.20, size * 0.36, dark);
    draw_circle(
        position.x - size * 0.16,
        position.y + size * 0.02,
        size * 0.26,
        mid,
    );
    draw_circle(
        position.x + size * 0.16,
        position.y + size * 0.02,
        size * 0.24,
        mid,
    );
    draw_line(
        position.x - size * 0.08,
        position.y - size * 0.26,
        position.x + size * 0.15,
        position.y - size * 0.05,
        (size * 0.045).clamp(0.5, 1.0),
        Color::new(0.42, 0.60, 0.22, 0.45),
    );
}

fn draw_hill_detail(center: Vec2, quad: [Vec2; 4], size: f32, seed: u32) {
    let shift = if seed % 2 == 0 {
        -size * 0.10
    } else {
        size * 0.08
    };
    let peak = center + vec2(shift, -size * 0.42);
    let left = vec2(quad[0].x + size * 0.12, quad[3].y - size * 0.04);
    let right = vec2(quad[1].x - size * 0.12, quad[2].y - size * 0.04);
    draw_triangle(
        peak + vec2(0.0, size * 0.10),
        left,
        right,
        Color::new(0.20, 0.18, 0.12, 0.34),
    );
    draw_triangle(
        peak,
        left,
        peak + (right - peak) * 0.48,
        Color::new(0.58, 0.50, 0.30, 0.34),
    );
    draw_line(
        peak.x,
        peak.y + size * 0.05,
        center.x + size * 0.24,
        center.y + size * 0.20,
        (size * 0.05).clamp(0.55, 1.2),
        Color::new(0.84, 0.72, 0.40, 0.30),
    );
}

fn draw_mountain_detail(center: Vec2, quad: [Vec2; 4], size: f32, seed: u32) {
    let offset = if seed.is_multiple_of(2) {
        -size * 0.16
    } else {
        size * 0.14
    };
    let peak = center + vec2(offset, -size * 0.88);
    let left = vec2(quad[0].x + size * 0.02, quad[3].y + size * 0.02);
    let right = vec2(quad[1].x - size * 0.02, quad[2].y + size * 0.02);
    draw_triangle(
        peak + vec2(0.0, size * 0.11),
        left + vec2(0.0, size * 0.11),
        right + vec2(0.0, size * 0.11),
        Color::new(0.07, 0.08, 0.09, 0.62),
    );
    draw_triangle(
        peak,
        left,
        center + vec2(0.0, size * 0.08),
        Color::new(0.48, 0.50, 0.50, 0.90),
    );
    draw_triangle(
        peak,
        center + vec2(0.0, size * 0.08),
        right,
        Color::new(0.25, 0.27, 0.29, 0.98),
    );
    draw_triangle(
        peak + vec2(0.0, size * 0.12),
        peak + vec2(-size * 0.15, size * 0.30),
        peak + vec2(size * 0.13, size * 0.25),
        Color::new(0.82, 0.84, 0.80, 0.78),
    );
    draw_line(
        peak.x,
        peak.y,
        peak.x + size * 0.34,
        peak.y + size * 0.65,
        (size * 0.05).clamp(0.6, 1.2),
        Color::new(0.72, 0.74, 0.70, 0.36),
    );
}

fn draw_water_detail(center: Vec2, size: f32, seed: u32) {
    for index in 0..2 {
        let y = center.y + (index as f32 - 0.5) * size * 0.22;
        let x = center.x
            + if seed % 2 == 0 {
                -size * 0.12
            } else {
                size * 0.12
            };
        draw_line(
            x - size * 0.24,
            y,
            x + size * 0.22,
            y - size * 0.04,
            (size * 0.045).clamp(0.5, 1.0),
            Color::new(0.62, 0.88, 0.90, 0.52),
        );
    }
}

fn draw_coast_detail(center: Vec2, quad: [Vec2; 4], size: f32, seed: u32) {
    let sand = Color::new(0.72, 0.64, 0.38, 0.36);
    draw_line(
        quad[0].x + size * 0.12,
        quad[0].y + size * 0.02,
        quad[1].x - size * 0.14,
        quad[1].y + size * 0.02,
        (size * 0.13).clamp(1.0, 2.4),
        sand,
    );
    if seed % 2 == 0 {
        draw_circle(
            center.x + size * 0.18,
            center.y,
            size * 0.07,
            Color::new(0.80, 0.76, 0.46, 0.34),
        );
    }
}

fn draw_marsh_detail(center: Vec2, size: f32, seed: u32) {
    draw_ellipse(
        center.x + size * 0.10,
        center.y + size * 0.08,
        size * 0.27,
        size * 0.12,
        0.0,
        Color::new(0.08, 0.22, 0.22, 0.58),
    );
    for index in 0..3 {
        let x = center.x + (index as f32 - 1.0) * size * 0.15;
        let y = center.y + size * 0.17 + (seed % 2) as f32 * size * 0.03;
        draw_line(
            x,
            y,
            x + size * 0.03,
            y - size * 0.28,
            (size * 0.04).clamp(0.5, 1.0),
            Color::new(0.64, 0.68, 0.32, 0.62),
        );
    }
}

fn draw_grass_tuft(position: Vec2, size: f32) {
    let color = Color::new(0.72, 0.78, 0.36, 0.44);
    draw_line(
        position.x,
        position.y + size * 0.26,
        position.x - size * 0.16,
        position.y - size * 0.18,
        0.8,
        color,
    );
    draw_line(
        position.x,
        position.y + size * 0.26,
        position.x + size * 0.10,
        position.y - size * 0.24,
        0.8,
        color,
    );
}

fn draw_region_washes(ctx: &UiContext<'_>, view: &MapView) {
    for region in &ctx.data.regions {
        let position = view.tile_center(region.label_position.x, region.label_position.y);
        if !view.is_visible(position, 150.0) {
            continue;
        }
        let tint = color_from_array(region.tint);
        draw_circle(
            position.x,
            position.y,
            view.tile_size() * 5.2,
            Color::new(tint.r, tint.g, tint.b, 0.055),
        );
    }
}

fn draw_river_network(ctx: &UiContext<'_>, view: &MapView) {
    for y in 0..ctx.data.terrain.height {
        for x in 0..ctx.data.terrain.width {
            let Some(terrain) = ctx.data.terrain_at(x as i32, y as i32) else {
                continue;
            };
            if terrain.id != "river" {
                continue;
            }
            let center = view.tile_center(x as i32, y as i32);
            for (nx, ny) in [(x + 1, y), (x, y + 1)] {
                let Some(next) = ctx.data.terrain_at(nx as i32, ny as i32) else {
                    continue;
                };
                if next.id != "river" {
                    continue;
                }
                let end = view.tile_center(nx as i32, ny as i32);
                draw_line(
                    center.x,
                    center.y,
                    end.x,
                    end.y,
                    view.tile_size() * 0.54,
                    Color::new(0.05, 0.22, 0.32, 0.54),
                );
                draw_line(
                    center.x,
                    center.y - view.tile_size() * 0.02,
                    end.x,
                    end.y - view.tile_size() * 0.02,
                    view.tile_size() * 0.30,
                    Color::new(0.15, 0.55, 0.69, 0.92),
                );
            }
        }
    }
}

fn draw_coastlines(ctx: &UiContext<'_>, view: &MapView) {
    for y in 0..ctx.data.terrain.height {
        for x in 0..ctx.data.terrain.width {
            let Some(terrain) = ctx.data.terrain_at(x as i32, y as i32) else {
                continue;
            };
            if terrain.id != "coast" {
                continue;
            }
            let quad = view.tile_quad(x as i32, y as i32);
            if x > 0
                && ctx
                    .data
                    .terrain_at(x as i32 - 1, y as i32)
                    .is_some_and(|neighbor| neighbor.id != "coast")
            {
                draw_line(
                    quad[0].x,
                    quad[0].y,
                    quad[3].x,
                    quad[3].y,
                    1.4,
                    Color::new(0.80, 0.70, 0.42, 0.40),
                );
            }
            if y > 0
                && ctx
                    .data
                    .terrain_at(x as i32, y as i32 - 1)
                    .is_some_and(|neighbor| neighbor.id != "coast")
            {
                draw_line(
                    quad[0].x,
                    quad[0].y,
                    quad[1].x,
                    quad[1].y,
                    1.4,
                    Color::new(0.80, 0.70, 0.42, 0.40),
                );
            }
        }
    }
}
