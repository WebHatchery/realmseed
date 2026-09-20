//! Event modal rendering and choice intents.

use super::{inspectable_button, style, virtual_button, ActionReview, UiAction, UiContext};
use crate::state::fill_event_text;
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;
use macroquad_toolkit::ui::RectExt;

pub(super) fn draw_event_modal(ctx: &UiContext<'_>, actions: &mut Vec<UiAction>) {
    let Some(pending) = &ctx.session.pending_event else {
        return;
    };
    let Some(template) = ctx.session.pending_event_template(ctx.data) else {
        return;
    };
    let site_name = ctx.session.pending_event_site_name(ctx.data);

    let screen = super::screen_rect(ctx);
    draw_rectangle(
        screen.x,
        screen.y,
        screen.w,
        screen.h,
        Color::new(0.004, 0.010, 0.012, 0.72),
    );

    let rect = super::centered_modal_rect(ctx, 884.0, 548.0);
    style::draw_panel(rect);
    let content = rect.inset(26.0);

    style::draw_framed_icon(
        event_icon(pending.severity),
        vec2(content.x + 28.0, content.y + 26.0),
        48.0,
        Color::new(0.13, 0.08, 0.05, 0.96),
    );
    style::draw_panel_title(
        &ctx.data.text("ui.council_event"),
        content.x + 68.0,
        content.y + 17.0,
    );
    draw_ui_text_ex(
        &fill_event_text(&template.title, &site_name),
        content.x + 68.0,
        content.y + 48.0,
        TextStyle::new(25.0, style::TEXT_BRIGHT).params(),
    );
    draw_severity_badge(
        ctx.data,
        Rect::new(content.right() - 126.0, content.y + 18.0, 126.0, 32.0),
        pending.severity,
    );
    style::draw_divider(content.x, content.y + 76.0, content.w);

    let mut y = content.y + 110.0;
    draw_text_block(
        &fill_event_text(&template.narrative, &site_name),
        content.x,
        y,
        content.w,
        58.0,
        16.0,
        4.0,
        style::TEXT,
    );
    y += 74.0;

    draw_event_detail(
        &ctx.data.text("ui.cause"),
        &pending.cause,
        content.x,
        y,
        content.w,
    );
    y += 42.0;
    draw_event_detail(
        &ctx.data.text("ui.visible"),
        &template.visible_consequences,
        content.x,
        y,
        content.w,
    );
    y += 42.0;
    draw_event_detail(
        &ctx.data.text("ui.uncertain"),
        &template.hidden_consequences,
        content.x,
        y,
        content.w,
    );
    y += 50.0;

    style::draw_divider(content.x, y - 18.0, content.w);

    for choice in &template.choices {
        let status = ctx.session.event_choice_status(ctx.data, choice);
        let button_w = (content.w * 0.32).clamp(220.0, 270.0);
        if inspectable_button(
            Rect::new(content.x, y, button_w, 34.0),
            &choice.label,
            status.enabled,
            ctx.action_review.is_none(),
            ButtonTone::Primary,
            ctx.pointer,
        ) {
            actions.push(UiAction::OpenActionReview(
                ActionReview::ResolveEventChoice(choice.id.clone()),
            ));
        }
        draw_text_block(
            &choice.visible_consequence,
            content.x + button_w + 18.0,
            y + 2.0,
            content.w - button_w - 18.0,
            34.0,
            14.0,
            2.0,
            if status.enabled {
                style::TEXT_DIM
            } else {
                style::RED
            },
        );
        y += 44.0;
    }

    if template.allow_defer
        && virtual_button(
            Rect::new(
                content.right() - 150.0,
                content.bottom() - 34.0,
                150.0,
                32.0,
            ),
            &ctx.data.text("ui.defer"),
            ctx.action_review.is_none(),
            ButtonTone::Secondary,
            ctx.pointer,
        )
    {
        actions.push(UiAction::DeferEvent);
    }
}

fn draw_event_detail(label: &str, text: &str, x: f32, y: f32, width: f32) {
    style::draw_vertical_divider(x + 64.0, y - 18.0, 32.0);
    draw_ui_text_ex(label, x, y, TextStyle::new(13.0, style::GOLD).params());
    draw_text_block(
        text,
        x + 82.0,
        y - 15.0,
        width - 82.0,
        34.0,
        14.0,
        2.0,
        style::TEXT_DIM,
    );
}

fn event_icon(severity: i32) -> style::IconKind {
    if severity >= 2 {
        style::IconKind::Danger
    } else {
        style::IconKind::Compass
    }
}

fn draw_severity_badge(data: &crate::data::GameData, rect: Rect, severity: i32) {
    let tone = if severity >= 3 {
        ButtonTone::Danger
    } else {
        ButtonTone::Warning
    };
    style::draw_button_frame(rect, tone, true, false, false);
    draw_text_centered_in_box_ex(
        &data.text_with("ui.severity", &[("{severity}", &severity.to_string())]),
        rect.x + 8.0,
        rect.y,
        rect.w - 16.0,
        rect.h,
        TextStyle::new(13.0, style::TEXT_BRIGHT),
    );
}
