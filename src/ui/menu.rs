//! Title menu and settings screen rendering.

use super::{style, virtual_button, MenuContext, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;

pub(super) fn draw_title_menu(ctx: MenuContext<'_>) -> Vec<UiAction> {
    let mut actions = Vec::new();
    let mouse = ctx.ui.mouse_position();
    let screen = Rect::new(0.0, 0.0, ctx.ui.logical_width, ctx.ui.logical_height);

    draw_title_background(screen, ctx.title_texture);
    draw_title_vignette(screen);

    let menu = menu_rect(screen);
    draw_menu_buttons(menu, mouse, ctx.save_exists, &mut actions);

    actions
}

pub(super) fn draw_settings_page(ctx: MenuContext<'_>) -> Vec<UiAction> {
    let mut actions = Vec::new();
    let mouse = ctx.ui.mouse_position();
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
        "Settings",
        content.x,
        content.y,
        content.w,
        48.0,
        TextStyle::new(34.0, style::TEXT_BRIGHT),
    );
    style::draw_divider(content.x, content.y + 72.0, content.w);

    let toggle_rect = Rect::new(content.x + 80.0, content.y + 122.0, content.w - 160.0, 48.0);
    let toggle_text = if ctx.fullscreen {
        "Fullscreen: On"
    } else {
        "Fullscreen: Off"
    };
    if virtual_button(toggle_rect, toggle_text, true, ButtonTone::Primary, mouse) {
        actions.push(UiAction::ToggleFullscreen);
    }

    let back_rect = Rect::new(
        content.x + 130.0,
        content.bottom() - 74.0,
        content.w - 260.0,
        44.0,
    );
    if virtual_button(back_rect, "Back", true, ButtonTone::Secondary, mouse) {
        actions.push(UiAction::CloseSettings);
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

fn draw_menu_buttons(rect: Rect, mouse: Vec2, save_exists: bool, actions: &mut Vec<UiAction>) {
    let button_h = if rect.h < 170.0 { 31.0 } else { 42.0 };
    let gap = if rect.h < 170.0 { 5.0 } else { 10.0 };
    let mut y = rect.y;
    if virtual_button(
        Rect::new(rect.x, y, rect.w, button_h),
        "New Game",
        true,
        ButtonTone::Primary,
        mouse,
    ) {
        actions.push(UiAction::NewGame);
    }
    y += button_h + gap;

    if virtual_button(
        Rect::new(rect.x, y, rect.w, button_h),
        "Continue",
        save_exists,
        ButtonTone::Secondary,
        mouse,
    ) {
        actions.push(UiAction::ContinueGame);
    }
    y += button_h + gap;

    if virtual_button(
        Rect::new(rect.x, y, rect.w, button_h),
        "Settings",
        true,
        ButtonTone::Secondary,
        mouse,
    ) {
        actions.push(UiAction::OpenSettings);
    }
    y += button_h + gap;

    if virtual_button(
        Rect::new(rect.x, y, rect.w, button_h),
        "Exit Game",
        true,
        ButtonTone::Danger,
        mouse,
    ) {
        actions.push(UiAction::ExitGame);
    }
}
