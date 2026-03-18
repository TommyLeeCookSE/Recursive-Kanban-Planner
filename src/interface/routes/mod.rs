use crate::domain::id::CardId;
use crate::interface::components::layout::NavbarLayout;
use crate::interface::routes::board::Board;
use crate::interface::routes::home::Home;
use dioxus::prelude::*;

pub mod board;
pub mod home;

#[derive(Clone, Routable, Debug, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(NavbarLayout)]
        #[route("/")]
        Home {},
        #[route("/board/:card_id")]
        Board { card_id: CardId },
}
