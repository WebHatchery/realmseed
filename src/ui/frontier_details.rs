//! Reachable route and active-issue details for the selected frontier site.

use super::{
    inspectable_button, style, virtual_button, ActionReview, FrontierDetailsTab, UiAction,
    UiContext,
};
use crate::data::{ActiveIssueState, RouteLevel};
use crate::state::RouteRuntimeState;
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;
use macroquad_toolkit::ui::RectExt;

pub(super) fn draw_frontier_details(ctx: &UiContext<'_>, actions: &mut Vec<UiAction>) {
    let rect = super::centered_modal_rect(ctx, 820.0, 580.0);
    draw_rectangle(
        0.0,
        0.0,
        ctx.ui.logical_width,
        ctx.ui.logical_height,
        Color::new(0.004, 0.010, 0.012, 0.74),
    );
    style::draw_panel(rect);
    let content = rect.inset(24.0);
    style::draw_panel_title(
        &ctx.data.text("ui.frontier_details"),
        content.x,
        content.y + 16.0,
    );
    draw_ui_text_ex(
        &selected_site_title(ctx),
        content.x,
        content.y + 52.0,
        TextStyle::new(24.0, style::TEXT_BRIGHT).params(),
    );
    if virtual_button(
        Rect::new(content.right() - 86.0, content.y + 2.0, 86.0, 32.0),
        &ctx.data.text("ui.close"),
        ctx.action_review.is_none(),
        ButtonTone::Secondary,
        ctx.pointer,
    ) {
        actions.push(UiAction::ToggleFrontierDetails);
    }
    style::draw_divider(content.x, content.y + 76.0, content.w);

    let tab_y = content.y + 90.0;
    let tab_w = (content.w - 8.0) * 0.5;
    if tab_button(
        ctx,
        Rect::new(content.x, tab_y, tab_w, 32.0),
        &ctx.data.text("ui.routes_tab"),
        FrontierDetailsTab::Routes,
    ) {
        actions.push(UiAction::SetFrontierDetailsTab(FrontierDetailsTab::Routes));
    }
    if tab_button(
        ctx,
        Rect::new(content.x + tab_w + 8.0, tab_y, tab_w, 32.0),
        &ctx.data.text("ui.issues_tab"),
        FrontierDetailsTab::Issues,
    ) {
        actions.push(UiAction::SetFrontierDetailsTab(FrontierDetailsTab::Issues));
    }

    let body = Rect::new(
        content.x,
        tab_y + 48.0,
        content.w,
        content.bottom() - tab_y - 48.0,
    );
    match ctx.frontier_details_tab {
        FrontierDetailsTab::Routes => draw_routes(ctx, actions, body),
        FrontierDetailsTab::Issues => draw_issues(ctx, body),
    }
}

fn selected_site_title(ctx: &UiContext<'_>) -> String {
    ctx.session
        .selected_site(ctx.data)
        .map(|site| site.name.clone())
        .unwrap_or_else(|| ctx.data.text("ui.no_site"))
}

fn tab_button(ctx: &UiContext<'_>, rect: Rect, label: &str, tab: FrontierDetailsTab) -> bool {
    virtual_button(
        rect,
        label,
        ctx.action_review.is_none(),
        if ctx.frontier_details_tab == tab {
            ButtonTone::Primary
        } else {
            ButtonTone::Secondary
        },
        ctx.pointer,
    )
}

fn draw_routes(ctx: &UiContext<'_>, actions: &mut Vec<UiAction>, body: Rect) {
    let Some(site) = ctx.session.selected_site(ctx.data) else {
        draw_text_block(
            &ctx.data.text("ui.no_site"),
            body.x,
            body.y + 24.0,
            body.w,
            30.0,
            16.0,
            2.0,
            style::TEXT_DIM,
        );
        return;
    };
    let routes: Vec<&RouteRuntimeState> = ctx
        .session
        .routes_for_site(&site.id)
        .filter(|route| route.known)
        .collect();
    if routes.is_empty() {
        draw_text_block(
            &ctx.data.text("ui.no_known_routes"),
            body.x,
            body.y + 24.0,
            body.w,
            30.0,
            16.0,
            2.0,
            style::TEXT_DIM,
        );
        return;
    }
    let project_status = ctx
        .session
        .regional_project_status(ctx.data, &site.region_id);
    let project_rect = Rect::new(body.right() - 166.0, body.y + 2.0, 166.0, 30.0);
    if inspectable_button(
        project_rect,
        &ctx.data.text("ui.wardens"),
        project_status.enabled,
        ctx.action_review.is_none(),
        ButtonTone::Secondary,
        ctx.pointer,
    ) {
        actions.push(UiAction::OpenActionReview(
            ActionReview::CompleteRegionalProject(site.region_id.clone()),
        ));
    }
    draw_text_block(
        &ctx.data.text("ui.route_details_hint"),
        body.x,
        body.y + 12.0,
        body.w - 180.0,
        28.0,
        13.0,
        2.0,
        style::TEXT_DIM,
    );
    let mut y = body.y + 42.0;
    for route in routes {
        draw_route_detail(ctx, actions, body, y, route, &site.id);
        y += 48.0;
    }
}

