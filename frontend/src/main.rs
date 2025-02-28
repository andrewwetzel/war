use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use serde::Deserialize;

#[derive(Deserialize, Clone, PartialEq)]
struct TableData {
    id: usize,
    name: String,
    email: String,
    role: String,
}

#[function_component(App)]
fn app() -> Html {
    // State to hold fetched data
    let table_data = use_state(|| Vec::<TableData>::new());

    {
        let table_data = table_data.clone();
        use_effect_with_deps(move |_| {
            // Spawn a local future to fetch the table data
            spawn_local(async move {
                match reqwest::get("http://127.0.0.1:9998/table-data").await {
                    Ok(resp) => {
                        if let Ok(data) = resp.json::<Vec<TableData>>().await {
                            table_data.set(data);
                        }
                    }
                    Err(err) => {
                        log::error!("Failed to fetch table data: {}", err);
                    }
                }
            });
            || ()
        }, ());
    }

    html! {
        <div class="container">
            <h1>{"Simple Table with Yew & Backend"}</h1>
            <table class="table">
                <thead>
                    <tr>
                        <th>{"ID"}</th>
                        <th>{"Name"}</th>
                        <th>{"Email"}</th>
                        <th>{"Role"}</th>
                    </tr>
                </thead>
                <tbody>
                    {
                        // Render rows from the fetched data
                        table_data.iter().map(|row| html! {
                            <tr key={row.id}>
                                <td>{row.id}</td>
                                <td>{&row.name}</td>
                                <td>{&row.email}</td>
                                <td>{&row.role}</td>
                            </tr>
                        }).collect::<Html>()
                    }
                </tbody>
            </table>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
