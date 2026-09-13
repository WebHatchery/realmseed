//! Selected-site panel, campaign controls, and chronicle overlay.

mod chronicle;
mod readouts;
mod settlement;
mod site;

pub(super) use chronicle::draw_chronicle_overlay;

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
    style::draw_panel_title("SELECTED SITE", content.x, content.y + 16.0);
    let mut y = content.y + 50.0;
    y = site::draw_selected_site(ctx, content, y);
    y = draw_settlement_section(ctx, tooltip, input_enabled, actions, content, y + 2.0);
    routes::draw_route_section(
        ctx,
        tooltip,
        ctx.pointer,
        input_enabled,
        actions,
        content,
        y + 4.0,
    );
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
        super::section_label("SCOUTING", content.x, y);
        return site::draw_scout_action(
            ctx,
            ctx.pointer,
            input_enabled,
            actions,
            content,
            y + 14.0,
        );
    }

    if let Some(settlement) = ctx.session.settlement_at_site(&site.id) {
        settlement::draw_existing_settlement(
            ctx,
            tooltip,
            ctx.pointer,
            input_enabled,
            actions,
            content,
            y,
            settlement,
        )
    } else if site.category == SiteCategory::Independent {
        super::section_label("SITE DECISIONS", content.x, y);
        site::draw_independent_actions(ctx, ctx.pointer, input_enabled, actions, content, y + 14.0)
    } else if site.category == SiteCategory::Settlement {
        super::section_label("SITE DECISIONS", content.x, y);
        site::draw_found_camp_action(ctx, ctx.pointer, input_enabled, actions, content, y + 14.0)
    } else {
        y
    }
}
