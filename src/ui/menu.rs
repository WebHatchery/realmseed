//! Title, pause, and settings menu rendering.

use super::{style, virtual_button, ExitWarningTarget, MenuContext, PauseMenuContext, UiAction};
use crate::data::GameData;
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::RectExt;

pub(super) fn draw_title_menu(ctx: MenuContext<'_>) -> Vec<UiAction> {
    let mut actions = Vec::new();
    let screen = Rect::new(0.0, 0.0, ctx.ui.logical_width, ctx.ui.logical_height);

    draw_title_background(screen, ctx.title_texture);
    draw_title_vignette(screen);

    let menu = menu_rect(screen);
    draw_menu_buttons(ctx.data, menu, ctx.pointer, ctx.save_exists, &mut actions);

    actions
}

pub(super) fn draw_settings_page(ctx: MenuContext<'_>) -> Vec<UiAction> {
    let mut actions = Vec::new();
    let screen = Rect::new(0.0, 0.0, ctx.ui.logical_width, ctx.ui.logical_height);

    draw_title_background(screen, ctx.title_texture);
    draw_rectangle(
        screen.x,
        screen.y,
        screen.w,
        screen.h,
        Color::new(0.006, 0.014, 0.016, 0.88),
    );

    let content = Rect::new(
        (screen.w - 520.0).max(24.0) * 0.5,
        78.0,
        520.0_f32.min(screen.w - 48.0),
        screen.h - 156.0,
    );
    draw_text_centered_in_box_ex(
        &ctx.data.text("ui.settings"),
        content.x,
        content.y,
        content.w,
        48.0,
        TextStyle::new(34.0, style::TEXT_BRIGHT),
    );
    style::draw_divider(content.x, content.y + 72.0, content.w);

    let toggle_rect = Rect::new(content.x + 80.0, content.y + 122.0, content.w - 160.0, 48.0);
    let toggle_text = if ctx.fullscreen {
        ctx.data.text("ui.fullscreen_on")
    } else {
        ctx.data.text("ui.fullscreen_off")
    };
    if virtual_button(
        toggle_rect,
        &toggle_text,
        true,
        ButtonTone::Primary,
        ctx.pointer,
    ) {
        actions.push(UiAction::ToggleFullscreen);
    }

    let back_rect = Rect::new(
        content.x + 130.0,
        content.bottom() - 74.0,
        content.w - 260.0,
        44.0,
    );
    if virtual_button(
        back_rect,
        &ctx.data.text("ui.back"),
        true,
        ButtonTone::Secondary,
        ctx.pointer,
    ) {
        actions.push(UiAction::CloseSettings);
    }

    actions
}

pub(super) fn draw_pause_menu(ctx: PauseMenuContext<'_>) -> Vec<UiAction> {
    let mut actions = Vec::new();
    let screen = Rect::new(0.0, 0.0, ctx.ui.logical_width, ctx.ui.logical_height);

    draw_rectangle(
        screen.x,
        screen.y,
        screen.w,
        screen.h,
        Color::new(0.006, 0.012, 0.014, 0.70),
    );

    let menu = pause_menu_rect(screen);
    style::draw_panel(menu);
    draw_text_centered_in_box_ex(
        &ctx.data.text("ui.paused"),
        menu.x,
        menu.y + 20.0,
        menu.w,
        42.0,
        TextStyle::new(31.0, style::TEXT_BRIGHT),
    );
    style::draw_divider(menu.x + 28.0, menu.y + 76.0, menu.w - 56.0);

    if ctx.pending_exit_warning.is_none() {
        draw_pause_buttons(
            ctx.data,
            menu.inset(30.0),
            ctx.pointer,
            ctx.save_exists,
            &mut actions,
        );
    }

    if let Some(target) = ctx.pending_exit_warning {
        draw_exit_warning(ctx.data, screen, ctx.pointer, target, &mut actions);
    }

    actions
}

