//! Pure advisor-priority queries used by the council guidance surface.

use super::{GameSession, SettlementStatus};
use crate::data::{GameData, RouteLevel, SiteCategory};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdvisorPriority {
    AnswerEvent,
    SpendActions,
    ScoutSelected,
    RevealFrontier,
    FoundCamp,
    BuildRoad,
    AdvanceSeason,
    ResolveIssues,
}

impl GameSession {
    pub fn advisor_priority(&self, data: &GameData) -> AdvisorPriority {
        if self.pending_event.is_some() {
            return AdvisorPriority::AnswerEvent;
        }
        if self.council_actions_remaining <= 0 {
            return AdvisorPriority::SpendActions;
        }
        if self.is_adjacent_unknown(data, &self.selected_site_id) {
            return AdvisorPriority::ScoutSelected;
        }
        if data
            .sites
            .iter()
            .any(|site| self.is_adjacent_unknown(data, &site.id))
        {
            return AdvisorPriority::RevealFrontier;
        }
        if self
            .settlements
            .iter()
            .filter(|settlement| settlement.status == SettlementStatus::Active)
            .count()
            < 2
            && data.sites.iter().any(|site| {
                site.category == SiteCategory::Settlement
                    && self.is_known(&site.id)
                    && site.owner.is_none()
                    && self.settlement_at_site(&site.id).is_none()
            })
        {
            return AdvisorPriority::FoundCamp;
        }
        if self.routes.iter().any(|route| {
            route.known
                && route.level == RouteLevel::None
                && self.route_action_status(data, &route.id).enabled
        }) {
            return AdvisorPriority::BuildRoad;
        }
        if self.active_issues.is_empty() {
            AdvisorPriority::AdvanceSeason
        } else {
            AdvisorPriority::ResolveIssues
        }
    }
}
