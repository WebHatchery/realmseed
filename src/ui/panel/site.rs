//! Selected-site summary and one-off site decisions (scout, trade, found).

use crate::data::SiteCategory;
use crate::state::{SettlementStatus, SiteKnowledge};
use crate::ui::{
    inspectable_button, style, virtual_icon_button, ActionReview, UiAction, UiContext,
};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub(super) fn draw_selected_site(ctx: &UiContext<'_>, content: Rect, y: f32) -> f32 {
    let Some(site) = ctx.session.selected_site(ctx.data) else {
        draw_ui_text_ex(
            &ctx.data.text("ui.no_site"),
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
        .map(|region| region.name.clone())
        .unwrap_or_else(|| ctx.data.text("ui.unknown_region"));

    let title = if knowledge == SiteKnowledge::Known {
        site.name.clone()
    } else {
        ctx.data.text("ui.rumored_site")
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
        &title,
        content.x + 58.0,
        y + 1.0,
        content.w - 150.0,
        34.0,
        22.0,
        2.0,
        style::TEXT_BRIGHT,
    );

    let settlement = ctx.session.settlement_at_site(&site.id);
    let active = knowledge == SiteKnowledge::Known
        && settlement.is_some_and(|settlement| settlement.is_active());
    let status = if knowledge == SiteKnowledge::Known {
        settlement
            .map(|settlement| settlement_status_label(ctx, settlement.status))
            .unwrap_or_else(|| ctx.data.text("ui.known"))
    } else {
        ctx.data.text("ui.rumor_status")
    };
    draw_badge(
        Rect::new(content.right() - 86.0, y, 86.0, 27.0),
        &status,
        if active {
            Color::new(0.10, 0.25, 0.09, 0.95)
        } else {
            Color::new(0.15, 0.16, 0.12, 0.95)
        },
        if active { style::GREEN } else { style::TEXT },
    );
    let site_subtitle = ctx
        .session
        .settlement_at_site(&site.id)
        .map(|settlement| {
            let tier = settlement_tier_label(ctx, settlement.tier);
            let status = settlement_status_label(ctx, settlement.status);
            ctx.data.text_with(
                "ui.site_subtitle",
                &[("{tier}", &tier), ("{status}", &status)],
            )
        })
        .unwrap_or_else(|| site.category.label().to_owned());
    draw_ui_text_ex(
        &site_subtitle,
        content.x + 58.0,
        y + 43.0,
        TextStyle::new(13.0, style::GOLD).params(),
    );

    if settlement.is_some() {
        return y + if ctx.ui.logical_width < 1040.0 {
            48.0
        } else {
            58.0
        };
    }

    let known_text = if knowledge == SiteKnowledge::Known {
        let owner = site
            .owner
            .clone()
            .unwrap_or_else(|| ctx.data.text("ui.unclaimed"));
        let site_type = site.type_label();
        let traits = site.traits.join(", ");
        ctx.data.text_with(
            "ui.site_detail",
            &[
                ("{region}", &region_name),
                ("{type}", &site_type),
                ("{owner}", &owner),
                ("{traits}", &traits),
                ("{description}", &site.description),
            ],
        )
    } else {
        let scout_status = ctx.session.scout_status(ctx.data);
        let known_status = match knowledge {
            SiteKnowledge::Known => ctx.data.text("ui.known"),
            SiteKnowledge::Unknown => ctx.data.text("ui.unknown"),
        };
        ctx.data.text_with(
            "ui.site_rumor_detail",
            &[
                ("{region}", &region_name),
                ("{status}", &known_status),
                ("{reason}", &scout_status.reason),
            ],
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

fn settlement_tier_label(ctx: &UiContext<'_>, tier: crate::data::SettlementTier) -> String {
    let text_id = match tier {
        crate::data::SettlementTier::Camp => "ui.tier_camp",
        crate::data::SettlementTier::Village => "ui.tier_village",
        crate::data::SettlementTier::Town => "ui.tier_town",
        crate::data::SettlementTier::City => "ui.tier_city",
    };
    ctx.data.text(text_id)
}

fn settlement_status_label(ctx: &UiContext<'_>, status: SettlementStatus) -> String {
    let text_id = match status {
        SettlementStatus::Active => "ui.status_active",
        SettlementStatus::Lost => "ui.status_lost",
    };
    ctx.data.text(text_id)
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
        &ctx.data.text("ui.scout_site"),
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
    let trust = independent.trust.to_string();
    let autonomy = independent.autonomy.to_string();
    let rival = independent.rival_pressure.to_string();
    let state = independent.integration_state.label().to_owned();
    let stats = ctx.data.text_with(
        "ui.independent_stats",
        &[
            ("{trust}", &trust),
            ("{autonomy}", &autonomy),
            ("{rival}", &rival),
            ("{state}", &state),
        ],
    );
    draw_ui_text_ex(
        &stats,
        content.x,
        y,
        TextStyle::new(14.0, dark::TEXT).params(),
    );
    let trade_status = ctx.session.independent_trade_status(ctx.data);
    let integration_status = ctx.session.integration_status(ctx.data);
    let half = (content.w - 8.0) / 2.0;
    if inspectable_button(
        Rect::new(content.x, y + 20.0, half, 30.0),
        &ctx.data.text("ui.open_trade"),
        trade_status.enabled,
        input_enabled,
        ButtonTone::Primary,
        pointer,
    ) {
        actions.push(UiAction::OpenActionReview(
            ActionReview::OpenIndependentTrade,
        ));
    }
    if inspectable_button(
        Rect::new(content.x + half + 8.0, y + 20.0, half, 30.0),
        &ctx.data.text("ui.integrate"),
        integration_status.enabled,
        input_enabled,
        ButtonTone::Positive,
        pointer,
    ) {
        actions.push(UiAction::OpenActionReview(
            ActionReview::BeginIndependentIntegration,
        ));
    }
    draw_text_block(
        &ctx.data
            .text_with("ui.need", &[("{need}", &independent.local_need)]),
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
    if inspectable_button(
        Rect::new(content.x, y, content.w, 34.0),
        &ctx.data.text("ui.found_camp"),
        status.enabled,
        input_enabled,
        ButtonTone::Positive,
        pointer,
    ) {
        actions.push(UiAction::OpenActionReview(ActionReview::FoundCamp));
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