fn draw_route_detail(
    ctx: &UiContext<'_>,
    actions: &mut Vec<UiAction>,
    body: Rect,
    y: f32,
    route: &RouteRuntimeState,
    selected_site_id: &str,
) {
    let row = Rect::new(body.x, y, body.w, 42.0);
    draw_surface(
        row,
        &SurfaceStyle::new(Color::new(0.018, 0.032, 0.034, 0.70))
            .with_border(1.0, Color::new(0.58, 0.45, 0.25, 0.18)),
    );
    let other_name = route
        .other_end(selected_site_id)
        .and_then(|site_id| ctx.data.site(site_id))
        .map(|site| site.name.clone())
        .unwrap_or_else(|| ctx.data.text("ui.unknown"));
    let status = ctx.session.route_action_status(ctx.data, &route.id);
    let label = ctx.session.route_action_label(ctx.data, &route.id);
    draw_ui_text_ex(
        &other_name,
        row.x + 12.0,
        row.y + 17.0,
        TextStyle::new(14.0, style::TEXT_BRIGHT).params(),
    );
    draw_ui_text_ex(
        &format_route_status(ctx, route),
        row.x + 12.0,
        row.y + 34.0,
        TextStyle::new(11.5, style::TEXT_DIM).params(),
    );
    let action_rect = Rect::new(row.right() - 178.0, row.y + 6.0, 166.0, 30.0);
    if inspectable_button(
        action_rect,
        &label,
        status.enabled,
        ctx.action_review.is_none(),
        ButtonTone::Secondary,
        ctx.pointer,
    ) {
        actions.push(UiAction::OpenActionReview(
            ActionReview::BuildOrUpgradeRoute(route.id.clone()),
        ));
    }
}

fn format_route_status(ctx: &UiContext<'_>, route: &RouteRuntimeState) -> String {
    let level = match route.level {
        RouteLevel::None => ctx.data.text("ui.route_none"),
        RouteLevel::Path => ctx.data.text("ui.route_path"),
        RouteLevel::Road => ctx.data.text("ui.route_road"),
        RouteLevel::StoneRoad => ctx.data.text("ui.route_stone"),
    };
    let condition = match route.condition {
        crate::state::RouteCondition::Clear => ctx.data.text("ui.route_good"),
        crate::state::RouteCondition::Damaged => ctx.data.text("ui.route_damaged"),
        crate::state::RouteCondition::Blocked => ctx.data.text("ui.route_blocked"),
    };
    format!("{level} · {condition}")
}

fn draw_issues(ctx: &UiContext<'_>, body: Rect) {
    let issues: Vec<_> = ctx
        .session
        .active_issues
        .iter()
        .filter(|issue| {
            issue.target_site_id == ctx.session.selected_site_id
                && !matches!(
                    issue.state,
                    ActiveIssueState::Dormant | ActiveIssueState::Resolution
                )
        })
        .collect();
    if issues.is_empty() {
        draw_text_block(
            &ctx.data.text("ui.no_active_issues"),
            body.x,
            body.y + 24.0,
            body.w,
            30.0,
            16.0,
            2.0,
            style::TEXT_DIM,
        );
        return;
    }
    draw_text_block(
        &ctx.data.text("ui.issue_details_hint"),
        body.x,
        body.y + 12.0,
        body.w,
        28.0,
        13.0,
        2.0,
        style::TEXT_DIM,
    );
    let mut y = body.y + 42.0;
    for issue in issues {
        let row = Rect::new(body.x, y, body.w, 58.0);
        draw_surface(
            row,
            &SurfaceStyle::new(Color::new(0.05, 0.025, 0.026, 0.72))
                .with_border(1.0, Color::new(0.70, 0.28, 0.22, 0.35)),
        );
        let state = issue_state_label(ctx, issue.state);
        let trigger = ctx
            .data
            .event_family(&issue.family_id)
            .map(|family| family.trigger_kind.replace('_', " "))
            .unwrap_or_else(|| ctx.data.text("ui.unknown"));
        let detail = ctx.data.text_with(
            "ui.issue_detail",
            &[
                ("{state}", &state),
                ("{severity}", &issue.severity.to_string()),
                ("{age}", &issue.age_seasons.to_string()),
                ("{cause}", &trigger),
            ],
        );
        draw_ui_text_ex(
            &ctx.data.text("ui.active_issue"),
            row.x + 12.0,
            row.y + 18.0,
            TextStyle::new(14.0, style::RED).params(),
        );
        draw_text_block(
            &detail,
            row.x + 12.0,
            row.y + 24.0,
            row.w - 24.0,
            28.0,
            12.0,
            2.0,
            style::TEXT,
        );
        if ctx
            .session
            .pending_event
            .as_ref()
            .is_some_and(|pending| pending.issue_id == issue.id)
        {
            draw_ui_text_ex(
                &ctx.data.text("ui.pending_event"),
                row.right() - 132.0,
                row.y + 18.0,
                TextStyle::new(12.0, style::GOLD).params(),
            );
        }
        y += 64.0;
    }
}

fn issue_state_label(ctx: &UiContext<'_>, state: ActiveIssueState) -> String {
    let key = match state {
        ActiveIssueState::Warning => "ui.issue_warning",
        ActiveIssueState::Active => "ui.issue_active",
        ActiveIssueState::Escalating => "ui.issue_escalating",
        ActiveIssueState::Resolution => "ui.issue_resolution",
        ActiveIssueState::Dormant => "ui.issue_dormant",
        ActiveIssueState::Collapse => "ui.issue_collapse",
    };
    ctx.data.text(key)
}
