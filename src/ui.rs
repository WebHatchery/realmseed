//! Immediate-mode UI for the Realmseed map, site panel, and chronicle.

use macroquad_toolkit::ui::draw_ui_text_ex;
mod advisor;
mod endgame;
mod event;
mod faction;
mod map;
mod map_site_sprites;
mod map_sites;
mod map_terrain;
mod menu;
mod panel;
mod routes;
mod style;

use crate::data::{GameData, ResourceStock};
use crate::state::GameSession;
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{HoverTooltip, VirtualUi};

pub const LOGICAL_WIDTH: f32 = 1280.0;
pub const LOGICAL_HEIGHT: f32 = 720.0;

const MARGIN: f32 = 12.0;
const COMPACT_MARGIN: f32 = 12.0;
const GAP: f32 = 10.0;
const COMPACT_GAP: f32 = 8.0;
const HEADER_HEIGHT: f32 = 64.0;
const FOOTER_HEIGHT: f32 = 92.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiAction {
    NewGame,
    ContinueGame,
    OpenSettings,
    CloseSettings,
    OpenPauseMenu,
    ClosePauseMenu,
    ToggleFullscreen,
    ExitGame,
    ReturnToTitle,
    ConfirmPendingExit,
    CancelPendingExit,
    SaveAndConfirmPendingExit,
    Save,
    Load,
    DeleteSave,
    SelectSite(String),
    ScoutSelectedSite,
    FoundCamp,
    UpgradeSelectedSettlement,
    SetSettlementFocus(String),
    BuildOrUpgradeRoute(String),
    CompleteRegionalProject(String),
    ResolveEventChoice(String),
    DeferEvent,
    OpenIndependentTrade,
    BeginIndependentIntegration,
    ToggleFactionPanel,
    SetMapOverlay(MapOverlay),
    ZoomMapIn,
    ZoomMapOut,
    SelectAmbition(String),
    CompleteProject(String),
    ActivateInstitution(String),
    AdvanceSeason,
    ToggleChronicle,
    EndgameNewGame,
    EndgameReturnToTitle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapOverlay {
    Realm,
    Supply,
    Danger,
}

impl MapOverlay {
    pub fn label(self) -> &'static str {
        match self {
            Self::Realm => "Realm",
            Self::Supply => "Supply",
            Self::Danger => "Danger",
        }
    }
}

pub struct UiContext<'a> {
    pub data: &'a GameData,
    pub session: &'a GameSession,
    pub sprites: MapSpriteTextures<'a>,
    pub camera_target: Vec2,
    pub camera_zoom: f32,
    pub sprite_showcase: bool,
    pub map_overlay: MapOverlay,
    pub show_chronicle: bool,
    pub show_factions: bool,
    pub input_blocked: bool,
    pub touch_claimed: bool,
    pub pointer: Pointer,
    pub ui: &'a VirtualUi,
}

pub struct MapSpriteTextures<'a> {
    pub capital: Option<&'a Texture2D>,
    pub village: Option<&'a Texture2D>,
    pub ruin: Option<&'a Texture2D>,
    pub resource: Option<&'a Texture2D>,
    pub grove: Option<&'a Texture2D>,
    pub bridge: Option<&'a Texture2D>,
    pub gate: Option<&'a Texture2D>,
    pub waystone: Option<&'a Texture2D>,
    pub shrine: Option<&'a Texture2D>,
}

pub struct MenuContext<'a> {
    pub data: &'a GameData,
    pub title_texture: Option<&'a Texture2D>,
    pub save_exists: bool,
    pub fullscreen: bool,
    pub pointer: Pointer,
    pub ui: &'a VirtualUi,
}

pub struct PauseMenuContext<'a> {
    pub data: &'a GameData,
    pub save_exists: bool,
    pub pending_exit_warning: Option<ExitWarningTarget>,
    pub pointer: Pointer,
    pub ui: &'a VirtualUi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitWarningTarget {
    Title,
    ExitGame,
}

pub fn draw_title_menu(ctx: MenuContext<'_>) -> Vec<UiAction> {
    menu::draw_title_menu(ctx)
}

pub fn draw_settings_page(ctx: MenuContext<'_>) -> Vec<UiAction> {
    menu::draw_settings_page(ctx)
}

pub fn draw_pause_menu(ctx: PauseMenuContext<'_>) -> Vec<UiAction> {
    menu::draw_pause_menu(ctx)
}

