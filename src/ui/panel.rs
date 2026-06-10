//! Selected-site panel, campaign controls, and chronicle overlay.

use super::{routes, style, virtual_button, virtual_icon_button, UiAction, UiContext};
use crate::data::{ActiveIssueState, SiteCategory};
use crate::state::SettlementRuntimeState;
use crate::state::SiteKnowledge;
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::RectExt;

pub(super) fn draw_side_panel(
    ctx: &UiContext<'_>,
    mouse: Vec2,
    input_enabled: bool,
    actions: &mut Vec<UiAction>,
) {
    let rect = super::side_panel_rect(ctx);
    style::draw_panel(rect);

    let content = rect.inset(18.0);
    style::draw_panel_title("SELECTED SITE", content.x, content.y + 16.0);
    let mut y = content.y + 50.0;
    y = draw_selected_site(ctx, content, y);
    y = draw_settlement_section(ctx, mouse, input_enabled, actions, content, y + 2.0);
    routes::draw_route_section(ctx, mouse, input_enabled, actions, content, y + 4.0);
}

fn draw_selected_site(ctx: &UiContext<'_>, content: Rect, y: f32) -> f32 {
    let Some(site) = ctx.session.selected_site(ctx.data) else {
        draw_text_ex(
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
    draw_text_ex(
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

fn draw_settlement_section(
    ctx: &UiContext<'_>,
    mouse: Vec2,
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
        return draw_scout_action(ctx, mouse, input_enabled, actions, content, y + 14.0);
    }

    if let Some(settlement) = ctx.session.settlement_at_site(&site.id) {
        draw_existing_settlement(ctx, mouse, input_enabled, actions, content, y, settlement)
    } else if site.category == SiteCategory::Independent {
        super::section_label("SITE DECISIONS", content.x, y);
        draw_independent_actions(ctx, mouse, input_enabled, actions, content, y + 14.0)
    } else {
        super::section_label("SITE DECISIONS", content.x, y);
        draw_found_camp_action(ctx, mouse, input_enabled, actions, content, y + 14.0)
    }
}

fn draw_scout_action(
    ctx: &UiContext<'_>,
    mouse: Vec2,
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
        mouse,
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

fn draw_independent_actions(
    ctx: &UiContext<'_>,
    mouse: Vec2,
    input_enabled: bool,
    actions: &mut Vec<UiAction>,
    content: Rect,
    y: f32,
) -> f32 {
    let Some(independent) = ctx.session.selected_independent() else {
        return y;
    };
    draw_text_ex(
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
    let trade_status = ctx.session.independent_trade_status();
    let integration_status = ctx.session.integration_status();
    let half = (content.w - 8.0) / 2.0;
    if virtual_icon_button(
        Rect::new(content.x, y + 20.0, half, 30.0),
        "Open Trade",
        style::IconKind::Wealth,
        input_enabled && trade_status.enabled,
        ButtonTone::Primary,
        mouse,
    ) {
        actions.push(UiAction::OpenIndependentTrade);
    }
    if virtual_icon_button(
        Rect::new(content.x + half + 8.0, y + 20.0, half, 30.0),
        "Integrate",
        style::IconKind::Crown,
        input_enabled && integration_status.enabled,
        ButtonTone::Positive,
        mouse,
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

fn draw_existing_settlement(
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
    super::section_label("STORES", content.x, y + 48.0);
    if active_issue_count > 0 {
        draw_badge(
            Rect::new(content.right() - 84.0, y + 35.0, 84.0, 22.0),
            &format!("{} issue", active_issue_count),
            Color::new(0.34, 0.14, 0.12, 1.0),
            style::TEXT,
        );
    }
    draw_store_grid(settlement, Rect::new(content.x, y + 60.0, content.w, 34.0));

    super::section_label("CURRENT FOCUS", content.x, y + 108.0);
    draw_focus_card(
        ctx,
        settlement,
        Rect::new(content.x, y + 122.0, content.w, 44.0),
    );

    let actions_label_y = y + 178.0;
    let focus_grid_y = actions_label_y + 13.0;
    super::section_label("ACTIONS", content.x, actions_label_y);
    draw_focus_buttons(
        ctx,
        mouse,
        input_enabled,
        actions,
        content,
        focus_grid_y,
        settlement,
    );
    let focus_rows = ctx.data.settlement_balance.focuses.len().div_ceil(3) as f32;
    let focus_grid_bottom = focus_grid_y + (focus_rows - 1.0).max(0.0) * 29.0 + 25.0;
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

fn focus_summary(ctx: &UiContext<'_>, settlement: &SettlementRuntimeState) -> String {
    let Some(focus) = settlement.focus(ctx.data) else {
        return "No focus selected.".to_owned();
    };
    format!(
        "{}: {}. {}",
        focus.name,
        resource_output_text(focus.output),
        focus.notes
    )
}

fn resource_output_text(output: crate::data::ResourceStock) -> String {
    let mut parts = Vec::new();
    if output.food != 0 {
        parts.push(format!("{:+} food", output.food));
    }
    if output.timber != 0 {
        parts.push(format!("{:+} timber", output.timber));
    }
    if output.stone != 0 {
        parts.push(format!("{:+} stone", output.stone));
    }
    if output.wealth != 0 {
        parts.push(format!("{:+} wealth", output.wealth));
    }
    if parts.is_empty() {
        "steady holdings".to_owned()
    } else {
        parts.join(", ")
    }
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
) {
    let col_w = (content.w - 12.0) / 3.0;
    for (index, focus) in ctx.data.settlement_balance.focuses.iter().enumerate() {
        let col = (index % 3) as f32;
        let row = (index / 3) as f32;
        let rect = Rect::new(content.x + col * (col_w + 6.0), y + row * 29.0, col_w, 25.0);
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
}

fn draw_found_camp_action(
    ctx: &UiContext<'_>,
    mouse: Vec2,
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
        mouse,
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

fn draw_metric_grid(x: f32, y: f32, width: f32, metrics: [(&str, i32); 4]) {
    let cell_w = width / metrics.len() as f32;
    for (index, (label, value)) in metrics.iter().enumerate() {
        let rect = Rect::new(x + index as f32 * cell_w, y, cell_w, 44.0);
        draw_surface(
            rect,
            &SurfaceStyle::new(Color::new(0.020, 0.034, 0.036, 0.76))
                .with_border(1.0, Color::new(0.58, 0.45, 0.25, 0.22)),
        );
        draw_text_centered_in_box_ex(
            label,
            rect.x + 4.0,
            rect.y + 4.0,
            rect.w - 8.0,
            16.0,
            TextStyle::new(10.5, style::TEXT_DIM),
        );
        let value_color = if *label == "Danger" {
            style::RED
        } else {
            style::TEXT_BRIGHT
        };
        draw_text_centered_in_box_ex(
            &value.to_string(),
            rect.x + 4.0,
            rect.y + 23.0,
            rect.w - 8.0,
            20.0,
            TextStyle::new(16.0, value_color),
        );
    }
}

fn draw_store_grid(settlement: &SettlementRuntimeState, rect: Rect) {
    let values = [
        (
            "Food",
            settlement.stored.food,
            style::IconKind::Food,
            Color::new(0.93, 0.66, 0.22, 1.0),
        ),
        (
            "Timber",
            settlement.stored.timber,
            style::IconKind::Timber,
            Color::new(0.36, 0.65, 0.25, 1.0),
        ),
        (
            "Stone",
            settlement.stored.stone,
            style::IconKind::Stone,
            Color::new(0.62, 0.58, 0.50, 1.0),
        ),
        (
            "Wealth",
            settlement.stored.wealth,
            style::IconKind::Wealth,
            Color::new(0.93, 0.70, 0.26, 1.0),
        ),
    ];
    let cell_w = rect.w / values.len() as f32;
    for (index, (label, amount, icon, color)) in values.iter().enumerate() {
        let x = rect.x + index as f32 * cell_w;
        style::draw_icon(*icon, vec2(x + 12.0, rect.y + 17.0), 22.0, *color);
        draw_text_ex(
            label,
            x + 26.0,
            rect.y + 11.0,
            TextStyle::new(11.0, style::TEXT_DIM).params(),
        );
        draw_text_ex(
            &amount.to_string(),
            x + 26.0,
            rect.y + 28.0,
            TextStyle::new(15.0, style::TEXT_BRIGHT).params(),
        );
        if index + 1 < values.len() {
            style::draw_vertical_divider(x + cell_w - 6.0, rect.y + 2.0, rect.h - 4.0);
        }
    }
}

fn draw_focus_card(ctx: &UiContext<'_>, settlement: &SettlementRuntimeState, rect: Rect) {
    draw_surface(
        rect,
        &SurfaceStyle::new(Color::new(0.022, 0.041, 0.045, 0.74))
            .with_border(1.0, Color::new(0.58, 0.45, 0.25, 0.22)),
    );
    let focus_name = settlement
        .focus(ctx.data)
        .map(|focus| focus.name.as_str())
        .unwrap_or("Unassigned");
    style::draw_icon(
        focus_icon(focus_name),
        vec2(rect.x + 24.0, rect.y + 22.0),
        30.0,
        style::GOLD,
    );
    draw_text_ex(
        focus_name,
        rect.x + 54.0,
        rect.y + 17.0,
        TextStyle::new(15.0, style::TEXT_BRIGHT).params(),
    );
    draw_text_block(
        &focus_summary(ctx, settlement),
        rect.x + 54.0,
        rect.y + 21.0,
        rect.w - 62.0,
        17.0,
        10.0,
        1.0,
        style::TEXT_DIM,
    );
}

fn focus_icon(name: &str) -> style::IconKind {
    match name {
        "Farming" => style::IconKind::Food,
        "Logging" => style::IconKind::Timber,
        "Quarrying" => style::IconKind::Stone,
        "Trade" => style::IconKind::Wealth,
        "Fortify" => style::IconKind::Castle,
        _ => style::IconKind::Crown,
    }
}

pub(super) fn draw_chronicle_overlay(
    ctx: &UiContext<'_>,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
) {
    let shade = Color::new(0.02, 0.025, 0.02, 0.60);
    let screen = super::screen_rect(ctx);
    draw_rectangle(screen.x, screen.y, screen.w, screen.h, shade);

    let rect = super::centered_modal_rect(ctx, 936.0, 556.0);
    let style = SurfaceStyle::new(Color::new(0.075, 0.070, 0.055, 0.99))
        .with_border(1.0, Color::new(0.64, 0.55, 0.34, 0.85))
        .with_header(48.0, Color::new(0.10, 0.095, 0.075, 1.0))
        .with_header_divider(1.0, Color::new(0.64, 0.55, 0.34, 0.45));
    draw_surface_with_title(
        rect,
        Some("Chronicle"),
        &style,
        TextStyle::new(20.0, dark::TEXT_BRIGHT),
    );

    if virtual_button(
        Rect::new(rect.right() - 96.0, rect.y + 10.0, 76.0, 30.0),
        "Close",
        true,
        ButtonTone::Secondary,
        mouse,
    ) {
        actions.push(UiAction::ToggleChronicle);
    }

    let content = rect.inset(24.0);
    let mut y = content.y + 48.0;
    for entry in ctx.session.chronicle.iter().rev().take(8) {
        let title = format!(
            "[{}] {} Year {} - {}",
            entry.tag,
            entry.season.label(),
            entry.year,
            entry.title
        );
        draw_text_ex(
            &title,
            content.x,
            y,
            TextStyle::new(17.0, dark::TEXT_BRIGHT).params(),
        );
        draw_text_block(
            &entry.body,
            content.x + 14.0,
            y + 10.0,
            content.w - 28.0,
            44.0,
            15.0,
            3.0,
            dark::TEXT_DIM,
        );
        y += 70.0;
    }

    if ctx.session.chronicle.is_empty() {
        draw_text_centered_in_box(
            "No chronicle entries yet.",
            content.x,
            content.y + 90.0,
            content.w,
            50.0,
            18.0,
            dark::TEXT_DIM,
        );
    }
}
