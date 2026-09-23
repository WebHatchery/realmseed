//! Active-settlement management: metrics, focus selection, and upgrades.

use super::readouts::{draw_metric_grid, draw_store_grid};
use crate::data::ActiveIssueState;
use crate::state::SettlementRuntimeState;
use crate::ui::{
    inspectable_button, style, virtual_button, ActionReview, FrontierDetailsTab, UiAction,
    UiContext,
};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::HoverTooltip;

pub(super) struct SettlementInteraction<'a> {
    pub(super) tooltip: &'a mut HoverTooltip,
    pub(super) pointer: Pointer,
    pub(super) input_enabled: bool,
    pub(super) actions: &'a mut Vec<UiAction>,
}

pub(super) fn draw_existing_settlement(
    ctx: &UiContext<'_>,
    interaction: &mut SettlementInteraction<'_>,
    content: Rect,
    y: f32,
    settlement: &SettlementRuntimeState,
) -> f32 {
    draw_metric_grid(
        ctx.data,
        content.x,
        y,
        content.w,
        [
            (ctx.data.text("ui.population"), settlement.population),
            (ctx.data.text("ui.loyalty"), settlement.loyalty),
            (ctx.data.text("ui.stability"), settlement.stability),
            (ctx.data.text("ui.danger"), settlement.danger),
        ],
    );

    let active_issue_count = active_issue_count(ctx, settlement);
    let stores_label_y = y + 60.0;
    let stores_grid_y = y + 72.0;
    crate::ui::section_label(&ctx.data.text("ui.stores"), content.x, stores_label_y);
    if active_issue_count > 0 {
        let issues_rect = Rect::new(content.right() - 112.0, stores_label_y - 16.0, 112.0, 28.0);
        if virtual_button(
            issues_rect,
            &ctx.data.text_with(
                "ui.active_issues_count",
                &[("{count}", &active_issue_count.to_string())],
            ),
            interaction.input_enabled,
            ButtonTone::Danger,
            interaction.pointer,
        ) {
            interaction
                .actions
                .push(UiAction::SetFrontierDetailsTab(FrontierDetailsTab::Issues));
            interaction.actions.push(UiAction::ToggleFrontierDetails);
        }
    }
    draw_store_grid(
        ctx.data,
        settlement,
        Rect::new(content.x, stores_grid_y, content.w, 34.0),
    );

    let focus_name = settlement
        .focus(ctx.data)
        .map(|focus| focus.name.as_str())
        .unwrap_or_default();
    draw_text_block(
        &ctx.data
            .text_with("ui.focus_current", &[("{name}", focus_name)]),
        content.x,
        y + 116.0,
        content.w,
        24.0,
        16.0,
        2.0,
        style::TEXT_BRIGHT,
    );
    if virtual_button(
        Rect::new(content.x, y + 148.0, content.w, 32.0),
        &ctx.data.text("ui.change_focus"),
        interaction.input_enabled,
        ButtonTone::Secondary,
        interaction.pointer,
    ) {
        interaction.actions.push(UiAction::ToggleFocusChoices);
    }
    let focus_grid_bottom = y + 184.0;
    if !settlement.is_active()
        || ctx
            .data
            .settlement_balance
            .upgrade_from(settlement.tier)
            .is_none()
    {
        return focus_grid_bottom + 10.0;
    }
    let upgrade_rect = Rect::new(content.x, focus_grid_bottom + 9.0, content.w, 28.0);
    let upgrade_status = ctx.session.upgrade_status(ctx.data);
    if inspectable_button(
        upgrade_rect,
        &ctx.session.selected_upgrade_label(ctx.data),
        upgrade_status.enabled,
        interaction.input_enabled,
        ButtonTone::Positive,
        interaction.pointer,
    ) {
        interaction
            .actions
            .push(UiAction::OpenActionReview(ActionReview::UpgradeSettlement));
    }
    style::hover_tooltip(
        interaction.tooltip,
        "selected_upgrade",
        upgrade_rect,
        &upgrade_preview(ctx, settlement, &upgrade_status.reason),
        interaction.pointer,
    );

    upgrade_rect.bottom() + 14.0
}

fn active_issue_count(ctx: &UiContext<'_>, settlement: &SettlementRuntimeState) -> usize {
    ctx.session
        .active_issues
        .iter()
        .filter(|issue| {
            issue.target_site_id == settlement.location_id
                && !matches!(
                    issue.state,
                    ActiveIssueState::Dormant | ActiveIssueState::Resolution
                )
        })
        .count()
}

fn upgrade_preview(
    ctx: &UiContext<'_>,
    settlement: &SettlementRuntimeState,
    status_reason: &str,
) -> String {
    let Some(upgrade) = ctx.data.settlement_balance.upgrade_from(settlement.tier) else {
        return status_reason.to_owned();
    };
    let population = check_text(
        ctx.data,
        settlement.population >= upgrade.min_population,
        "ui.requirement_population",
    );
    let prosperity = check_text(
        ctx.data,
        settlement.prosperity >= upgrade.min_prosperity,
        "ui.requirement_prosperity",
    );
    let stability = check_text(
        ctx.data,
        settlement.stability >= upgrade.min_stability,
        "ui.requirement_stability",
    );
    let stores = check_text(
        ctx.data,
        settlement.stored.deficit_text(upgrade.cost).is_none(),
        "ui.requirement_stores",
    );
    let supply = check_text(
        ctx.data,
        !upgrade.requires_capital_network
            || ctx
                .session
                .is_site_in_capital_network(ctx.data, &settlement.location_id),
        "ui.requirement_supply",
    );
    ctx.data.text_with(
        "ui.upgrade_preview",
        &[
            ("{population}", &population),
            ("{prosperity}", &prosperity),
            ("{stability}", &stability),
            ("{stores}", &stores),
            ("{supply}", &supply),
            ("{reason}", status_reason),
        ],
    )
}

fn check_text(data: &crate::data::GameData, ok: bool, label_id: &str) -> String {
    let label = data.text(label_id);
    if ok {
        data.text_with("ui.requirement_met", &[("{label}", &label)])
    } else {
        data.text_with("ui.requirement_needed", &[("{label}", &label)])
    }
}
