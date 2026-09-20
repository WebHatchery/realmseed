//! Shared confirmation surface for consequential council actions.

use super::{style, ActionReview, UiAction, UiContext};
use crate::state::SettlementActionStatus;
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;
use macroquad_toolkit::ui::RectExt;

pub(super) fn draw_action_review(ctx: &UiContext<'_>, actions: &mut Vec<UiAction>) {
    let Some(review) = ctx.action_review else {
        return;
    };
    let status = review.status(ctx);
    let screen = super::screen_rect(ctx);
    draw_rectangle(
        screen.x,
        screen.y,
        screen.w,
        screen.h,
        Color::new(0.004, 0.010, 0.012, 0.76),
    );

    let rect = super::centered_modal_rect(ctx, 700.0, 330.0);
    style::draw_panel(rect);
    let content = rect.inset(24.0);
    style::draw_panel_title(
        &ctx.data.text("ui.action_review_title"),
        content.x,
        content.y + 16.0,
    );
    draw_ui_text_ex(
        &review.label(ctx),
        content.x,
        content.y + 53.0,
        TextStyle::new(25.0, style::TEXT_BRIGHT).params(),
    );
    style::draw_divider(content.x, content.y + 76.0, content.w);

    let status_text = if status.enabled {
        ctx.data.text("ui.action_review_ready")
    } else {
        ctx.data
            .text_with("ui.action_review_blocked", &[("{reason}", &status.reason)])
    };
    draw_ui_text_ex(
        &status_text,
        content.x,
        content.y + 108.0,
        TextStyle::new(
            16.0,
            if status.enabled {
                style::GREEN
            } else {
                style::RED
            },
        )
        .params(),
    );
    draw_text_block(
        &status.reason,
        content.x,
        content.y + 130.0,
        content.w,
        64.0,
        16.0,
        3.0,
        style::TEXT,
    );
    draw_text_block(
        &ctx.data.text("ui.action_review_hint"),
        content.x,
        content.y + 202.0,
        content.w,
        34.0,
        13.0,
        2.0,
        style::TEXT_DIM,
    );

    let button_y = content.bottom() - 34.0;
    if super::virtual_button(
        Rect::new(content.right() - 126.0, button_y, 126.0, 32.0),
        &ctx.data.text("ui.cancel"),
        true,
        ButtonTone::Secondary,
        ctx.pointer,
    ) {
        actions.push(UiAction::CancelActionReview);
    }
    if super::virtual_button(
        Rect::new(content.right() - 310.0, button_y, 172.0, 32.0),
        &ctx.data.text("ui.confirm"),
        status.enabled,
        ButtonTone::Primary,
        ctx.pointer,
    ) {
        actions.push(UiAction::ConfirmActionReview);
    }
}

impl ActionReview {
    fn label(&self, ctx: &UiContext<'_>) -> String {
        match self {
            Self::FoundCamp => ctx.data.text("ui.found_camp"),
            Self::UpgradeSettlement => ctx.session.selected_upgrade_label(ctx.data),
            Self::BuildOrUpgradeRoute(route_id) => {
                ctx.session.route_action_label(ctx.data, route_id)
            }
            Self::CompleteRegionalProject(_) => ctx.data.text("ui.wardens"),
            Self::ResolveEventChoice(choice_id) => ctx
                .session
                .pending_event_template(ctx.data)
                .and_then(|template| {
                    template
                        .choices
                        .iter()
                        .find(|choice| choice.id == *choice_id)
                })
                .map(|choice| choice.label.clone())
                .unwrap_or_else(|| ctx.data.text("ui.council_event")),
            Self::OpenIndependentTrade => ctx.data.text("ui.open_trade"),
            Self::BeginIndependentIntegration => ctx.data.text("ui.integrate"),
            Self::CompleteProject(project_id) => ctx
                .data
                .campaign_balance
                .project(project_id)
                .map(|project| project.name.clone())
                .unwrap_or_else(|| ctx.data.text("state.unknown_project")),
            Self::ActivateInstitution(institution_id) => ctx
                .data
                .campaign_balance
                .institution(institution_id)
                .map(|institution| institution.name.clone())
                .unwrap_or_else(|| ctx.data.text("state.unknown_institution")),
        }
    }

    fn status(&self, ctx: &UiContext<'_>) -> SettlementActionStatus {
        match self {
            Self::FoundCamp => ctx.session.founding_status(ctx.data),
            Self::UpgradeSettlement => ctx.session.upgrade_status(ctx.data),
            Self::BuildOrUpgradeRoute(route_id) => {
                ctx.session.route_action_status(ctx.data, route_id)
            }
            Self::CompleteRegionalProject(region_id) => {
                ctx.session.regional_project_status(ctx.data, region_id)
            }
            Self::ResolveEventChoice(choice_id) => {
                let Some(template) = ctx.session.pending_event_template(ctx.data) else {
                    return SettlementActionStatus::disabled(ctx.data.text("event.no_pending"));
                };
                let Some(choice) = template
                    .choices
                    .iter()
                    .find(|choice| choice.id == *choice_id)
                else {
                    return SettlementActionStatus::disabled(ctx.data.text("event.choice_unknown"));
                };
                ctx.session.event_choice_status(ctx.data, choice)
            }
            Self::OpenIndependentTrade => ctx.session.independent_trade_status(ctx.data),
            Self::BeginIndependentIntegration => ctx.session.integration_status(ctx.data),
            Self::CompleteProject(project_id) => ctx.session.project_status(ctx.data, project_id),
            Self::ActivateInstitution(institution_id) => {
                ctx.session.institution_status(ctx.data, institution_id)
            }
        }
    }

    pub(crate) fn gameplay_action(&self) -> UiAction {
        match self {
            Self::FoundCamp => UiAction::FoundCamp,
            Self::UpgradeSettlement => UiAction::UpgradeSelectedSettlement,
            Self::BuildOrUpgradeRoute(route_id) => UiAction::BuildOrUpgradeRoute(route_id.clone()),
            Self::CompleteRegionalProject(region_id) => {
                UiAction::CompleteRegionalProject(region_id.clone())
            }
            Self::ResolveEventChoice(choice_id) => UiAction::ResolveEventChoice(choice_id.clone()),
            Self::OpenIndependentTrade => UiAction::OpenIndependentTrade,
            Self::BeginIndependentIntegration => UiAction::BeginIndependentIntegration,
            Self::CompleteProject(project_id) => UiAction::CompleteProject(project_id.clone()),
            Self::ActivateInstitution(institution_id) => {
                UiAction::ActivateInstitution(institution_id.clone())
            }
        }
    }
}
