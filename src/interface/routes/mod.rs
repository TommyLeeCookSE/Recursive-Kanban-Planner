use crate::domain::id::CardId;
use crate::interface::components::layout::NavbarLayout;
use crate::interface::routes::board::Board;
use crate::interface::routes::home::Home;
use crate::interface::routes::map_screen::MapScreen;
use dioxus::prelude::*;

pub mod board;
pub(crate) mod board_screen;
pub mod home;
pub mod map_screen;

/// The top-level application routes used by Dioxus Router.
#[derive(Clone, Routable, Debug, PartialEq, Eq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(NavbarLayout)]
    #[route("/", Home)]
    Home {},
    #[route("/board/:card_id", Board)]
    Board { card_id: CardId },
    #[route("/map/:focus_card_id", MapScreen)]
    Map { focus_card_id: CardId },
}
