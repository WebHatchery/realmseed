//! Realmseed-specific UI styling primitives.

use macroquad::prelude::*;
use macroquad_toolkit::prelude::ButtonTone;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;
use macroquad_toolkit::ui::RectExt;
use std::cell::RefCell;

pub(super) const INK: Color = Color::new(0.015, 0.026, 0.030, 1.0);
pub(super) const PANEL: Color = Color::new(0.026, 0.045, 0.047, 0.94);
pub(super) const PANEL_DARK: Color = Color::new(0.014, 0.026, 0.030, 0.96);
pub(super) const GOLD: Color = Color::new(0.86, 0.68, 0.36, 1.0);
pub(super) const GOLD_DIM: Color = Color::new(0.56, 0.43, 0.24, 0.78);
pub(super) const CYAN: Color = Color::new(0.20, 0.82, 0.90, 1.0);
pub(super) const GREEN: Color = Color::new(0.44, 0.75, 0.28, 1.0);
pub(super) const RED: Color = Color::new(0.90, 0.24, 0.20, 1.0);
pub(super) const TEXT: Color = Color::new(0.90, 0.86, 0.74, 1.0);
pub(super) const TEXT_BRIGHT: Color = Color::new(0.98, 0.94, 0.84, 1.0);
pub(super) const TEXT_DIM: Color = Color::new(0.62, 0.64, 0.58, 1.0);

const TOOLTIP_DELAY: f64 = 0.45;
const TOOLTIP_FADE_IN: f64 = 0.12;
const TOOLTIP_FADE_OUT: f64 = 0.22;

#[derive(Debug, Clone)]
struct HoverTooltipState {
    id: String,
    text: String,
    anchor: Vec2,
    entered_at: f64,
    last_hover_at: f64,
}

thread_local! {
    static HOVER_TOOLTIP: RefCell<Option<HoverTooltipState>> = const { RefCell::new(None) };
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum IconKind {
    Food,
    Timber,
    Stone,
    Wealth,
    People,
    Actions,
    Crown,
    Castle,
    Tree,
    Road,
    Danger,
    Compass,
}

pub(super) fn draw_panel(rect: Rect) {
    draw_rectangle(
        rect.x + 4.0,
        rect.y + 6.0,
        rect.w,
        rect.h,
        Color::new(0.0, 0.0, 0.0, 0.32),
    );
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, PANEL);
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        44.0_f32.min(rect.h),
        Color::new(0.018, 0.032, 0.036, 0.82),
    );
    draw_chamfer_lines(rect, GOLD_DIM, 1.0);
    draw_chamfer_lines(rect.inset(7.0), Color::new(0.88, 0.74, 0.48, 0.08), 1.0);
    draw_line(
        rect.x + 14.0,
        rect.y + 1.0,
        rect.right() - 14.0,
        rect.y + 1.0,
        1.0,
        Color::new(1.0, 0.86, 0.54, 0.22),
    );
}

pub(super) fn draw_band(rect: Rect) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, PANEL_DARK);
    draw_line(rect.x, rect.y, rect.right(), rect.y, 1.0, GOLD_DIM);
    draw_line(
        rect.x,
        rect.bottom(),
        rect.right(),
        rect.bottom(),
        1.0,
        Color::new(0.28, 0.22, 0.13, 0.70),
    );
}

pub(super) fn draw_panel_title(text: &str, x: f32, y: f32) {
    draw_ui_text_ex(text, x, y, TextStyle::new(15.0, GOLD).params());
}

