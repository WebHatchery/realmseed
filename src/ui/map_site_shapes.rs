//! Small, reusable site marker details for the strategic map.

use super::map::MapView;
use super::{style, UiContext};
use crate::data::SiteDef;
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub(super) fn draw_unknown_site(ctx: &UiContext<'_>, view: &MapView, site: &SiteDef) {
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
            &ctx.data.text("ui.scout"),
            position.x + radius + 5.0,
            position.y - radius - 2.0,
            TextStyle::new(12.0, style::TEXT_BRIGHT).params(),
        );
    }
}

pub(super) fn draw_site_label(site: &SiteDef, position: Vec2, radius: f32, selected: bool) {
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