fn draw_title_background(screen: Rect, texture: Option<&Texture2D>) {
    if let Some(texture) = texture {
        let tex_w = texture.width().max(1.0);
        let tex_h = texture.height().max(1.0);
        let screen_aspect = screen.w / screen.h.max(1.0);
        let tex_aspect = tex_w / tex_h;
        let source = if tex_aspect > screen_aspect {
            let crop_w = tex_h * screen_aspect;
            Rect::new((tex_w - crop_w) * 0.5, 0.0, crop_w, tex_h)
        } else {
            let crop_h = tex_w / screen_aspect.max(0.01);
            Rect::new(0.0, (tex_h - crop_h) * 0.5, tex_w, crop_h)
        };
        draw_texture_ex(
            texture,
            screen.x,
            screen.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen.w, screen.h)),
                source: Some(source),
                ..Default::default()
            },
        );
    } else {
        clear_background(style::INK);
    }
}

fn draw_title_vignette(screen: Rect) {
    draw_rectangle(
        screen.x,
        screen.y,
        screen.w,
        screen.h,
        Color::new(0.0, 0.0, 0.0, 0.32),
    );
    draw_rectangle(
        screen.x,
        screen.y,
        screen.w,
        screen.h,
        Color::new(0.012, 0.022, 0.020, 0.22),
    );
}

fn menu_rect(screen: Rect) -> Rect {
    let (button_h, gap) = menu_metrics(screen);
    let width = (screen.w * 0.42).clamp(300.0, 420.0);
    let height = button_h * 4.0 + gap * 3.0;
    let bottom_margin = (screen.h * 0.03).clamp(14.0, 32.0);
    Rect::new(
        screen.x + (screen.w - width) * 0.5,
        screen.bottom() - height - bottom_margin,
        width,
        height,
    )
}

fn menu_metrics(screen: Rect) -> (f32, f32) {
    if screen.h < 560.0 {
        (31.0, 5.0)
    } else {
        (42.0, 10.0)
    }
}

fn draw_menu_buttons(
    data: &GameData,
    rect: Rect,
    pointer: Pointer,
    save_exists: bool,
    actions: &mut Vec<UiAction>,
) {
    let button_h = if rect.h < 170.0 { 31.0 } else { 42.0 };
    let gap = if rect.h < 170.0 { 5.0 } else { 10.0 };
    let mut y = rect.y;
    if virtual_button(
        Rect::new(rect.x, y, rect.w, button_h),
        &data.text("ui.new_game"),
        true,
        ButtonTone::Primary,
        pointer,
    ) {
        actions.push(UiAction::NewGame);
    }
    y += button_h + gap;

    if virtual_button(
        Rect::new(rect.x, y, rect.w, button_h),
        &data.text("ui.continue"),
        save_exists,
        ButtonTone::Secondary,
        pointer,
    ) {
        actions.push(UiAction::ContinueGame);
    }
    y += button_h + gap;

    if virtual_button(
        Rect::new(rect.x, y, rect.w, button_h),
        &data.text("ui.settings"),
        true,
        ButtonTone::Secondary,
        pointer,
    ) {
        actions.push(UiAction::OpenSettings);
    }
    y += button_h + gap;

    if virtual_button(
        Rect::new(rect.x, y, rect.w, button_h),
        &data.text("ui.exit"),
        true,
        ButtonTone::Danger,
        pointer,
    ) {
        actions.push(UiAction::ExitGame);
    }
}

fn pause_menu_rect(screen: Rect) -> Rect {
    let (button_h, gap) = menu_metrics(screen);
    let width = (screen.w * 0.34).clamp(330.0, 430.0);
    let height = 126.0 + button_h * 6.0 + gap * 5.0;
    Rect::new(
        screen.x + (screen.w - width) * 0.5,
        screen.y + (screen.h - height) * 0.5,
        width,
        height,
    )
}