pub(super) fn draw_hover_tooltip(id: &str, hover_rect: Rect, text: &str, mouse: Vec2) {
    let now = get_time();
    let hovering = hover_rect.contains_point(mouse);
    let mut draw: Option<(String, Vec2, f32)> = None;

    HOVER_TOOLTIP.with(|cell| {
        let mut state = cell.borrow_mut();
        if hovering {
            let anchor = tooltip_anchor(hover_rect);
            match state.as_mut() {
                Some(current) if current.id == id => {
                    current.text = text.to_owned();
                    current.anchor = anchor;
                    current.last_hover_at = now;
                }
                _ => {
                    *state = Some(HoverTooltipState {
                        id: id.to_owned(),
                        text: text.to_owned(),
                        anchor,
                        entered_at: now,
                        last_hover_at: now,
                    });
                }
            }
        }

        let Some(current) = state.as_mut() else {
            return;
        };
        if current.id != id {
            return;
        }

        let visible_age = now - current.entered_at - TOOLTIP_DELAY;
        if visible_age < 0.0 {
            return;
        }

        let leave_age = if hovering {
            0.0
        } else {
            now - current.last_hover_at
        };
        if leave_age > TOOLTIP_FADE_OUT {
            *state = None;
            return;
        }

        let fade_in = (visible_age / TOOLTIP_FADE_IN).clamp(0.0, 1.0);
        let fade_out = if hovering {
            1.0
        } else {
            (1.0 - leave_age / TOOLTIP_FADE_OUT).clamp(0.0, 1.0)
        };
        draw = Some((
            current.text.clone(),
            current.anchor,
            (fade_in * fade_out) as f32,
        ));
    });

    if let Some((tooltip_text, anchor, alpha)) = draw {
        draw_tooltip_alpha(&tooltip_text, anchor, alpha);
    }
}

fn tooltip_anchor(rect: Rect) -> Vec2 {
    let max_width = 300.0;
    let x = (rect.x + rect.w * 0.5 - max_width * 0.5 - 14.0)
        .clamp(6.0, (screen_width() - max_width - 20.0).max(6.0));
    vec2(x, rect.bottom() + 6.0)
}

fn draw_tooltip_alpha(text: &str, anchor: Vec2, alpha: f32) {
    if alpha <= 0.02 {
        return;
    }
    let tooltip_style = TooltipStyle {
        background: Color::new(0.020, 0.030, 0.032, 0.94 * alpha),
        border: with_alpha(GOLD, 0.78 * alpha),
        text: with_alpha(TEXT, alpha),
        padding: 7.0,
        max_width: 300.0,
        font_size: 13.0,
        line_gap: 2.0,
    };
    draw_tooltip_styled(text, anchor, &tooltip_style, None);
}

pub(super) fn draw_button_frame(
    rect: Rect,
    tone: ButtonTone,
    enabled: bool,
    hovered: bool,
    pressed: bool,
) {
    let fill = button_fill(tone, enabled, hovered, pressed);
    let border = if matches!(tone, ButtonTone::Primary) {
        CYAN
    } else {
        GOLD_DIM
    };
    let bright = if matches!(tone, ButtonTone::Primary) {
        with_alpha(CYAN, if hovered { 0.92 } else { 0.58 })
    } else {
        with_alpha(GOLD, if hovered { 0.82 } else { 0.42 })
    };
    let cut = 5.0_f32.min(rect.h * 0.22);
    let body = Rect::new(rect.x + 4.0, rect.y, rect.w - 8.0, rect.h);

    draw_rectangle(
        rect.x + 2.0,
        rect.y + 3.0,
        rect.w,
        rect.h,
        Color::new(0.0, 0.0, 0.0, 0.25),
    );
    draw_rectangle(body.x, body.y, body.w, body.h, fill);
    draw_rectangle(rect.x, rect.y + cut, rect.w, rect.h - cut * 2.0, fill);
    draw_triangle(
        vec2(rect.x + 4.0, rect.y),
        vec2(rect.x + cut + 4.0, rect.y),
        vec2(rect.x, rect.y + cut),
        fill,
    );
    draw_triangle(
        vec2(rect.right() - 4.0, rect.y),
        vec2(rect.right() - cut - 4.0, rect.y),
        vec2(rect.right(), rect.y + cut),
        fill,
    );
    draw_triangle(
        vec2(rect.x + 4.0, rect.bottom()),
        vec2(rect.x + cut + 4.0, rect.bottom()),
        vec2(rect.x, rect.bottom() - cut),
        fill,
    );
    draw_triangle(
        vec2(rect.right() - 4.0, rect.bottom()),
        vec2(rect.right() - cut - 4.0, rect.bottom()),
        vec2(rect.right(), rect.bottom() - cut),
        fill,
    );

    draw_chamfer_lines(rect, border, if hovered { 1.4 } else { 1.0 });
    draw_chamfer_lines(
        rect.inset(3.0),
        Color::new(bright.r, bright.g, bright.b, 0.18),
        1.0,
    );
    draw_line(
        rect.x + cut + 4.0,
        rect.y + 2.0,
        rect.right() - cut - 4.0,
        rect.y + 2.0,
        1.0,
        Color::new(1.0, 0.88, 0.56, if enabled { 0.22 } else { 0.06 }),
    );
    draw_button_corner_marks(rect, bright);
    if pressed {
        draw_rectangle(
            rect.x + 2.0,
            rect.y + 2.0,
            rect.w - 4.0,
            rect.h - 4.0,
            Color::new(0.0, 0.0, 0.0, 0.10),
        );
    }
}

