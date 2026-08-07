use dioxus::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

const CSS: &str = include_str!("style.css");

pub fn launch_ui() {
    dioxus::launch(App);
}

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Dashboard {},
    #[route("/runs")]
    RunsList {},
    #[route("/runs/:id")]
    RunDetails { id: String },
    #[route("/agents")]
    AgentsList {},
    #[route("/settings")]
    Settings {},
}

#[component]
fn App() -> Element {
    rsx! {
        style { "{CSS}" }
        div {
            id: "main-app",
            Sidebar {}
            div {
                class: "content-area",
                Router::<Route> {}
            }
        }
    }
}

#[component]
fn Sidebar() -> Element {
    rsx! {
        div {
            class: "sidebar",
            div {
                class: "sidebar-logo",
                "Forgeyard"
            }
            Link {
                to: Route::Dashboard {},
                class: "nav-link animate-fade-in delay-100",
                "Dashboard"
            }
            Link {
                to: Route::RunsList {},
                class: "nav-link animate-fade-in delay-200",
                "Pipeline Runs"
            }
            Link {
                to: Route::AgentsList {},
                class: "nav-link animate-fade-in delay-300",
                "Runners & Agents"
            }
            Link {
                to: Route::Settings {},
                class: "nav-link animate-fade-in delay-300",
                "Settings"
            }
        }
    }
}

async fn fetch_runners() -> Result<forgeyard_api::ListRunnersResponse, reqwest::Error> {
    reqwest::get("http://127.0.0.1:8080/api/v1/runners")
        .await?
        .json::<forgeyard_api::ListRunnersResponse>()
        .await
}

async fn fetch_status() -> Result<forgeyard_api::GetStatusResponse, reqwest::Error> {
    reqwest::get("http://127.0.0.1:8080/api/v1/status/latest")
        .await?
        .json::<forgeyard_api::GetStatusResponse>()
        .await
}

async fn fetch_runs() -> Result<forgeyard_api::ListRunsResponse, reqwest::Error> {
    reqwest::get("http://127.0.0.1:8080/api/v1/runs")
        .await?
        .json::<forgeyard_api::ListRunsResponse>()
        .await
}

async fn fetch_run_details(id: String) -> Result<forgeyard_api::GetStatusResponse, reqwest::Error> {
    reqwest::get(&format!("http://127.0.0.1:8080/api/v1/status/{}", id))
        .await?
        .json::<forgeyard_api::GetStatusResponse>()
        .await
}

async fn fetch_run_logs(id: String) -> Result<forgeyard_api::GetLogsResponse, reqwest::Error> {
    reqwest::get(&format!("http://127.0.0.1:8080/api/v1/logs/{}", id))
        .await?
        .json::<forgeyard_api::GetLogsResponse>()
        .await
}

