//! Supply state and urgent route warnings in the selected-site inspector.

use super::{style, virtual_button, FrontierDetailsTab, UiAction, UiContext};
use crate::state::RouteCondition;
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;

pub(super) fn draw_route_section(
    ctx: &UiContext<'_>,
    input_enabled: bool,
    actions: &mut Vec<UiAction>,
    content: Rect,
    y: f32,
) {
    let Some(site) = ctx.session.selected_site(ctx.data) else {
        return;
    };
    if !ctx.session.is_known(&site.id) {
        return;
    }

    let mut damaged = 0;
    let mut blocked = 0;
    for route in ctx
        .session
        .routes_for_site(&site.id)
        .filter(|route| route.known)
    {
        match route.condition {
            RouteCondition::Damaged => damaged += 1,
            RouteCondition::Blocked => blocked += 1,
            RouteCondition::Clear => {}
        }
    }
    let warning = damaged + blocked > 0;
    let label = if warning {
        ctx.data.text_with(
            "ui.routes_warning",
            &[
                ("{damaged}", &damaged.to_string()),
                ("{blocked}", &blocked.to_string()),
            ],
        )
    } else {
        ctx.data.text("ui.routes_supply")
    };
    style::draw_divider(content.x, y - 6.0, content.w);
    if virtual_button(
        Rect::new(content.x, y, content.w, 28.0),
        &label,
        input_enabled,
        if warning {
            ButtonTone::Danger
        } else {
            ButtonTone::Secondary
        },
        ctx.pointer,
    ) {
        actions.push(UiAction::SetFrontierDetailsTab(FrontierDetailsTab::Routes));
        actions.push(UiAction::ToggleFrontierDetails);
    }
    draw_text_block(
        &ctx.session.settlement_supply_summary(ctx.data, &site.id),
        content.x,
        y + 31.0,
        content.w,
        28.0,
        12.0,
        2.0,
        style::TEXT_DIM,
    );
}
