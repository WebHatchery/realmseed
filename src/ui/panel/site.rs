//! Selected-site summary and one-off site decisions (scout, trade, found).

use crate::data::SiteCategory;
use crate::state::SiteKnowledge;
use crate::ui::{style, virtual_icon_button, UiAction, UiContext};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub(super) fn draw_selected_site(ctx: &UiContext<'_>, content: Rect, y: f32) -> f32 {
    let Some(site) = ctx.session.selected_site(ctx.data) else {
        draw_ui_text_ex(
            "No site selected",
            content.x,
            y + 24.0,
            TextStyle::new(22.0, dark::TEXT_BRIGHT).params(),
        );
        return y + 70.0;
    };

    let knowledge = ctx.session.site_knowledge(&site.id);
    let region_name = ctx
        .data
        .region(&site.region_id)
        .map(|region| region.name.as_str())
        .unwrap_or("Unknown region");

    let title = if knowledge == SiteKnowledge::Known {
        site.name.as_str()
    } else {
        "Rumored Site"
    };
    let icon = if ctx.session.settlement_at_site(&site.id).is_some() {
        style::IconKind::Castle
    } else if site.category == SiteCategory::Independent {
        style::IconKind::Road
    } else {
        style::IconKind::Tree
    };
    style::draw_framed_icon(
        icon,
        vec2(content.x + 26.0, y + 20.0),
        50.0,
        Color::new(0.18, 0.13, 0.07, 0.95),
    );
    draw_text_block(
        title,
        content.x + 58.0,
        y + 1.0,
        content.w - 150.0,
        34.0,
        22.0,
        2.0,
        style::TEXT_BRIGHT,
    );

    let status = if knowledge == SiteKnowledge::Known {
        if ctx.session.settlement_at_site(&site.id).is_some() {
            "Active"
        } else {
            "Known"
        }
    } else {
        "Rumor"
    };
    draw_badge(
        Rect::new(content.right() - 86.0, y, 86.0, 27.0),
        status,
        if status == "Active" {
            Color::new(0.10, 0.25, 0.09, 0.95)
        } else {
            Color::new(0.15, 0.16, 0.12, 0.95)
        },
        if status == "Active" {
            style::GREEN
        } else {
            style::TEXT
        },
    );
    let site_subtitle = ctx
        .session
        .settlement_at_site(&site.id)
        .map(|settlement| {
            format!(
                "{} / {}",
                settlement.tier.label(),
                settlement.status.label()
            )
        })
        .unwrap_or_else(|| site.category.label().to_owned());
    draw_ui_text_ex(
        &site_subtitle,
        content.x + 58.0,
        y + 43.0,
        TextStyle::new(13.0, style::GOLD).params(),
    );

    if ctx.session.settlement_at_site(&site.id).is_some() {
        return y + 58.0;
    }

    let known_text = if knowledge == SiteKnowledge::Known {
        format!(
            "Region: {}\nType: {}\nOwner: {}\nTraits: {}\n{}",
            region_name,
            site.type_label(),
            site.owner.as_deref().unwrap_or("Unclaimed"),
            site.traits.join(", "),
            site.description
        )
    } else {
        let scout_status = ctx.session.scout_status(ctx.data);
        format!(
            "Region: {}\nKnown status: {}\n{}",
            region_name,
            knowledge.label(),
            scout_status.reason
        )
    };
    draw_text_block(
        &known_text,
        content.x,
        y + 70.0,
        content.w,
        54.0,
        15.0,
        3.0,
        style::TEXT_DIM,
    );

    y + 130.0
}

pub(super) fn draw_scout_action(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    input_enabled: bool,
    actions: &mut Vec<UiAction>,
    content: Rect,
    y: f32,
) -> f32 {
    let status = ctx.session.scout_status(ctx.data);
    if virtual_icon_button(
        Rect::new(content.x, y, content.w, 34.0),
        "Scout Site",
        style::IconKind::Compass,
        input_enabled && status.enabled,
        ButtonTone::Primary,
        pointer,
    ) {
        actions.push(UiAction::ScoutSelectedSite);
    }
    draw_text_block(
        &status.reason,
        content.x,
        y + 40.0,
        content.w,
        38.0,
        14.0,
        3.0,
        if status.enabled {
            dark::TEXT_DIM
        } else {
            dark::WARNING
        },
    );

    y + 84.0
}

pub(super) fn draw_independent_actions(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    input_enabled: bool,
    actions: &mut Vec<UiAction>,
    content: Rect,
    y: f32,
) -> f32 {
    let Some(independent) = ctx.session.selected_independent() else {
        return y;
    };
    draw_ui_text_ex(
        &format!(
            "Trust {}  Autonomy {}  Rival {}  {}",
            independent.trust,
            independent.autonomy,
            independent.rival_pressure,
            independent.integration_state.label()
        ),
        content.x,
        y,
        TextStyle::new(14.0, dark::TEXT).params(),
    );
    let trade_status = ctx.session.independent_trade_status(ctx.data);
    let integration_status = ctx.session.integration_status(ctx.data);
    let half = (content.w - 8.0) / 2.0;
    if virtual_icon_button(
        Rect::new(content.x, y + 20.0, half, 30.0),
        "Open Trade",
        style::IconKind::Wealth,
        input_enabled && trade_status.enabled,
        ButtonTone::Primary,
        pointer,
    ) {
        actions.push(UiAction::OpenIndependentTrade);
    }
    if virtual_icon_button(
        Rect::new(content.x + half + 8.0, y + 20.0, half, 30.0),
        "Integrate",
        style::IconKind::Crown,
        input_enabled && integration_status.enabled,
        ButtonTone::Positive,
        pointer,
    ) {
        actions.push(UiAction::BeginIndependentIntegration);
    }
    draw_text_block(
        &format!("Need: {}", independent.local_need),
        content.x,
        y + 56.0,
        content.w,
        26.0,
        13.0,
        2.0,
        dark::TEXT_DIM,
    );
    y + 88.0
}

pub(super) fn draw_found_camp_action(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    input_enabled: bool,
    actions: &mut Vec<UiAction>,
    content: Rect,
    y: f32,
) -> f32 {
    let status = ctx.session.founding_status(ctx.data);
    if virtual_icon_button(
        Rect::new(content.x, y, content.w, 34.0),
        "Found Camp",
        style::IconKind::Castle,
        input_enabled && status.enabled,
        ButtonTone::Positive,
        pointer,
    ) {
        actions.push(UiAction::FoundCamp);
    }
    draw_text_block(
        &status.reason,
        content.x,
        y + 40.0,
        content.w,
        38.0,
        14.0,
        3.0,
        if status.enabled {
            dark::TEXT_DIM
        } else {
            dark::WARNING
        },
    );

    y + 84.0
}
