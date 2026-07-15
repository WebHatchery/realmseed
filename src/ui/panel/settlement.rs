//! Active-settlement management: metrics, focus selection, and upgrades.

use super::readouts::{draw_focus_card, draw_metric_grid, draw_store_grid, focus_icon};
use crate::data::ActiveIssueState;
use crate::state::SettlementRuntimeState;
use crate::ui::{style, virtual_icon_button, UiAction, UiContext};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;

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

pub(super) fn draw_existing_settlement(
    ctx: &UiContext<'_>,
    mouse: Vec2,
    input_enabled: bool,
    actions: &mut Vec<UiAction>,
    content: Rect,
    y: f32,
    settlement: &SettlementRuntimeState,
) -> f32 {
    draw_metric_grid(
        content.x,
        y,
        content.w,
        [
            ("Population", settlement.population),
            ("Loyalty", settlement.loyalty),
            ("Stability", settlement.stability),
            ("Danger", settlement.danger),
        ],
    );

    let active_issue_count = active_issue_count(ctx, settlement);
    crate::ui::section_label("STORES", content.x, y + 48.0);
    if active_issue_count > 0 {
        draw_badge(
            Rect::new(content.right() - 84.0, y + 35.0, 84.0, 22.0),
            &format!("{} issue", active_issue_count),
            Color::new(0.34, 0.14, 0.12, 1.0),
            style::TEXT,
        );
    }
    draw_store_grid(settlement, Rect::new(content.x, y + 60.0, content.w, 34.0));

    crate::ui::section_label("CURRENT FOCUS", content.x, y + 108.0);
    draw_focus_card(
        ctx,
        settlement,
        Rect::new(content.x, y + 122.0, content.w, 44.0),
    );

    let actions_label_y = y + 178.0;
    let focus_grid_y = actions_label_y + 13.0;
    let focus_layout = focus_button_layout(ctx, content, y, settlement);
    crate::ui::section_label("ACTIONS", content.x, actions_label_y);
    let focus_grid_bottom = draw_focus_buttons(
        ctx,
        mouse,
        input_enabled,
        actions,
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
        input_enabled && upgrade_status.enabled,
        ButtonTone::Positive,
        mouse,
    ) {
        actions.push(UiAction::UpgradeSelectedSettlement);
    }
    style::draw_hover_tooltip(
        "selected_upgrade",
        upgrade_rect,
        &upgrade_preview(ctx, settlement, &upgrade_status.reason),
        mouse,
    );

    let mut next_y = upgrade_rect.bottom() + 10.0;
    if next_y < y + 282.0 {
        next_y = y + 282.0;
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
    let actions_label_y = y + 178.0;
    let focus_grid_y = actions_label_y + 13.0;
    let focus_grid_bottom =
        layout.grid_bottom(focus_grid_y, ctx.data.settlement_balance.focuses.len());
    let upgrade_bottom = focus_grid_bottom + 9.0 + 28.0;
    let route_y = (upgrade_bottom + 10.0).max(y + 282.0) + 4.0;
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
    let population = check_text(settlement.population >= upgrade.min_population, "pop");
    let prosperity = check_text(settlement.prosperity >= upgrade.min_prosperity, "pros");
    let stability = check_text(settlement.stability >= upgrade.min_stability, "stab");
    let stores = check_text(
        settlement.stored.deficit_text(upgrade.cost).is_none(),
        "stores",
    );
    let supply = check_text(
        !upgrade.requires_capital_network
            || ctx
                .session
                .is_site_in_capital_network(ctx.data, &settlement.location_id),
        "supply",
    );
    format!(
        "{} {} {} {} {}. {}",
        population, prosperity, stability, stores, supply, status_reason
    )
}

fn check_text(ok: bool, label: &str) -> String {
    if ok {
        format!("[ok] {}", label)
    } else {
        format!("[need] {}", label)
    }
}

fn draw_focus_buttons(
    ctx: &UiContext<'_>,
    mouse: Vec2,
    input_enabled: bool,
    actions: &mut Vec<UiAction>,
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
            input_enabled && status.enabled,
            tone,
            mouse,
        ) {
            actions.push(UiAction::SetSettlementFocus(focus.id.clone()));
        }
    }
    layout.grid_bottom(y, ctx.data.settlement_balance.focuses.len())
}
