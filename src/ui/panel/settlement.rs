//! Active-settlement management: metrics, focus selection, and upgrades.

use super::readouts::{draw_focus_card, draw_metric_grid, draw_store_grid, focus_icon};
use crate::data::ActiveIssueState;
use crate::state::SettlementRuntimeState;
use crate::ui::{style, virtual_icon_button, UiAction, UiContext};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::HoverTooltip;

#[derive(Debug, Clone, Copy)]
struct FocusButtonLayout {
    columns: usize,
    column_gap: f32,
    row_gap: f32,
    button_height: f32,
}

impl FocusButtonLayout {
    const fn compact() -> Self {
        Self {
            columns: 3,
            column_gap: 6.0,
            row_gap: 4.0,
            button_height: 25.0,
        }
    }

    const fn roomy() -> Self {
        Self {
            columns: 2,
            column_gap: 8.0,
            row_gap: 6.0,
            button_height: 28.0,
        }
    }

    fn rows(self, item_count: usize) -> usize {
        item_count.div_ceil(self.columns)
    }

    fn row_step(self) -> f32 {
        self.button_height + self.row_gap
    }

    fn grid_bottom(self, y: f32, item_count: usize) -> f32 {
        let rows = self.rows(item_count);
        if rows == 0 {
            y
        } else {
            y + (rows - 1) as f32 * self.row_step() + self.button_height
        }
    }
}

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
    let compact = ctx.ui.logical_width < 1040.0;
    let stores_label_y = if compact { y + 50.0 } else { y + 48.0 };
    let stores_grid_y = y + 60.0;
    crate::ui::section_label(&ctx.data.text("ui.stores"), content.x, stores_label_y);
    if active_issue_count > 0 {
        draw_badge(
            Rect::new(content.right() - 84.0, stores_label_y - 13.0, 84.0, 22.0),
            &ctx.data.text_with(
                "ui.active_issues_count",
                &[("{count}", &active_issue_count.to_string())],
            ),
            Color::new(0.34, 0.14, 0.12, 1.0),
            style::TEXT,
        );
    }
    draw_store_grid(
        ctx.data,
        settlement,
        Rect::new(content.x, stores_grid_y, content.w, 34.0),
    );

    let focus_label_y = if compact { y + 102.0 } else { y + 108.0 };
    let focus_card_y = if compact { y + 116.0 } else { y + 122.0 };
    crate::ui::section_label(&ctx.data.text("ui.current_focus"), content.x, focus_label_y);
    draw_focus_card(
        ctx,
        settlement,
        Rect::new(content.x, focus_card_y, content.w, 44.0),
    );

    let actions_label_y = if compact { y + 162.0 } else { y + 178.0 };
    let focus_grid_y = actions_label_y + 13.0;
    let focus_layout = focus_button_layout(ctx, content, y, settlement);
    crate::ui::section_label(
        &ctx.data.text("ui.settlement_actions"),
        content.x,
        actions_label_y,
    );
    let focus_grid_bottom = draw_focus_buttons(
        ctx,
        interaction,
        content,
        focus_grid_y,
        settlement,
        focus_layout,
    );
    let upgrade_rect = Rect::new(content.x, focus_grid_bottom + 9.0, content.w, 28.0);
    let upgrade_status = ctx.session.upgrade_status(ctx.data);
    if virtual_icon_button(
        upgrade_rect,
        &ctx.session.selected_upgrade_label(ctx.data),
        style::IconKind::Castle,
        interaction.input_enabled && upgrade_status.enabled,
        ButtonTone::Positive,
        interaction.pointer,
    ) {
        interaction
            .actions
            .push(UiAction::UpgradeSelectedSettlement);
    }
    style::hover_tooltip(
        interaction.tooltip,
        "selected_upgrade",
        upgrade_rect,
        &upgrade_preview(ctx, settlement, &upgrade_status.reason),
        interaction.pointer,
    );

    let mut next_y = upgrade_rect.bottom() + 10.0;
    let minimum_next_y = if compact { y + 266.0 } else { y + 282.0 };
    if next_y < minimum_next_y {
        next_y = minimum_next_y;
    }

    next_y
}

fn focus_button_layout(
    ctx: &UiContext<'_>,
    content: Rect,
    y: f32,
    settlement: &SettlementRuntimeState,
) -> FocusButtonLayout {
    let roomy = FocusButtonLayout::roomy();
    if focus_layout_fits(ctx, content, y, settlement, roomy) {
        roomy
    } else {
        FocusButtonLayout::compact()
    }
}

fn focus_layout_fits(
    ctx: &UiContext<'_>,
    content: Rect,
    y: f32,
    settlement: &SettlementRuntimeState,
    layout: FocusButtonLayout,
) -> bool {
    let actions_label_y = if ctx.ui.logical_width < 1040.0 {
        y + 162.0
    } else {
        y + 178.0
    };
    let focus_grid_y = actions_label_y + 13.0;
    let focus_grid_bottom =
        layout.grid_bottom(focus_grid_y, ctx.data.settlement_balance.focuses.len());
    let upgrade_bottom = focus_grid_bottom + 9.0 + 28.0;
    let minimum_next_y = if ctx.ui.logical_width < 1040.0 {
        y + 266.0
    } else {
        y + 282.0
    };
    let route_y = (upgrade_bottom + 10.0).max(minimum_next_y) + 4.0;
    let route_count = ctx
        .session
        .routes_for_site(&settlement.location_id)
        .filter(|route| route.known)
        .take(2)
        .count();
    let route_bottom = route_y + route_section_height(route_count);

    route_bottom <= content.bottom() + 16.0
}

fn route_section_height(route_count: usize) -> f32 {
    if route_count == 0 {
        48.0
    } else {
        46.0 + route_count as f32 * 24.0 + 2.0
    }
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

fn draw_focus_buttons(
    ctx: &UiContext<'_>,
    interaction: &mut SettlementInteraction<'_>,
    content: Rect,
    y: f32,
    settlement: &SettlementRuntimeState,
    layout: FocusButtonLayout,
) -> f32 {
    let col_w =
        (content.w - layout.column_gap * (layout.columns - 1) as f32) / layout.columns as f32;
    for (index, focus) in ctx.data.settlement_balance.focuses.iter().enumerate() {
        let col = (index % layout.columns) as f32;
        let row = (index / layout.columns) as f32;
        let rect = Rect::new(
            content.x + col * (col_w + layout.column_gap),
            y + row * layout.row_step(),
            col_w,
            layout.button_height,
        );
        let status = ctx.session.focus_change_status(ctx.data, &focus.id);
        let is_current = settlement.focus_id == focus.id;
        let tone = if is_current {
            ButtonTone::Primary
        } else {
            ButtonTone::Secondary
        };
        if virtual_icon_button(
            rect,
            &focus.name,
            focus_icon(&focus.name),
            interaction.input_enabled && status.enabled,
            tone,
            interaction.pointer,
        ) {
            interaction
                .actions
                .push(UiAction::SetSettlementFocus(focus.id.clone()));
        }
    }
    layout.grid_bottom(y, ctx.data.settlement_balance.focuses.len())
}
