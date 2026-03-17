use kanban_planner::app::App;

fn main() {
    // Launch the Dioxus app.
    // The `dx` CLI determines the platform (web, desktop, or mobile) at compile time.
    dioxus::launch(App);
}
