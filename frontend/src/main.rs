use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use serde::Deserialize;
use gloo_net::http::Request;

#[derive(Deserialize, Clone, PartialEq)]
struct TableData {
    id: usize,
    name: String,
    email: String,
    role: String,
}

#[derive(Clone, PartialEq)]
enum SortColumn {
    Id,
    Name,
    Email,
    Role,
}

#[derive(Clone, PartialEq)]
enum SortOrder {
    Ascending,
    Descending,
}

#[function_component(App)]
fn app() -> Html {
    // State to hold fetched data
    let table_data = use_state(|| Vec::<TableData>::new());
    // State to track current sort settings: (column, order)
    let sort_state = use_state(|| None as Option<(SortColumn, SortOrder)>);

    // Fetch data only once.
    {
        let table_data = table_data.clone();
        use_effect_with_deps(move |_| {
            spawn_local(async move {
                let result = Request::get("http://127.0.0.1:9998/table-data").send().await;
                match result {
                    Ok(response) => {
                        match response.json::<Vec<TableData>>().await {
                            Ok(data) => table_data.set(data),
                            Err(err) => log::error!("Failed to parse JSON: {:?}", err),
                        }
                    }
                    Err(err) => {
                        log::error!("Failed to fetch table data: {:?}", err);
                    }
                }
            });
            || ()
        }, ());
    }

    // Create a sorted version of the data
    let mut sorted_data = (*table_data).clone();
    if let Some((ref col, ref order)) = *sort_state {
        sorted_data.sort_by(|a, b| {
            let cmp = match col {
                SortColumn::Id => a.id.cmp(&b.id),
                SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SortColumn::Email => a.email.to_lowercase().cmp(&b.email.to_lowercase()),
                SortColumn::Role => a.role.to_lowercase().cmp(&b.role.to_lowercase()),
            };
            match order {
                SortOrder::Ascending => cmp,
                SortOrder::Descending => cmp.reverse(),
            }
        });
    }

    // Helper to update sort state when header is clicked.
    let on_header_click = {
        let sort_state = sort_state.clone();
        move |col: SortColumn| {
            let new_state = if let Some((current_col, current_order)) = &*sort_state {
                if *current_col == col {
                    // Toggle order if same column is clicked.
                    Some((col.clone(), match current_order {
                        SortOrder::Ascending => SortOrder::Descending,
                        SortOrder::Descending => SortOrder::Ascending,
                    }))
                } else {
                    // Change to new column, default to ascending.
                    Some((col.clone(), SortOrder::Ascending))
                }
            } else {
                // No sort state set, default to ascending.
                Some((col.clone(), SortOrder::Ascending))
            };
            sort_state.set(new_state);
        }
    };

    // Helper to render the header with a clickable sort arrow.
    let render_header = |title: &str, col: SortColumn| {
        let arrow = if let Some((ref current_col, ref order)) = *sort_state {
            if *current_col == col {
                match order {
                    SortOrder::Ascending => " ▲",
                    SortOrder::Descending => " ▼",
                }
            } else {
                ""
            }
        } else {
            ""
        };
        html! {
            <th onclick={Callback::from({
                let col = col.clone();
                let on_header_click = on_header_click.clone();
                move |_| on_header_click(col.clone())
            })}>
                { format!("{}{}", title, arrow) }
            </th>
        }
    };

    html! {
        <div class="container">
            <h1>{ "Simple Table with Yew & Backend" }</h1>
            <table class="table">
                <thead>
                    <tr>
                        { render_header("ID", SortColumn::Id) }
                        { render_header("Name", SortColumn::Name) }
                        { render_header("Email", SortColumn::Email) }
                        { render_header("Role", SortColumn::Role) }
                    </tr>
                </thead>
                <tbody>
                    {
                        sorted_data.iter().map(|row| html! {
                            <tr key={row.id}>
                                <td>{ row.id }</td>
                                <td>{ &row.name }</td>
                                <td>{ &row.email }</td>
                                <td>{ &row.role }</td>
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