pub fn draw_game_ui(ctx: UiContext<'_>, tooltip: &mut HoverTooltip) -> Vec<UiAction> {
    let mut actions = Vec::new();
    let event_open = ctx.session.pending_event.is_some();
    let endgame_open = ctx.session.endgame_summary.is_some();
    let modal_open =
        ctx.input_blocked || ctx.show_chronicle || ctx.show_factions || event_open || endgame_open;
    let input_enabled = !modal_open && !ctx.touch_claimed;

    draw_header(&ctx);
    map::draw_map_panel(&ctx, tooltip, input_enabled, &mut actions);
    advisor::draw_realm_overview(&ctx);
    panel::draw_side_panel(&ctx, tooltip, input_enabled, &mut actions);
    advisor::draw_council_footer(&ctx, input_enabled, &mut actions);
    // The close strategic projection intentionally extends beyond its viewport
    // while panning. Repaint the fixed header last so its chrome stays crisp.
    draw_header(&ctx);
    style::draw_tooltip_overlay(tooltip);

    if endgame_open {
        endgame::draw_endgame_summary(&ctx, &mut actions);
    } else if event_open {
        event::draw_event_modal(&ctx, &mut actions);
    } else if ctx.show_factions {
        faction::draw_faction_overlay(&ctx, &mut actions);
    } else if ctx.show_chronicle {
        panel::draw_chronicle_overlay(&ctx, &mut actions);
    }

    actions
}

fn draw_header(ctx: &UiContext<'_>) {
    let rect = header_rect(ctx);
    style::draw_band(rect);
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.01, 0.020, 0.024, 0.42),
    );

    draw_brand(rect);
    let clock = &ctx.session.clock;
    let clock_x = rect.x
        + if ctx.ui.logical_width < 1040.0 {
            92.0
        } else {
            96.0
        };
    draw_ui_text_ex(
        &format!("Year {}, {}", clock.year, clock.season.label()),
        clock_x,
        rect.y + 27.0,
        TextStyle::new(20.0, style::TEXT_BRIGHT).params(),
    );
    draw_ui_text_ex(
        "Clear Skies",
        clock_x,
        rect.y + 48.0,
        TextStyle::new(12.5, style::TEXT_DIM).params(),
    );

    let totals = realm_totals(ctx);
    let flows = realm_flows(ctx);
    let population: i32 = ctx
        .session
        .settlements
        .iter()
        .map(|settlement| settlement.population)
        .sum();
    let badges = [
        (
            "Food",
            totals.food,
            flows.food,
            style::IconKind::Food,
            Color::new(0.93, 0.66, 0.22, 1.0),
        ),
        (
            "Timber",
            totals.timber,
            flows.timber,
            style::IconKind::Timber,
            Color::new(0.36, 0.65, 0.25, 1.0),
        ),
        (
            "Stone",
            totals.stone,
            flows.stone,
            style::IconKind::Stone,
            Color::new(0.62, 0.58, 0.50, 1.0),
        ),
        (
            "Wealth",
            totals.wealth,
            flows.wealth,
            style::IconKind::Wealth,
            Color::new(0.93, 0.70, 0.26, 1.0),
        ),
        (
            "People",
            population,
            population_flow(ctx),
            style::IconKind::People,
            Color::new(0.48, 0.65, 0.68, 1.0),
        ),
        (
            "Actions",
            ctx.session.council_actions_remaining,
            0,
            style::IconKind::Actions,
            style::GOLD,
        ),
    ];
    let badge_gap = if ctx.ui.logical_width < 1040.0 {
        5.0
    } else {
        9.0
    };
    let title_reserve = if ctx.ui.logical_width < 1040.0 {
        242.0
    } else {
        288.0
    };
    let badge_w = ((rect.w - title_reserve - badge_gap * (badges.len() - 1) as f32 - 18.0)
        / badges.len() as f32)
        .clamp(78.0, 132.0);
    let start_x = (clock_x + 166.0).min(
        rect.right() - badge_w * badges.len() as f32 - badge_gap * (badges.len() - 1) as f32 - 10.0,
    );
    for (index, (label, value, rate, icon, color)) in badges.iter().enumerate() {
        let item = Rect::new(
            start_x + index as f32 * (badge_w + badge_gap),
            rect.y + 11.0,
            badge_w,
            43.0,
        );
        draw_resource_readout(item, label, *value, *rate, *icon, *color);
    }
}

