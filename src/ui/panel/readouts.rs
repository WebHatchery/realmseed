//! Settlement readout widgets: metric grid, store grid, and focus card.

use crate::data::GameData;
use crate::state::SettlementRuntimeState;
use crate::ui::{style, UiContext};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub(super) fn draw_metric_grid(
    _data: &GameData,
    x: f32,
    y: f32,
    width: f32,
    metrics: [(String, i32); 4],
) {
    let cell_w = width / metrics.len() as f32;
    for (index, (label, value)) in metrics.iter().enumerate() {
        let rect = Rect::new(x + index as f32 * cell_w, y, cell_w, 44.0);
        draw_surface(
            rect,
            &SurfaceStyle::new(Color::new(0.020, 0.034, 0.036, 0.76))
                .with_border(1.0, Color::new(0.58, 0.45, 0.25, 0.22)),
        );
        draw_text_centered_in_box_ex(
            label,
            rect.x + 4.0,
            rect.y + 4.0,
            rect.w - 8.0,
            16.0,
            TextStyle::new(10.5, style::TEXT_DIM),
        );
        let value_color = if index == 3 {
            style::RED
        } else {
            style::TEXT_BRIGHT
        };
        draw_text_centered_in_box_ex(
            &value.to_string(),
            rect.x + 4.0,
            rect.y + 23.0,
            rect.w - 8.0,
            20.0,
            TextStyle::new(16.0, value_color),
        );
    }
}

pub(super) fn draw_store_grid(data: &GameData, settlement: &SettlementRuntimeState, rect: Rect) {
    let values = [
        (
            data.text("ui.food"),
            settlement.stored.food,
            style::IconKind::Food,
            Color::new(0.93, 0.66, 0.22, 1.0),
        ),
        (
            data.text("ui.timber"),
            settlement.stored.timber,
            style::IconKind::Timber,
            Color::new(0.36, 0.65, 0.25, 1.0),
        ),
        (
            data.text("ui.stone"),
            settlement.stored.stone,
            style::IconKind::Stone,
            Color::new(0.62, 0.58, 0.50, 1.0),
        ),
        (
            data.text("ui.wealth"),
            settlement.stored.wealth,
            style::IconKind::Wealth,
            Color::new(0.93, 0.70, 0.26, 1.0),
        ),
    ];
    let cell_w = rect.w / values.len() as f32;
    for (index, (label, amount, icon, color)) in values.iter().enumerate() {
        let x = rect.x + index as f32 * cell_w;
        style::draw_icon(*icon, vec2(x + 12.0, rect.y + 17.0), 22.0, *color);
        draw_ui_text_ex(
            label,
            x + 26.0,
            rect.y + 11.0,
            TextStyle::new(11.0, style::TEXT_DIM).params(),
        );
        draw_ui_text_ex(
            &amount.to_string(),
            x + 26.0,
            rect.y + 28.0,
            TextStyle::new(15.0, style::TEXT_BRIGHT).params(),
        );
        if index + 1 < values.len() {
            style::draw_vertical_divider(x + cell_w - 6.0, rect.y + 2.0, rect.h - 4.0);
        }
    }
}

pub(super) fn draw_focus_card(
    ctx: &UiContext<'_>,
    settlement: &SettlementRuntimeState,
    rect: Rect,
) {
    draw_surface(
        rect,
        &SurfaceStyle::new(Color::new(0.022, 0.041, 0.045, 0.74))
            .with_border(1.0, Color::new(0.58, 0.45, 0.25, 0.22)),
    );
    let focus_name = settlement
        .focus(ctx.data)
        .map(|focus| focus.name.clone())
        .unwrap_or_else(|| ctx.data.text("ui.unassigned"));
    style::draw_icon(
        focus_icon(&focus_name),
        vec2(rect.x + 24.0, rect.y + 22.0),
        30.0,
        style::GOLD,
    );
    draw_ui_text_ex(
        &focus_name,
        rect.x + 54.0,
        rect.y + 17.0,
        TextStyle::new(15.0, style::TEXT_BRIGHT).params(),
    );
    draw_text_block(
        &focus_summary(ctx, settlement),
        rect.x + 54.0,
        rect.y + 21.0,
        rect.w - 62.0,
        17.0,
        10.0,
        1.0,
        style::TEXT_DIM,
    );
}

pub(super) fn focus_icon(name: &str) -> style::IconKind {
    match name {
        "Farming" => style::IconKind::Food,
        "Logging" => style::IconKind::Timber,
        "Quarrying" => style::IconKind::Stone,
        "Trade" => style::IconKind::Wealth,
        "Fortify" => style::IconKind::Castle,
        _ => style::IconKind::Crown,
    }
}

fn focus_summary(ctx: &UiContext<'_>, settlement: &SettlementRuntimeState) -> String {
    let Some(focus) = settlement.focus(ctx.data) else {
        return ctx.data.text("ui.no_focus");
    };
    let output = resource_output_text(ctx.data, focus.output);
    ctx.data.text_with(
        "ui.focus_summary",
        &[
            ("{name}", &focus.name),
            ("{output}", &output),
            ("{notes}", &focus.notes),
        ],
    )
}

fn resource_output_text(data: &GameData, output: crate::data::ResourceStock) -> String {
    let mut parts = Vec::new();
    if output.food != 0 {
        parts.push(output_amount(data, output.food, "ui.food"));
    }
    if output.timber != 0 {
        parts.push(output_amount(data, output.timber, "ui.timber"));
    }
    if output.stone != 0 {
        parts.push(output_amount(data, output.stone, "ui.stone"));
    }
    if output.wealth != 0 {
        parts.push(output_amount(data, output.wealth, "ui.wealth"));
    }
    if parts.is_empty() {
        data.text("ui.steady_holdings")
    } else {
        parts.join(", ")
    }
}

fn output_amount(data: &GameData, amount: i32, resource_id: &str) -> String {
    let amount = format!("{amount:+}");
    let resource = data.text(resource_id);
    data.text_with(
        "ui.output_resource",
        &[("{amount}", &amount), ("{resource}", &resource)],
    )
}
