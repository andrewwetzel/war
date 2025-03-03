use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use gloo_net::http::Request;
use std::collections::{HashMap, HashSet};
use crate::models::{TableData, SortColumn, SortOrder};
use web_sys::{HtmlInputElement, Event};

/// For non-datetime columns, we store a set of allowed values.
/// For the datetime column, we use a (start, end) tuple (both optional).
#[derive(Clone, PartialEq, Debug)]
enum FilterState {
    Values(HashSet<String>),
    DateRange { start: Option<String>, end: Option<String> },
}

#[function_component(Table)]
pub fn table() -> Html {
    // Data state (fetched from backend)
    let table_data = use_state(|| Vec::<TableData>::new());
    // Sorting state: Option<(column, order)>
    let sort_state = use_state(|| None as Option<(SortColumn, SortOrder)>);
    // Filter state: for each column, store its FilterState.
    // We’ll initialize non-datetime columns with an empty set.
    let filters = use_state(|| {
        let mut map = HashMap::new();
        // For the DateTime column we initialize with an empty range filter.
        map.insert(SortColumn::DateTime, FilterState::DateRange { start: None, end: None });
        map
    });
    // Which column's filter popup is currently open.
    let open_filter = use_state(|| None as Option<SortColumn>);

    // Fetch data once.
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

    let mut sorted_data = (*table_data).clone();
    if let Some((col, ref order)) = *sort_state {
        sorted_data.sort_by(|a, b| {
            let cmp = match col {
                SortColumn::Id => a.id.cmp(&b.id),
                SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SortColumn::Email => a.email.to_lowercase().cmp(&b.email.to_lowercase()),
                SortColumn::Role => a.role.to_lowercase().cmp(&b.role.to_lowercase()),
                // Compare ISO8601 strings; this works because they sort chronologically.
                SortColumn::DateTime => a.created_at.cmp(&b.created_at),
            };
            match order {
                SortOrder::Ascending => cmp,
                SortOrder::Descending => cmp.reverse(),
            }
        });
    }

    let filtered_data: Vec<TableData> = sorted_data.into_iter().filter(|row| {
        // Check checkbox filters for non-datetime columns.
        for (col, state) in filters.iter() {
            match col {
                SortColumn::DateTime => { /* handle below */ }
                _ => {
                    if let FilterState::Values(allowed) = state {
                        let value = match col {
                            SortColumn::Id => row.id.to_string(),
                            SortColumn::Name => row.name.clone(),
                            SortColumn::Email => row.email.clone(),
                            SortColumn::Role => row.role.clone(),
                            _ => String::new(),
                        };
                        if !allowed.is_empty() && !allowed.contains(&value) {
                            return false;
                        }
                    }
                }
            }
        }
        // Check date range filter for DateTime column.
        if let Some(FilterState::DateRange { start, end }) = filters.get(&SortColumn::DateTime) {
            let ts = &row.created_at;
            if let Some(s) = start {
                if ts < s {
                    return false;
                }
            }
            if let Some(e) = end {
                if ts > e {
                    return false;
                }
            }
        }
        true
    }).collect();

    // Helper: update sort state on header click.
    let on_header_click = {
        let sort_state = sort_state.clone();
        move |col: SortColumn| {
            let new_state = if let Some((current_col, current_order)) = &*sort_state {
                if *current_col == col {
                    Some((col, match current_order {
                        SortOrder::Ascending => SortOrder::Descending,
                        SortOrder::Descending => SortOrder::Ascending,
                    }))
                } else {
                    Some((col, SortOrder::Ascending))
                }
            } else {
                Some((col, SortOrder::Ascending))
            };
            sort_state.set(new_state);
        }
    };

    // Helper: get all unique values (as strings) for a given column.
    let get_unique_values = |col: &SortColumn| -> Vec<String> {
        let mut set = HashSet::new();
        for row in table_data.iter() {
            let value = match col {
                SortColumn::Id => row.id.to_string(),
                SortColumn::Name => row.name.clone(),
                SortColumn::Email => row.email.clone(),
                SortColumn::Role => row.role.clone(),
                // For DateTime we don’t use a checkbox filter.
                SortColumn::DateTime => row.created_at.clone(),
            };
            set.insert(value);
        }
        let mut vals: Vec<String> = set.into_iter().collect();
        vals.sort();
        vals
    };

    // Render checkbox filter popup for non-datetime columns.
    let render_checkbox_filter = |col: SortColumn| {
        let unique_values = get_unique_values(&col);
        let current_filters = {
            let filters = filters.clone();
            if let Some(FilterState::Values(set)) = (*filters).get(&col) {
                set.clone()
            } else {
                HashSet::new()
            }
        };

        let on_checkbox_change = {
            let filters = filters.clone();
            Callback::from(move |(value, checked): (String, bool)| {
                let mut new_filters = (*filters).clone();
                let entry = new_filters.entry(col).or_insert(FilterState::Values(HashSet::new()));
                if let FilterState::Values(ref mut set) = entry {
                    if checked {
                        set.insert(value);
                    } else {
                        set.remove(&value);
                    }
                }
                filters.set(new_filters);
            })
        };

        html! {
            <div class="filter-popup"
                style="position: absolute; background: white; border: 1px solid #ccc; padding: 8px; width: fit-content; z-index: 10;">
                {
                    unique_values.iter().map(|val| {
                        let is_checked = current_filters.contains(val);
                        let value_clone = val.clone();
                        let on_change = {
                            let on_checkbox_change = on_checkbox_change.clone();
                            Callback::from(move |e: Event| {
                                let input: HtmlInputElement = e.target_unchecked_into();
                                let checked = input.checked();
                                on_checkbox_change.emit((value_clone.clone(), checked));
                            })
                        };
                        html! {
                            <div>
                                <label>
                                    <input type="checkbox" onchange={on_change} checked={is_checked} />
                                    { val }
                                </label>
                            </div>
                        }
                    }).collect::<Html>()
                }
            </div>
        }
    };

    // Render date range filter popup for datetime column.
    let render_date_range_filter = || {
        // Get current date filter state.
        let (start, end) = if let Some(FilterState::DateRange { start, end }) =
            filters.get(&SortColumn::DateTime) {
            (start.clone(), end.clone())
        } else {
            (None, None)
        };

        let filters_clone = filters.clone();
        // Create onchange callbacks for start and end inputs.
        let on_start_change = Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let new_start = if input.value().is_empty() { None } else { Some(input.value()) };
            let mut new_filters = (*filters_clone).clone();
            new_filters.insert(
                SortColumn::DateTime,
                FilterState::DateRange { start: new_start, end: new_filters.get(&SortColumn::DateTime)
                    .and_then(|fs| if let FilterState::DateRange { end, .. } = fs { end.clone() } else { None }) },
            );
            filters_clone.set(new_filters);
        });
        let filters_clone2 = filters.clone();
        let on_end_change = Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let new_end = if input.value().is_empty() { None } else { Some(input.value()) };
            let mut new_filters = (*filters_clone2).clone();
            new_filters.insert(
                SortColumn::DateTime,
                FilterState::DateRange { start: new_filters.get(&SortColumn::DateTime)
                    .and_then(|fs| if let FilterState::DateRange { start, .. } = fs { start.clone() } else { None }),
                    end: new_end },
            );
            filters_clone2.set(new_filters);
        });

        html! {
            <div class="filter-popup"
                style="position: absolute; background: white; border: 1px solid #ccc; padding: 8px; z-index: 10;">
                <div>
                    <label>{"Start: "}</label>
                    <input type="datetime-local" value={start.unwrap_or_default()} onchange={on_start_change} />
                </div>
                <div>
                    <label>{"End: "}</label>
                    <input type="datetime-local" value={end.unwrap_or_default()} onchange={on_end_change} />
                </div>
            </div>
        }
    };

    // Render header cell with sorting and filter controls.
    let render_header = |title: &str, col: SortColumn| {
        // Only show a filter button if there is more than one unique value,
        // or if the column is DateTime (always show date range filter).
        let unique_count = get_unique_values(&col).len();
        let show_filter = if col == SortColumn::DateTime {
            true
        } else {
            unique_count > 1
        };

        // Determine sort arrow (if active)
        let arrow = if let Some((current_col, ref order)) = *sort_state {
            if current_col == col {
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

        // Determine whether the filter popup for this column is open.
        let is_filter_open = if let Some(ref open) = *open_filter {
            *open == col
        } else {
            false
        };

        let on_filter_click = {
            let open_filter = open_filter.clone();
            Callback::from(move |_| {
                open_filter.set(if let Some(ref current) = *open_filter {
                    if *current == col { None } else { Some(col) }
                } else {
                    Some(col)
                });
            })
        };

        html! {
            <th style="position: relative;">
                <span onclick={Callback::from({
                    let col = col;
                    let on_header_click = on_header_click.clone();
                    move |_| on_header_click(col)
                })}>
                    { format!("{}{}", title, arrow) }
                </span>
                { 
                    if show_filter {
                        html! { <button onclick={on_filter_click} style="margin-left: 8px;">{"⚲"}</button> }
                    } else {
                        html! {}
                    }
                }
                { if is_filter_open {
                    if col == SortColumn::DateTime {
                        render_date_range_filter()
                    } else {
                        render_checkbox_filter(col)
                    }
                } else {
                    html! {}
                }}
            </th>
        }
    };

    html! {
        <div class="container">
            <h1>{ "Simple Table with Yew & Backend" }</h1>
            <table class="table" style="position: relative;">
                <thead>
                    <tr>
                        { render_header("ID", SortColumn::Id) }
                        { render_header("Name", SortColumn::Name) }
                        { render_header("Email", SortColumn::Email) }
                        { render_header("Role", SortColumn::Role) }
                        { render_header("Timestamp", SortColumn::DateTime) }
                    </tr>
                </thead>
                <tbody>
                    {
                        filtered_data.iter().map(|row| html! {
                            <tr key={row.id}>
                                <td>{ row.id }</td>
                                <td>{ &row.name }</td>
                                <td>{ &row.email }</td>
                                <td>{ &row.role }</td>
                                <td>{ &row.created_at }</td>
                            </tr>
                        }).collect::<Html>()
                    }
                </tbody>
            </table>
        </div>
    }
}