fn draw_pause_buttons(
    data: &GameData,
    rect: Rect,
    pointer: Pointer,
    save_exists: bool,
    actions: &mut Vec<UiAction>,
) {
    let button_h = if rect.h < 310.0 { 31.0 } else { 42.0 };
    let gap = if rect.h < 310.0 { 5.0 } else { 10.0 };
    let mut y = rect.y + 70.0;

    let buttons = [
        (
            data.text("ui.resume"),
            true,
            ButtonTone::Secondary,
            UiAction::ClosePauseMenu,
        ),
        (
            data.text("ui.save"),
            true,
            ButtonTone::Primary,
            UiAction::Save,
        ),
        (
            data.text("ui.load"),
            save_exists,
            ButtonTone::Secondary,
            UiAction::Load,
        ),
        (
            data.text("ui.settings"),
            true,
            ButtonTone::Secondary,
            UiAction::OpenSettings,
        ),
        (
            data.text("ui.title"),
            true,
            ButtonTone::Secondary,
            UiAction::ReturnToTitle,
        ),
        (
            data.text("ui.exit"),
            true,
            ButtonTone::Danger,
            UiAction::ExitGame,
        ),
    ];

    for (label, enabled, tone, action) in buttons {
        if virtual_button(
            Rect::new(rect.x, y, rect.w, button_h),
            &label,
            enabled,
            tone,
            pointer,
        ) {
            actions.push(action);
        }
        y += button_h + gap;
    }
}

fn draw_exit_warning(
    data: &GameData,
    screen: Rect,
    pointer: Pointer,
    target: ExitWarningTarget,
    actions: &mut Vec<UiAction>,
) {
    draw_rectangle(
        screen.x,
        screen.y,
        screen.w,
        screen.h,
        Color::new(0.0, 0.0, 0.0, 0.38),
    );

    let width = (screen.w * 0.44).clamp(390.0, 560.0);
    let rect = Rect::new(
        screen.x + (screen.w - width) * 0.5,
        screen.y + (screen.h - 246.0) * 0.5,
        width,
        246.0,
    );
    style::draw_panel(rect);

    draw_text_centered_in_box_ex(
        &data.text("ui.recent_save_needed"),
        rect.x + 22.0,
        rect.y + 20.0,
        rect.w - 44.0,
        34.0,
        TextStyle::new(24.0, style::TEXT_BRIGHT),
    );
    style::draw_divider(rect.x + 28.0, rect.y + 72.0, rect.w - 56.0);

    let body = match target {
        ExitWarningTarget::Title => data.text("ui.save_warning_title"),
        ExitWarningTarget::ExitGame => data.text("ui.save_warning_exit"),
    };
    draw_text_block(
        &body,
        rect.x + 34.0,
        rect.y + 98.0,
        rect.w - 68.0,
        56.0,
        16.0,
        4.0,
        style::TEXT,
    );

    let button_w = (rect.w - 88.0) / 3.0;
    let y = rect.bottom() - 62.0;
    if virtual_button(
        Rect::new(rect.x + 24.0, y, button_w, 40.0),
        &data.text("ui.save_first"),
        true,
        ButtonTone::Primary,
        pointer,
    ) {
        actions.push(UiAction::SaveAndConfirmPendingExit);
    }
    let anyway_label = match target {
        ExitWarningTarget::Title => data.text("ui.title_anyway"),
        ExitWarningTarget::ExitGame => data.text("ui.exit_anyway"),
    };
    if virtual_button(
        Rect::new(rect.x + 34.0 + button_w, y, button_w, 40.0),
        &anyway_label,
        true,
        ButtonTone::Danger,
        pointer,
    ) {
        actions.push(UiAction::ConfirmPendingExit);
    }
    if virtual_button(
        Rect::new(rect.x + 44.0 + button_w * 2.0, y, button_w, 40.0),
        &data.text("ui.cancel"),
        true,
        ButtonTone::Secondary,
        pointer,
    ) {
        actions.push(UiAction::CancelPendingExit);
    }
}
