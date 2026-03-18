use kanban_planner::App;

fn main() {
    // Launch the Dioxus app.
    // The `dx` CLI determines the platform (web, desktop, or mobile) at compile time.
    dioxus::launch(App);
}