fn draw_brand(rect: Rect) {
    let mark_center = vec2(rect.x + 43.0, rect.y + 31.0);
    draw_circle_lines(mark_center.x, mark_center.y, 21.0, 1.4, style::GOLD);
    draw_circle_lines(
        mark_center.x,
        mark_center.y,
        15.0,
        0.8,
        Color::new(0.80, 0.62, 0.34, 0.38),
    );
    style::draw_icon(style::IconKind::Tree, mark_center, 33.0, style::GOLD);
    style::draw_vertical_divider(rect.x + 76.0, rect.y + 14.0, rect.h - 28.0);
}

fn draw_resource_readout(
    rect: Rect,
    label: &str,
    value: i32,
    rate: i32,
    icon: style::IconKind,
    color: Color,
) {
    style::draw_vertical_divider(rect.x - 5.0, rect.y + 1.0, rect.h - 2.0);
    let icon_center = vec2(rect.x + 16.0, rect.y + 21.0);
    style::draw_icon(icon, icon_center, 27.0, color);
    draw_ui_text_ex(
        &format!("{} {}", label, value),
        rect.x + 34.0,
        rect.y + 18.0,
        TextStyle::new(15.5, style::TEXT_BRIGHT).params(),
    );
    if label == "Actions" {
        return;
    }
    let rate_text = if rate >= 0 {
        format!("+{}/turn", rate)
    } else {
        format!("{}/turn", rate)
    };
    draw_ui_text_ex(
        &rate_text,
        rect.x + 34.0,
        rect.y + 39.0,
        TextStyle::new(12.0, style::TEXT_DIM).params(),
    );
}

fn realm_totals(ctx: &UiContext<'_>) -> ResourceStock {
    let mut totals = ResourceStock::default();
    for settlement in &ctx.session.settlements {
        totals.add(settlement.stored);
    }
    totals
}

fn realm_flows(ctx: &UiContext<'_>) -> ResourceStock {
    let mut totals = ResourceStock::default();
    for settlement in &ctx.session.settlements {
        if !settlement.is_active() {
            continue;
        }
        let Some(focus) = settlement.focus(ctx.data) else {
            continue;
        };
        let tier_modifier = ctx
            .data
            .settlement_balance
            .tier(settlement.tier)
            .map(|tier| tier.production_modifier)
            .unwrap_or(1.0);
        totals.add(focus.output.scaled(tier_modifier / 10.0));
    }
    totals
}

fn population_flow(ctx: &UiContext<'_>) -> i32 {
    ctx.session
        .settlements
        .iter()
        .filter(|settlement| settlement.is_active())
        .map(|settlement| (settlement.population as f32 * 0.03).round() as i32)
        .sum()
}

pub(super) fn section_label(text: &str, x: f32, y: f32) {
    style::draw_panel_title(text, x, y);
}

pub(super) fn virtual_button(
    rect: Rect,
    text: &str,
    enabled: bool,
    tone: ButtonTone,
    pointer: Pointer,
) -> bool {
    let hit_rect = touch_area(rect);
    let hovered = enabled && pointer.hovering_over(rect);
    let pressed = enabled && pointer.pressing(hit_rect);
    let activated = enabled && pointer.released_on(hit_rect);
    style::draw_button_frame(rect, tone, enabled, hovered, pressed);
    draw_text_centered_in_box_ex(
        text,
        rect.x + 8.0,
        rect.y + if pressed { 2.0 } else { 0.0 },
        rect.w - 16.0,
        rect.h,
        TextStyle::new(
            16.0,
            if enabled {
                if matches!(tone, ButtonTone::Primary) {
                    Color::new(0.86, 0.98, 1.0, 1.0)
                } else {
                    style::TEXT_BRIGHT
                }
            } else {
                style::TEXT_DIM
            },
        ),
    );
    activated
}

pub(super) fn virtual_icon_button(
    rect: Rect,
    text: &str,
    icon: style::IconKind,
    enabled: bool,
    tone: ButtonTone,
    pointer: Pointer,
) -> bool {
    let hit_rect = touch_area(rect);
    let hovered = enabled && pointer.hovering_over(rect);
    let pressed = enabled && pointer.pressing(hit_rect);
    let activated = enabled && pointer.released_on(hit_rect);
    style::draw_button_frame(rect, tone, enabled, hovered, pressed);

    let text_color = if enabled {
        if matches!(tone, ButtonTone::Primary) {
            Color::new(0.86, 0.98, 1.0, 1.0)
        } else {
            style::TEXT_BRIGHT
        }
    } else {
        style::TEXT_DIM
    };
    let icon_color = if enabled {
        if matches!(tone, ButtonTone::Primary) {
            style::CYAN
        } else {
            style::GOLD
        }
    } else {
        Color::new(
            style::TEXT_DIM.r,
            style::TEXT_DIM.g,
            style::TEXT_DIM.b,
            0.42,
        )
    };
    let icon_size = (rect.h * 0.58).clamp(15.0, 25.0);
    style::draw_icon(
        icon,
        vec2(rect.x + 20.0, rect.y + rect.h * 0.52),
        icon_size,
        icon_color,
    );
    draw_text_centered_in_box_ex(
        text,
        rect.x + 36.0,
        rect.y + if pressed { 1.5 } else { 0.0 },
        rect.w - 44.0,
        rect.h,
        TextStyle::new(15.0, text_color),
    );
    activated
}

