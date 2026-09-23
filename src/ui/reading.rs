//! Paged details keep secondary prose out of active decisions.

use super::{style, virtual_button, UiAction, UiContext};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::RectExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadingView {
    Site,
    History,
    Completed,
    Events,
}

struct Entry {
    title: String,
    body: String,
}

pub(super) fn draw_reading(ctx: &UiContext<'_>, actions: &mut Vec<UiAction>) {
    let Some(view) = ctx.reading else {
        return;
    };
    let entries = entries(ctx, view);
    let page = ctx.reading_page.min(entries.len().saturating_sub(1));
    let height = match view {
        ReadingView::Site => 300.0,
        ReadingView::Events | ReadingView::History => 380.0,
        ReadingView::Completed => 380.0,
    };
    let rect = super::centered_modal_rect(ctx, 820.0, height);
    style::draw_panel(rect);
    let content = rect.inset(24.0);
    let heading = match view {
        ReadingView::Site => "ui.details",
        ReadingView::History => "ui.recent_actions",
        ReadingView::Completed => "ui.completed",
        ReadingView::Events => "ui.event_details",
    };
    style::draw_panel_title(&ctx.data.text(heading), content.x, content.y + 22.0);
    if virtual_button(
        Rect::new(content.right() - 86.0, content.y, 86.0, 32.0),
        &ctx.data.text("ui.close"),
        true,
        ButtonTone::Secondary,
        ctx.pointer,
    ) {
        actions.push(UiAction::CloseReading);
    }
    if let Some(entry) = entries.get(page) {
        draw_text_block(
            &entry.title,
            content.x,
            content.y + 60.0,
            content.w,
            50.0,
            23.0,
            3.0,
            style::TEXT_BRIGHT,
        );
        draw_text_block(
            &entry.body,
            content.x,
            content.y + 122.0,
            content.w,
            content.h - 182.0,
            17.0,
            4.0,
            style::TEXT,
        );
    }
    if entries.len() > 1 {
        let y = content.bottom() - 34.0;
        if virtual_button(
            Rect::new(content.x, y, 116.0, 32.0),
            &ctx.data.text("ui.previous"),
            page > 0,
            ButtonTone::Secondary,
            ctx.pointer,
        ) {
            actions.push(UiAction::ReadingPage(page - 1));
        }
        draw_text_centered_in_box_ex(
            &ctx.data.text_with(
                "ui.reading_page",
                &[
                    ("{page}", &(page + 1).to_string()),
                    ("{pages}", &entries.len().to_string()),
                ],
            ),
            content.x + 128.0,
            y,
            content.w - 256.0,
            32.0,
            TextStyle::new(14.0, style::TEXT_DIM),
        );
        if virtual_button(
            Rect::new(content.right() - 116.0, y, 116.0, 32.0),
            &ctx.data.text("ui.next"),
            page + 1 < entries.len(),
            ButtonTone::Secondary,
            ctx.pointer,
        ) {
            actions.push(UiAction::ReadingPage(page + 1));
        }
    }
}

fn entries(ctx: &UiContext<'_>, view: ReadingView) -> Vec<Entry> {
    match view {
        ReadingView::Site => ctx
            .session
            .selected_site(ctx.data)
            .filter(|site| ctx.session.is_known(&site.id))
            .map(|site| {
                vec![Entry {
                    title: site.name.clone(),
                    body: site.description.clone(),
                }]
            })
            .unwrap_or_default(),
        ReadingView::Events => vec![Entry {
            title: ctx.data.text("ui.council_event"),
            body: ctx.data.text("ui.event_help"),
        }],
        ReadingView::History => ctx
            .session
            .rival_faction
            .action_log
            .iter()
            .rev()
            .map(|entry| {
                let target = entry
                    .target_site_id
                    .as_deref()
                    .and_then(|id| ctx.data.site(id))
                    .map(|site| site.name.clone())
                    .unwrap_or_else(|| ctx.data.text("state.frontier"));
                Entry {
                    title: ctx.session.rival_faction.name.clone(),
                    body: ctx.data.text_with(
                        "ui.rival_action_log",
                        &[
                            ("{season}", entry.season.label()),
                            ("{year}", &entry.year.to_string()),
                            ("{action}", &entry.action),
                            ("{target}", &target),
                            ("{reason}", &entry.reason),
                            ("{result}", &entry.result),
                        ],
                    ),
                }
            })
            .collect(),
        ReadingView::Completed => completed_entries(ctx),
    }
}

fn completed_entries(ctx: &UiContext<'_>) -> Vec<Entry> {
    let mut result = Vec::new();
    for done in &ctx.session.completed_projects {
        let Some(project) = ctx.data.campaign_balance.project(&done.id) else {
            continue;
        };
        let target = ctx
            .data
            .site(&done.target_id)
            .map(|site| site.name.clone())
            .or_else(|| {
                ctx.data
                    .region(&done.target_id)
                    .map(|region| region.name.clone())
            })
            .unwrap_or_else(|| done.target_id.clone());
        let applied = project_effects(ctx, &project.effects);
        result.push(Entry {
            title: project.name.clone(),
            body: ctx.data.text_with(
                "ui.completed_entry",
                &[
                    ("{target}", &target),
                    ("{year}", &done.completed_year.to_string()),
                    (
                        "{description}",
                        &format!("{}\n{}", project.description, applied),
                    ),
                ],
            ),
        });
    }
    for active in &ctx.session.active_institutions {
        if let Some(institution) = ctx.data.campaign_balance.institution(&active.id) {
            result.push(Entry {
                title: institution.name.clone(),
                body: institution.description.clone(),
            });
        }
    }
    result
}

fn project_effects(ctx: &UiContext<'_>, effects: &crate::data::ProjectEffectDef) -> String {
    let changes = [
        ("ui.food", effects.resource_delta.food),
        ("ui.timber", effects.resource_delta.timber),
        ("ui.stone", effects.resource_delta.stone),
        ("ui.wealth", effects.resource_delta.wealth),
        ("ui.prosperity", effects.prosperity_delta),
        ("ui.stability", effects.stability_delta),
        ("ui.loyalty", effects.loyalty_delta),
        ("ui.danger", effects.danger_delta),
        ("ui.wilderness", effects.wilderness_pressure_delta),
    ]
    .into_iter()
    .filter(|(_, value)| *value != 0)
    .map(|(label, value)| {
        ctx.data.text_with(
            "ui.output_resource",
            &[
                ("{amount}", &format!("{value:+}")),
                ("{resource}", &ctx.data.text(label)),
            ],
        )
    })
    .collect::<Vec<_>>()
    .join(", ");
    ctx.data
        .text_with("ui.effects_achieved", &[("{changes}", &changes)])
}
