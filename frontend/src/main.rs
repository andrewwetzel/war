mod components;
mod models;

use yew::Renderer;
use components::table::Table;

fn main() {
    Renderer::<Table>::new().render();
}