fn button_fill(tone: ButtonTone, enabled: bool, hovered: bool, pressed: bool) -> Color {
    let mut fill = match tone {
        ButtonTone::Primary => Color::new(0.025, 0.155, 0.176, if enabled { 0.96 } else { 0.40 }),
        ButtonTone::Positive => Color::new(0.075, 0.135, 0.070, if enabled { 0.95 } else { 0.38 }),
        ButtonTone::Warning => Color::new(0.17, 0.118, 0.050, if enabled { 0.95 } else { 0.38 }),
        ButtonTone::Danger => Color::new(0.18, 0.052, 0.050, if enabled { 0.95 } else { 0.38 }),
        ButtonTone::Muted => Color::new(0.026, 0.036, 0.036, 0.68),
        ButtonTone::Secondary => Color::new(0.025, 0.037, 0.038, if enabled { 0.94 } else { 0.36 }),
    };
    if hovered {
        fill.r = (fill.r + 0.038).min(1.0);
        fill.g = (fill.g + 0.052).min(1.0);
        fill.b = (fill.b + 0.050).min(1.0);
    }
    if pressed {
        fill.r *= 0.82;
        fill.g *= 0.82;
        fill.b *= 0.82;
    }
    fill
}

fn draw_button_corner_marks(rect: Rect, color: Color) {
    let mark = 5.0_f32.min(rect.h * 0.18);
    let alpha = multiply_alpha(color, 0.82);
    draw_line(
        rect.x + 6.0,
        rect.y + 5.0,
        rect.x + 6.0 + mark,
        rect.y + 5.0,
        1.0,
        alpha,
    );
    draw_line(
        rect.x + 6.0,
        rect.y + 5.0,
        rect.x + 6.0,
        rect.y + 5.0 + mark,
        1.0,
        alpha,
    );
    draw_line(
        rect.right() - 6.0,
        rect.y + 5.0,
        rect.right() - 6.0 - mark,
        rect.y + 5.0,
        1.0,
        alpha,
    );
    draw_line(
        rect.right() - 6.0,
        rect.y + 5.0,
        rect.right() - 6.0,
        rect.y + 5.0 + mark,
        1.0,
        alpha,
    );
}

pub(super) fn draw_divider(x: f32, y: f32, w: f32) {
    draw_line(x, y, x + w, y, 1.0, Color::new(0.74, 0.58, 0.32, 0.22));
}

pub(super) fn draw_vertical_divider(x: f32, y: f32, h: f32) {
    draw_line(x, y, x, y + h, 1.0, Color::new(0.74, 0.58, 0.32, 0.20));
}

pub(super) fn draw_icon(kind: IconKind, center: Vec2, size: f32, color: Color) {
    match kind {
        IconKind::Food => draw_wheat(center, size, color),
        IconKind::Timber => draw_tree(center, size, color),
        IconKind::Stone => draw_stone(center, size, color),
        IconKind::Wealth => draw_coin(center, size, color),
        IconKind::People => draw_people(center, size, color),
        IconKind::Actions | IconKind::Compass => draw_compass(center, size, color),
        IconKind::Crown => draw_crown(center, size, color),
        IconKind::Castle => draw_castle(center, size, color),
        IconKind::Tree => draw_crest_tree(center, size, color),
        IconKind::Road => draw_road(center, size, color),
        IconKind::Danger => draw_skull(center, size, color),
    }
}

