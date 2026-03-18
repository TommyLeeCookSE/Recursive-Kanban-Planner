use crate::domain::id::CardId;
use crate::interface::components::layout::NavbarLayout;
use crate::interface::routes::board::Board;
use crate::interface::routes::home::Home;
use dioxus::prelude::*;

pub mod board;
pub mod home;

/// The top-level application routes used by Dioxus Router.
///
/// # Examples
///
/// ```rust
/// use kanban_planner::interface::Route;
///
/// let route = Route::Home {};
/// assert!(matches!(route, Route::Home {}));
/// ```
#[derive(Clone, Routable, Debug, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(NavbarLayout)]
        #[route("/")]
        Home {},
        #[route("/board/:card_id")]
        Board { card_id: CardId },
}
