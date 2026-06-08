//! Selected-site panel, campaign controls, and chronicle overlay.

use super::{routes, virtual_button, UiAction, UiContext};
use crate::state::SiteKnowledge;
use crate::state::{SettlementRuntimeState, SettlementStatus};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::RectExt;

pub(super) fn draw_side_panel(
    ctx: &UiContext<'_>,
    mouse: Vec2,
    input_enabled: bool,
    actions: &mut Vec<UiAction>,
) {
    let rect = Rect::new(852.0, 96.0, 410.0, 520.0);
    let style = SurfaceStyle::new(Color::new(0.075, 0.080, 0.065, 0.98))
        .with_border(1.0, Color::new(0.52, 0.47, 0.32, 0.70))
        .with_header(42.0, Color::new(0.095, 0.100, 0.080, 1.0))
        .with_header_divider(1.0, Color::new(0.52, 0.47, 0.32, 0.35));
    draw_surface_with_title(
        rect,
        Some("Selected Site"),
        &style,
        TextStyle::new(18.0, dark::TEXT),
    );

    let content = rect.inset(18.0);
    let mut y = content.y + 38.0;
    y = draw_selected_site(ctx, content, y);
    y = draw_settlement_section(ctx, mouse, input_enabled, actions, content, y + 8.0);
    y = routes::draw_route_section(ctx, mouse, input_enabled, actions, content, y + 8.0);
    y = draw_campaign_actions(ctx, mouse, input_enabled, actions, content, y + 10.0);
    if y + 110.0 < content.y + content.h {
        draw_save_actions(ctx, mouse, input_enabled, actions, content, y + 10.0);
    }
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
    draw_text_block(
        title,
        content.x,
        y,
        content.w,
        34.0,
        24.0,
        2.0,
        dark::TEXT_BRIGHT,
    );

    let status = if knowledge == SiteKnowledge::Known {
        "Known"
    } else {
        "Unknown adjacent"
    };
    draw_badge(
        Rect::new(content.x, y + 42.0, 138.0, 28.0),
        status,
        Color::new(0.20, 0.24, 0.18, 1.0),
        dark::TEXT,
    );
    draw_badge(
        Rect::new(content.x + 148.0, y + 42.0, 172.0, 28.0),
        site.category.label(),
        Color::new(0.22, 0.20, 0.16, 1.0),
        dark::TEXT,
    );

    if ctx.session.settlement_at_site(&site.id).is_some() {
        return y + 76.0;
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
        format!(
            "Region: {}\nKnown status: {}\nScout this marker to add its name, traits, and road context to the chronicle.",
            region_name,
            knowledge.label()
        )
    };
    draw_text_block(
        &known_text,
        content.x,
        y + 78.0,
        content.w,
        50.0,
        15.0,
        3.0,
        dark::TEXT_DIM,
    );

    y + 136.0
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
        return y;
    }

    draw_text_ex(
        "Settlement",
        content.x,
        y,
        TextStyle::new(18.0, dark::TEXT_BRIGHT).params(),
    );

    if let Some(settlement) = ctx.session.settlement_at_site(&site.id) {
        draw_existing_settlement(
            ctx,
            mouse,
            input_enabled,
            actions,
            content,
            y + 18.0,
            settlement,
        )
    } else {
        draw_found_camp_action(ctx, mouse, input_enabled, actions, content, y + 18.0)
    }
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
    let status_tone = if settlement.status == SettlementStatus::Lost {
        Color::new(0.34, 0.16, 0.14, 1.0)
    } else {
        Color::new(0.18, 0.26, 0.18, 1.0)
    };
    draw_badge(
        Rect::new(content.x, y, 94.0, 24.0),
        settlement.tier.label(),
        Color::new(0.20, 0.23, 0.18, 1.0),
        dark::TEXT,
    );
    draw_badge(
        Rect::new(content.x + 104.0, y, 92.0, 24.0),
        settlement.status.label(),
        status_tone,
        dark::TEXT,
    );
    draw_text_ex(
        &format!(
            "Pop {}  Founded {} Y{}",
            settlement.population,
            settlement.founded_season.label(),
            settlement.founded_year
        ),
        content.x + 206.0,
        y + 18.0,
        TextStyle::new(14.0, dark::TEXT_DIM).params(),
    );

    let resources = format!(
        "Food {}   Timber {}   Stone {}   Wealth {}",
        settlement.stored.food,
        settlement.stored.timber,
        settlement.stored.stone,
        settlement.stored.wealth
    );
    draw_text_ex(
        &resources,
        content.x,
        y + 36.0,
        TextStyle::new(15.0, dark::TEXT).params(),
    );

    draw_metric_row(
        content.x,
        y + 52.0,
        [
            ("Stab", settlement.stability),
            ("Loyal", settlement.loyalty),
            ("Pros", settlement.prosperity),
            ("Def", settlement.defence),
            ("Danger", settlement.danger),
        ],
    );

    let focus_name = settlement
        .focus(ctx.data)
        .map(|focus| focus.name.as_str())
        .unwrap_or("Unknown");
    draw_text_ex(
        &format!("Focus: {}", focus_name),
        content.x,
        y + 88.0,
        TextStyle::new(15.0, dark::TEXT_BRIGHT).params(),
    );
    draw_focus_buttons(
        ctx,
        mouse,
        input_enabled,
        actions,
        content,
        y + 100.0,
        settlement,
    );

    let upgrade_status = ctx.session.upgrade_status(ctx.data);
    if virtual_button(
        Rect::new(content.x, y + 158.0, content.w, 30.0),
        &ctx.session.selected_upgrade_label(ctx.data),
        input_enabled && upgrade_status.enabled,
        ButtonTone::Positive,
        mouse,
    ) {
        actions.push(UiAction::UpgradeSelectedSettlement);
    }
    draw_text_block(
        &upgrade_status.reason,
        content.x,
        y + 190.0,
        content.w,
        24.0,
        13.0,
        2.0,
        if upgrade_status.enabled {
            dark::TEXT_DIM
        } else {
            dark::WARNING
        },
    );

    y + 218.0
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
        let rect = Rect::new(content.x + col * (col_w + 6.0), y + row * 27.0, col_w, 23.0);
        let status = ctx.session.focus_change_status(ctx.data, &focus.id);
        let is_current = settlement.focus_id == focus.id;
        let tone = if is_current {
            ButtonTone::Primary
        } else {
            ButtonTone::Secondary
        };
        if virtual_button(
            rect,
            &focus.name,
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
    if virtual_button(
        Rect::new(content.x, y, content.w, 34.0),
        "Found Camp",
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

fn draw_metric_row(x: f32, y: f32, metrics: [(&str, i32); 5]) {
    let badge_w = 68.0;
    for (index, (label, value)) in metrics.iter().enumerate() {
        let rect = Rect::new(x + index as f32 * (badge_w + 5.0), y, badge_w, 25.0);
        let color = if *label == "Danger" {
            Color::new(0.28, 0.18, 0.14, 1.0)
        } else {
            Color::new(0.18, 0.22, 0.18, 1.0)
        };
        draw_badge(rect, &format!("{} {}", label, value), color, dark::TEXT);
    }
}

fn draw_campaign_actions(
    ctx: &UiContext<'_>,
    mouse: Vec2,
    input_enabled: bool,
    actions: &mut Vec<UiAction>,
    content: Rect,
    y: f32,
) -> f32 {
    draw_text_ex(
        "Campaign",
        content.x,
        y,
        TextStyle::new(18.0, dark::TEXT_BRIGHT).params(),
    );
    let next_y = y + 16.0;
    let third = (content.w - 16.0) / 3.0;
    let scout_enabled = input_enabled && ctx.session.can_scout_selected_site(ctx.data);
    if virtual_button(
        Rect::new(content.x, next_y, third, 34.0),
        "Scout",
        scout_enabled,
        ButtonTone::Primary,
        mouse,
    ) {
        actions.push(UiAction::ScoutSelectedSite);
    }

    if virtual_button(
        Rect::new(content.x + third + 8.0, next_y, third, 34.0),
        "Advance",
        input_enabled,
        ButtonTone::Positive,
        mouse,
    ) {
        actions.push(UiAction::AdvanceSeason);
    }
    let chronicle_label = if ctx.show_chronicle {
        "Close Chronicle"
    } else {
        "Chronicle"
    };
    if virtual_button(
        Rect::new(content.x + (third + 8.0) * 2.0, next_y, third, 34.0),
        chronicle_label,
        input_enabled,
        ButtonTone::Secondary,
        mouse,
    ) {
        actions.push(UiAction::ToggleChronicle);
    }
    next_y + 44.0
}

fn draw_save_actions(
    ctx: &UiContext<'_>,
    mouse: Vec2,
    input_enabled: bool,
    actions: &mut Vec<UiAction>,
    content: Rect,
    y: f32,
) {
    draw_text_ex(
        "Campaign Data",
        content.x,
        y,
        TextStyle::new(18.0, dark::TEXT_BRIGHT).params(),
    );
    let mut next_y = y + 16.0;
    let half = (content.w - 10.0) / 2.0;

    if virtual_button(
        Rect::new(content.x, next_y, half, 36.0),
        "New Campaign",
        input_enabled,
        ButtonTone::Secondary,
        mouse,
    ) {
        actions.push(UiAction::NewGame);
    }
    if virtual_button(
        Rect::new(content.x + half + 10.0, next_y, half, 36.0),
        "Save",
        input_enabled,
        ButtonTone::Positive,
        mouse,
    ) {
        actions.push(UiAction::Save);
    }
    next_y += 46.0;

    if virtual_button(
        Rect::new(content.x, next_y, half, 34.0),
        "Load",
        input_enabled && ctx.save_exists,
        ButtonTone::Primary,
        mouse,
    ) {
        actions.push(UiAction::Load);
    }
    if virtual_button(
        Rect::new(content.x + half + 10.0, next_y, half, 34.0),
        "Delete Save",
        input_enabled && ctx.save_exists,
        ButtonTone::Danger,
        mouse,
    ) {
        actions.push(UiAction::DeleteSave);
    }
    next_y += 44.0;

    let saves = if ctx.save_slots.is_empty() {
        "No save slots found".to_owned()
    } else {
        format!("Save slots: {}", ctx.save_slots.join(", "))
    };
    draw_text_block(
        &format!(
            "{}\nManifest textures loaded: {}\nToolkit slot: {}",
            saves, ctx.loaded_assets, ctx.data.config.save_slot
        ),
        content.x,
        next_y,
        content.w,
        58.0,
        14.0,
        3.0,
        dark::TEXT_DIM,
    );
}

pub(super) fn draw_chronicle_overlay(
    ctx: &UiContext<'_>,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
) {
    let shade = Color::new(0.02, 0.025, 0.02, 0.60);
    draw_rectangle(0.0, 0.0, super::LOGICAL_WIDTH, super::LOGICAL_HEIGHT, shade);

    let rect = Rect::new(172.0, 82.0, 936.0, 556.0);
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
            "{} Year {} - {}",
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