pub(super) fn draw_framed_icon(kind: IconKind, center: Vec2, size: f32, fill: Color) {
    draw_poly(center.x, center.y, 6, size * 0.64, 0.0, fill);
    draw_poly_lines(center.x, center.y, 6, size * 0.64, 0.0, 1.4, GOLD);
    draw_icon(kind, center, size * 0.60, TEXT_BRIGHT);
}

fn draw_chamfer_lines(rect: Rect, color: Color, width: f32) {
    let cut = 8.0_f32.min(rect.w * 0.12).min(rect.h * 0.12);
    let x = rect.x;
    let y = rect.y;
    let r = rect.right();
    let b = rect.bottom();
    draw_line(x + cut, y, r - cut, y, width, color);
    draw_line(r - cut, y, r, y + cut, width, color);
    draw_line(r, y + cut, r, b - cut, width, color);
    draw_line(r, b - cut, r - cut, b, width, color);
    draw_line(r - cut, b, x + cut, b, width, color);
    draw_line(x + cut, b, x, b - cut, width, color);
    draw_line(x, b - cut, x, y + cut, width, color);
    draw_line(x, y + cut, x + cut, y, width, color);
}

fn draw_wheat(center: Vec2, size: f32, color: Color) {
    let stem_bottom = vec2(center.x - size * 0.10, center.y + size * 0.42);
    let stem_top = vec2(center.x + size * 0.13, center.y - size * 0.44);
    draw_line(
        stem_bottom.x,
        stem_bottom.y,
        stem_top.x,
        stem_top.y,
        2.0,
        color,
    );
    for i in 0..4 {
        let t = i as f32 / 4.0;
        let y = center.y + size * (0.20 - t * 0.55);
        let x = center.x + size * (-0.03 + t * 0.12);
        draw_line(x, y, x - size * 0.18, y - size * 0.10, 1.6, color);
        draw_line(x, y, x + size * 0.18, y - size * 0.13, 1.6, color);
    }
}

fn draw_tree(center: Vec2, size: f32, color: Color) {
    draw_triangle(
        vec2(center.x, center.y - size * 0.46),
        vec2(center.x - size * 0.34, center.y + size * 0.10),
        vec2(center.x + size * 0.34, center.y + size * 0.10),
        color,
    );
    draw_triangle(
        vec2(center.x, center.y - size * 0.20),
        vec2(center.x - size * 0.42, center.y + size * 0.30),
        vec2(center.x + size * 0.42, center.y + size * 0.30),
        color,
    );
    draw_rectangle(
        center.x - size * 0.07,
        center.y + size * 0.20,
        size * 0.14,
        size * 0.24,
        color,
    );
}

fn draw_stone(center: Vec2, size: f32, color: Color) {
    let points = [
        vec2(center.x - size * 0.34, center.y - size * 0.08),
        vec2(center.x - size * 0.08, center.y - size * 0.34),
        vec2(center.x + size * 0.32, center.y - size * 0.22),
        vec2(center.x + size * 0.38, center.y + size * 0.16),
        vec2(center.x + size * 0.06, center.y + size * 0.36),
        vec2(center.x - size * 0.30, center.y + size * 0.22),
    ];
    for i in 1..points.len() - 1 {
        draw_triangle(points[0], points[i], points[i + 1], color);
    }
    draw_line(
        points[0].x,
        points[0].y,
        points[2].x,
        points[2].y,
        1.0,
        TEXT_DIM,
    );
}

fn draw_coin(center: Vec2, size: f32, color: Color) {
    draw_circle(center.x, center.y, size * 0.35, color);
    draw_circle_lines(center.x, center.y, size * 0.35, 1.0, TEXT_BRIGHT);
    draw_circle_lines(
        center.x,
        center.y,
        size * 0.22,
        1.0,
        Color::new(0.50, 0.34, 0.12, 0.65),
    );
}

fn draw_people(center: Vec2, size: f32, color: Color) {
    for offset in [-0.24, 0.0, 0.24] {
        draw_circle(
            center.x + size * offset,
            center.y - size * 0.18,
            size * 0.12,
            color,
        );
        draw_rectangle(
            center.x + size * offset - size * 0.10,
            center.y - size * 0.04,
            size * 0.20,
            size * 0.30,
            Color::new(color.r, color.g, color.b, 0.72),
        );
    }
}

