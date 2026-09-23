//! Touch-accessible comparison of settlement focuses.

use super::readouts::{focus_icon, resource_output_text};
use crate::ui::{style, virtual_button, virtual_icon_button, UiAction, UiContext};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::RectExt;

pub(crate) fn draw_focus_comparison(ctx: &UiContext<'_>, actions: &mut Vec<UiAction>) {
    let Some(settlement) = ctx.session.selected_settlement() else {
        return;
    };
    draw_rectangle(
        0.0,
        0.0,
        ctx.ui.logical_width,
        ctx.ui.logical_height,
        Color::new(0.004, 0.010, 0.012, 0.74),
    );
    let rect = crate::ui::centered_modal_rect(ctx, 850.0, 510.0);
    style::draw_panel(rect);
    let content = rect.inset(24.0);
    style::draw_panel_title(
        &ctx.data.text("ui.change_focus"),
        content.x,
        content.y + 22.0,
    );
    if virtual_button(
        Rect::new(content.right() - 86.0, content.y, 86.0, 32.0),
        &ctx.data.text("ui.close"),
        true,
        ButtonTone::Secondary,
        ctx.pointer,
    ) {
        actions.push(UiAction::ToggleFocusChoices);
    }
    let gap = 18.0;
    let width = (content.w - gap) / 2.0;
    let height = (content.h - 58.0) / 3.0;
    for (index, focus) in ctx.data.settlement_balance.focuses.iter().enumerate() {
        let x = content.x + (index % 2) as f32 * (width + gap);
        let y = content.y + 52.0 + (index / 2) as f32 * height;
        let status = ctx.session.focus_change_status(ctx.data, &focus.id);
        let current = focus.id == settlement.focus_id;
        let label = if current {
            ctx.data
                .text_with("ui.focus_selected", &[("{name}", &focus.name)])
        } else {
            focus.name.clone()
        };
        if virtual_icon_button(
            Rect::new(x, y, width, 32.0),
            &label,
            focus_icon(&focus.name),
            status.enabled,
            if current {
                ButtonTone::Primary
            } else {
                ButtonTone::Secondary
            },
            ctx.pointer,
        ) {
            actions.push(UiAction::SetSettlementFocus(focus.id.clone()));
            actions.push(UiAction::ToggleFocusChoices);
        }
        let detail = if status.enabled {
            status.reason
        } else {
            format!("{}\n{}", focus.notes, status.reason)
        };
        let text = ctx.data.text_with(
            "ui.focus_comparison",
            &[
                ("{output}", &resource_output_text(ctx.data, focus.output)),
                ("{detail}", &detail),
            ],
        );
        draw_text_block(
            &text,
            x,
            y + 39.0,
            width,
            height - 47.0,
            13.0,
            2.0,
            style::TEXT,
        );
    }
}