pub(super) fn color_from_array(color: [f32; 4]) -> Color {
    Color::new(color[0], color[1], color[2], color[3])
}

pub(super) fn screen_rect(ctx: &UiContext<'_>) -> Rect {
    Rect::new(0.0, 0.0, ctx.ui.logical_width, ctx.ui.logical_height)
}

pub(super) fn header_rect(ctx: &UiContext<'_>) -> Rect {
    Rect::new(0.0, 0.0, ctx.ui.logical_width, HEADER_HEIGHT)
}

pub(super) fn footer_rect(ctx: &UiContext<'_>) -> Rect {
    let margin = layout_margin(ctx);
    Rect::new(
        margin,
        ctx.ui.logical_height - margin - FOOTER_HEIGHT,
        ctx.ui.logical_width - margin * 2.0,
        FOOTER_HEIGHT,
    )
}

pub(super) fn main_area_rect(ctx: &UiContext<'_>) -> Rect {
    let margin = layout_margin(ctx);
    let header = header_rect(ctx);
    let footer = footer_rect(ctx);
    let y = header.bottom();
    let h = (footer.y - y).max(260.0);
    Rect::new(margin, y, ctx.ui.logical_width - margin * 2.0, h)
}

pub(super) fn left_panel_rect(ctx: &UiContext<'_>) -> Rect {
    let main = main_area_rect(ctx);
    Rect::new(main.x, main.y + 18.0, left_panel_width(ctx), main.h - 30.0)
}

pub(super) fn map_panel_rect(ctx: &UiContext<'_>) -> Rect {
    let main = main_area_rect(ctx);
    let gap = layout_gap(ctx);
    let left = left_panel_rect(ctx);
    let side = side_panel_width(ctx);
    Rect::new(
        left.right() + gap,
        main.y,
        main.w - left.w - side - gap * 2.0,
        main.h,
    )
}

pub(super) fn side_panel_rect(ctx: &UiContext<'_>) -> Rect {
    let main = main_area_rect(ctx);
    Rect::new(
        main.right() - side_panel_width(ctx),
        main.y + 18.0,
        side_panel_width(ctx),
        main.h - 30.0,
    )
}

pub(super) fn centered_modal_rect(ctx: &UiContext<'_>, max_w: f32, max_h: f32) -> Rect {
    let screen = screen_rect(ctx);
    let margin = layout_margin(ctx) * 2.0;
    let w = max_w.min(screen.w - margin * 2.0).max(320.0);
    let h = max_h.min(screen.h - margin * 2.0).max(260.0);
    Rect::new((screen.w - w) * 0.5, (screen.h - h) * 0.5, w, h)
}

fn layout_margin(ctx: &UiContext<'_>) -> f32 {
    if ctx.ui.logical_width < 1040.0 {
        COMPACT_MARGIN
    } else {
        MARGIN
    }
}

fn layout_gap(ctx: &UiContext<'_>) -> f32 {
    if ctx.ui.logical_width < 1040.0 {
        COMPACT_GAP
    } else {
        GAP
    }
}

fn side_panel_width(ctx: &UiContext<'_>) -> f32 {
    if ctx.ui.logical_width < 1040.0 {
        (ctx.ui.logical_width * 0.31).clamp(286.0, 320.0)
    } else {
        (ctx.ui.logical_width * 0.29).clamp(320.0, 354.0)
    }
}

fn left_panel_width(ctx: &UiContext<'_>) -> f32 {
    if ctx.ui.logical_width < 1040.0 {
        (ctx.ui.logical_width * 0.22).clamp(184.0, 214.0)
    } else {
        (ctx.ui.logical_width * 0.19).clamp(220.0, 246.0)
    }
}
