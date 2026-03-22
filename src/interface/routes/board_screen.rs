use crate::application::build_card_view;
use crate::domain::error::DomainError;
use crate::domain::id::CardId;
use crate::domain::registry::CardRegistry;
use crate::interface::Route;
use crate::interface::components::visuals::{CardDisplayData, build_card_display};

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct BoardScreenData {
    pub board_id: CardId,
    pub board_title: String,
    pub board_due_date: String,
    pub back_route: Option<Route>,
    pub back_label: String,
    pub child_cards: Vec<CardDisplayData>,
}

struct BackNavigation {
    route: Option<Route>,
    label: String,
}

pub(crate) fn load_workspace_screen_data(
    registry: &CardRegistry,
) -> Result<BoardScreenData, DomainError> {
    let workspace_id = registry.workspace_card_id()?;
    build_screen_data(workspace_id, registry, None)
}

pub(crate) fn load_board_screen_data(
    card_id: CardId,
    registry: &CardRegistry,
) -> Result<BoardScreenData, DomainError> {
    let back_navigation = build_back_navigation(card_id, registry)?;
    build_screen_data(card_id, registry, Some(back_navigation))
}

fn build_screen_data(
    board_id: CardId,
    registry: &CardRegistry,
    back_navigation: Option<BackNavigation>,
) -> Result<BoardScreenData, DomainError> {
    let view = build_card_view(board_id, registry)?;
    let board_display = build_card_display(view.card, None);
    let board_due_date = board_display
        .due_date
        .as_deref()
        .unwrap_or("None")
        .to_string();
    let child_cards = view
        .children
        .iter()
        .map(|card| {
            let preview_view = build_card_view(card.id(), registry)?;
            Ok(build_card_display(card, Some(&preview_view)))
        })
        .collect::<Result<Vec<_>, DomainError>>()?;

    let back_navigation = back_navigation.unwrap_or(BackNavigation {
        route: None,
        label: view.card.title().to_string(),
    });

    Ok(BoardScreenData {
        board_id,
        board_title: view.card.title().to_string(),
        board_due_date,
        back_route: back_navigation.route,
        back_label: back_navigation.label,
        child_cards,
    })
}

fn build_back_navigation(
    card_id: CardId,
    registry: &CardRegistry,
) -> Result<BackNavigation, DomainError> {
    let card = registry.get_card(card_id)?;
    match card.parent_id() {
        Some(parent_id) => {
            let parent = registry.get_card(parent_id)?;
            Ok(BackNavigation {
                route: Some(Route::Board { card_id: parent_id }),
                label: parent.title().to_string(),
            })
        }
        None => Ok(BackNavigation {
            route: None,
            label: card.title().to_string(),
        }),
    }
}