#[component]
fn Dashboard() -> Element {
    let status_future = use_resource(move || async move { fetch_status().await });
    let runners_future = use_resource(move || async move { fetch_runners().await });

    let running_jobs = match status_future.read_unchecked().as_ref() {
        Some(Ok(s)) => s.jobs.iter().filter(|j| j.state != "Succeeded" && j.state != "Failed").count(),
        _ => 0,
    };
    
    let active_runners = match runners_future.read_unchecked().as_ref() {
        Some(Ok(r)) => r.runners.len(),
        _ => 0,
    };

    rsx! {
        div {
            class: "animate-fade-in",
            h1 { class: "page-title", "Dashboard" }
            p { class: "page-subtitle", "Overview of your local CI/CD cluster" }

            div {
                class: "grid-cols-3",
                div {
                    class: "card",
                    div { class: "card-header", "Active Runs" }
                    div { class: "card-value", "1" }
                }
                div {
                    class: "card",
                    div { class: "card-header", "Running Jobs" }
                    div { class: "card-value", "{running_jobs}" }
                }
                div {
                    class: "card",
                    div { class: "card-header", "Connected Agents" }
                    div { class: "card-value", "{active_runners}" }
                }
            }

            h2 { class: "card-header", style: "margin-top: 2.5rem;", "Latest Pipeline Execution" }
            div {
                class: "card",
                match status_future.read_unchecked().as_ref() {
                    Some(Ok(status)) => {
                        rsx! {
                            div {
                                class: "grid-cols-2",
                                for job in &status.jobs {
                                    div {
                                        class: "card", style: "background: rgba(0,0,0,0.2);",
                                        div { class: "card-header", "{job.job_name}" }
                                        div {
                                            if job.state == "Succeeded" {
                                                span { class: "badge badge-success", "{job.state}" }
                                            } else if job.state == "Failed" {
                                                span { class: "badge badge-danger", "{job.state}" }
                                            } else {
                                                span { class: "badge badge-warning", "{job.state}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    Some(Err(_)) => rsx! { div { "Daemon offline or no latest run" } },
                    None => rsx! { div { "Loading..." } },
                }
            }
        }
    }
}

#[component]
fn AgentsList() -> Element {
    let runners_future = use_resource(move || async move { fetch_runners().await });

    rsx! {
        div {
            class: "animate-fade-in",
            h1 { class: "page-title", "Runners & Agents" }
            p { class: "page-subtitle", "Manage your execution fleet" }

            div {
                class: "card",
                table {
                    class: "data-table",
                    thead {
                        tr {
                            th { "Runner ID" }
                            th { "OS" }
                            th { "Arch" }
                            th { "Capabilities" }
                            th { "Last Seen" }
                        }
                    }
                    tbody {
                        match runners_future.read_unchecked().as_ref() {
                            Some(Ok(res)) => {
                                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                                rsx! {
                                    for runner in &res.runners {
                                        tr {
                                            td { "{runner.runner_id.chars().take(8).collect::<String>()}..." }
                                            td { "{runner.os}" }
                                            td { "{runner.arch}" }
                                            td {
                                                for cap in &runner.capabilities {
                                                    span { class: "badge badge-primary", style: "margin-right: 0.5rem;", "{cap}" }
                                                }
                                            }
                                            td { "{now.saturating_sub(runner.last_seen)}s ago" }
                                        }
                                    }
                                }
                            },
                            Some(Err(err)) => rsx! { tr { td { colspan: 5, "Failed to fetch runners: {err}" } } },
                            None => rsx! { tr { td { colspan: 5, "Loading..." } } },
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn RunsList() -> Element {
    let runs_future = use_resource(move || async move { fetch_runs().await });

    rsx! { 
        div { 
            class: "animate-fade-in",
            h1 { class: "page-title", "Pipeline Runs" }
            p { class: "page-subtitle", "History of executed pipelines" }
            
            div {
                class: "card",
                table {
                    class: "data-table",
                    thead {
                        tr {
                            th { "Run ID" }
                            th { "Actions" }
                        }
                    }
                    tbody {
                        match runs_future.read_unchecked().as_ref() {
                            Some(Ok(res)) => {
                                rsx! {
                                    for run_id in &res.runs {
                                        tr {
                                            td { "{run_id}" }
                                            td {
                                                Link {
                                                    to: Route::RunDetails { id: run_id.clone() },
                                                    class: "btn btn-primary",
                                                    "View Details"
                                                }
                                            }
                                        }
                                    }
                                }
                            },
                            Some(Err(err)) => rsx! { tr { td { colspan: 2, "Failed to fetch runs: {err}" } } },
                            None => rsx! { tr { td { colspan: 2, "Loading..." } } },
                        }
                    }
                }
            }
        } 
    }
}

#[component]
fn RunDetails(id: String) -> Element {
    let id_clone = id.clone();
    let id_clone2 = id.clone();
    let status_future = use_resource(move || { let id = id_clone.clone(); async move { fetch_run_details(id).await } });
    let logs_future = use_resource(move || { let id = id_clone2.clone(); async move { fetch_run_logs(id).await } });

    rsx! {
        div {
            class: "animate-fade-in",
            h1 { class: "page-title", "Run {id}" }
            
            div {
                class: "card",
                style: "margin-bottom: 1.5rem;",
                h2 { class: "card-header", "Jobs" }
                match status_future.read_unchecked().as_ref() {
                    Some(Ok(status)) => rsx! {
                        div {
                            class: "grid-cols-2",
                            for job in &status.jobs {
                                div {
                                    class: "card", style: "background: rgba(0,0,0,0.2);",
                                    div { class: "card-header", "{job.job_name}" }
                                    div {
                                        if job.state == "Succeeded" {
                                            span { class: "badge badge-success", "{job.state}" }
                                        } else if job.state == "Failed" {
                                            span { class: "badge badge-danger", "{job.state}" }
                                        } else {
                                            span { class: "badge badge-warning", "{job.state}" }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    Some(Err(err)) => rsx! { div { "Failed to load jobs: {err}" } },
                    None => rsx! { div { "Loading jobs..." } },
                }
            }

            div {
                class: "card",
                h2 { class: "card-header", "Logs" }
                pre {
                    style: "background: #111; padding: 1rem; border-radius: 8px; overflow-x: auto; font-family: monospace; font-size: 0.875rem; color: #ddd;",
                    match logs_future.read_unchecked().as_ref() {
                        Some(Ok(res)) => rsx! {
                            for log in &res.logs {
                                div { "{log}" }
                            }
                        },
                        Some(Err(err)) => rsx! { div { "Failed to load logs: {err}" } },
                        None => rsx! { div { "Loading logs..." } },
                    }
                }
            }
        }
    }
}

#[component]
fn Settings() -> Element {
    rsx! { 
        div { 
            class: "animate-fade-in",
            h1 { class: "page-title", "Settings" }
            p { class: "page-subtitle", "Configure your Forgeyard instance" }
            
            div { 
                class: "card",
                div {
                    class: "settings-group",
                    style: "margin-bottom: 1.5rem;",
                    h3 { "Daemon Configuration" }
                    p { class: "text-muted", style: "color: hsl(var(--text-muted));", "Local runner limits and concurrency settings." }
                    div {
                        style: "display: flex; flex-direction: column; gap: 0.5rem; margin-top: 1rem;",
                        label { style: "font-size: 0.875rem; color: hsl(var(--text-secondary));", "Max Concurrent Jobs" }
                        input { 
                            type: "number", 
                            value: "4", 
                            style: "padding: 0.75rem; border-radius: 8px; border: 1px solid hsla(var(--border-strong), 0.5); background: hsla(var(--bg-surface-elevated), 0.5); color: white; width: 100%; max-width: 300px; outline: none; transition: border-color 0.2s;" 
                        }
                    }
                }
                button { class: "btn btn-primary", "Save Settings" }
            } 
        } 
    }
}