fn draw_compass(center: Vec2, size: f32, color: Color) {
    let long = size * 0.44;
    let short = size * 0.18;
    draw_triangle(
        vec2(center.x, center.y - long),
        vec2(center.x - short, center.y),
        vec2(center.x + short, center.y),
        color,
    );
    draw_triangle(
        vec2(center.x, center.y + long),
        vec2(center.x - short, center.y),
        vec2(center.x + short, center.y),
        Color::new(color.r, color.g, color.b, 0.45),
    );
    draw_line(
        center.x - long,
        center.y,
        center.x + long,
        center.y,
        1.2,
        color,
    );
    draw_circle_lines(
        center.x,
        center.y,
        size * 0.32,
        1.0,
        Color::new(color.r, color.g, color.b, 0.35),
    );
}

fn draw_crown(center: Vec2, size: f32, color: Color) {
    let y = center.y + size * 0.18;
    draw_rectangle(center.x - size * 0.32, y, size * 0.64, size * 0.16, color);
    draw_triangle(
        vec2(center.x - size * 0.32, y),
        vec2(center.x - size * 0.22, center.y - size * 0.28),
        vec2(center.x - size * 0.06, y),
        color,
    );
    draw_triangle(
        vec2(center.x - size * 0.10, y),
        vec2(center.x, center.y - size * 0.42),
        vec2(center.x + size * 0.10, y),
        color,
    );
    draw_triangle(
        vec2(center.x + size * 0.06, y),
        vec2(center.x + size * 0.22, center.y - size * 0.28),
        vec2(center.x + size * 0.32, y),
        color,
    );
}

fn draw_castle(center: Vec2, size: f32, color: Color) {
    let w = size * 0.50;
    let h = size * 0.44;
    draw_rectangle(center.x - w * 0.5, center.y - h * 0.2, w, h, color);
    draw_rectangle(
        center.x - w * 0.62,
        center.y - h * 0.44,
        w * 0.24,
        h * 0.70,
        color,
    );
    draw_rectangle(
        center.x + w * 0.38,
        center.y - h * 0.44,
        w * 0.24,
        h * 0.70,
        color,
    );
    draw_rectangle(
        center.x - w * 0.12,
        center.y + h * 0.10,
        w * 0.24,
        h * 0.32,
        INK,
    );
}

fn draw_crest_tree(center: Vec2, size: f32, color: Color) {
    draw_line(
        center.x,
        center.y + size * 0.35,
        center.x,
        center.y - size * 0.34,
        1.8,
        color,
    );
    for branch in [-0.26, -0.08, 0.10] {
        let y = center.y + size * branch;
        draw_line(
            center.x,
            y,
            center.x - size * 0.26,
            y - size * 0.14,
            1.4,
            color,
        );
        draw_line(
            center.x,
            y,
            center.x + size * 0.26,
            y - size * 0.14,
            1.4,
            color,
        );
    }
    draw_circle_lines(center.x, center.y - size * 0.12, size * 0.26, 1.1, color);
}

fn draw_road(center: Vec2, size: f32, color: Color) {
    draw_line(
        center.x - size * 0.36,
        center.y + size * 0.32,
        center.x - size * 0.10,
        center.y - size * 0.28,
        2.0,
        color,
    );
    draw_line(
        center.x + size * 0.36,
        center.y + size * 0.32,
        center.x + size * 0.10,
        center.y - size * 0.28,
        2.0,
        color,
    );
    draw_line(
        center.x - size * 0.04,
        center.y,
        center.x + size * 0.04,
        center.y,
        1.2,
        color,
    );
}

fn draw_skull(center: Vec2, size: f32, color: Color) {
    draw_circle(center.x, center.y - size * 0.08, size * 0.28, color);
    draw_rectangle(
        center.x - size * 0.18,
        center.y + size * 0.10,
        size * 0.36,
        size * 0.20,
        color,
    );
    draw_circle(
        center.x - size * 0.10,
        center.y - size * 0.08,
        size * 0.06,
        INK,
    );
    draw_circle(
        center.x + size * 0.10,
        center.y - size * 0.08,
        size * 0.06,
        INK,
    );
}
