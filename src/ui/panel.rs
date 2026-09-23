//! Selected-site panel, campaign controls, and chronicle overlay.

mod chronicle;
mod focus;
mod readouts;
mod settlement;
mod site;

pub(super) use chronicle::draw_chronicle_overlay;
pub(super) use focus::draw_focus_comparison;

use super::{routes, style, UiAction, UiContext};
use crate::data::SiteCategory;
use crate::state::SiteKnowledge;
use macroquad::prelude::*;
use macroquad_toolkit::ui::HoverTooltip;
use macroquad_toolkit::ui::RectExt;

pub(super) fn draw_side_panel(
    ctx: &UiContext<'_>,
    tooltip: &mut HoverTooltip,
    input_enabled: bool,
    actions: &mut Vec<UiAction>,
) {
    let rect = super::side_panel_rect(ctx);
    style::draw_panel(rect);

    let content = rect.inset(18.0);
    let mut y = content.y + 2.0;
    y = site::draw_selected_site(ctx, content, y);
    y = draw_settlement_section(ctx, tooltip, input_enabled, actions, content, y + 2.0);
    routes::draw_route_section(ctx, input_enabled, actions, content, y + 4.0);
}

fn draw_settlement_section(
    ctx: &UiContext<'_>,
    tooltip: &mut HoverTooltip,
    input_enabled: bool,
    actions: &mut Vec<UiAction>,
    content: Rect,
    y: f32,
) -> f32 {
    let Some(site) = ctx.session.selected_site(ctx.data) else {
        return y;
    };
    if ctx.session.site_knowledge(&site.id) != SiteKnowledge::Known {
        return site::draw_scout_action(ctx, ctx.pointer, input_enabled, actions, content, y);
    }

    if let Some(settlement) = ctx.session.settlement_at_site(&site.id) {
        let mut interaction = settlement::SettlementInteraction {
            tooltip,
            pointer: ctx.pointer,
            input_enabled,
            actions,
        };
        settlement::draw_existing_settlement(ctx, &mut interaction, content, y, settlement)
    } else if site.category == SiteCategory::Independent {
        site::draw_independent_actions(ctx, ctx.pointer, input_enabled, actions, content, y)
    } else if site.category == SiteCategory::Settlement {
        site::draw_found_camp_action(ctx, ctx.pointer, input_enabled, actions, content, y)
    } else {
        y
    }
}
