use yew::prelude::*;

// Define our data structure for the table rows
struct TableData {
    id: usize,
    name: String,
    email: String,
    role: String,
}

// Simple app component
#[function_component(App)]
fn app() -> Html {
    // Sample data for the table
    let data: Vec<TableData> = vec![
        TableData {
            id: 1,
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            role: "Developer".to_string(),
        },
        TableData {
            id: 2,
            name: "Jane Smith".to_string(),
            email: "jane@example.com".to_string(),
            role: "Designer".to_string(),
        },
        TableData {
            id: 3,
            name: "Bob Johnson".to_string(),
            email: "bob@example.com".to_string(),
            role: "Manager".to_string(),
        },
    ];

    html! {
        <div class="container">
            <h1>{"Simple Table with Yew"}</h1>
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
                    { data.iter().map(|row| html! {
                        <tr key={row.id}>
                            <td>{row.id}</td>
                            <td>{&row.name}</td>
                            <td>{&row.email}</td>
                            <td>{&row.role}</td>
                        </tr>
                    }).collect::<Html>() }
                </tbody>
            </table>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
