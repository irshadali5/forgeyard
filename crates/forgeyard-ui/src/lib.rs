#![allow(clippy::collapsible_if)]
use dioxus::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

const CSS: &str = include_str!("style.css");

pub fn launch_ui() {
    dioxus::launch(App);
}

fn get_api_base_url() -> String {
    if let Ok(url) = std::env::var("FORGEYARD_API_URL") {
        if !url.trim().is_empty() {
            return url;
        }
    }
    "http://127.0.0.1:8080".to_string()
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
    #[route("/graph")]
    KnowledgeGraph {},
    #[route("/search")]
    SemanticSearch {},
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
                to: Route::SemanticSearch {},
                class: "nav-link animate-fade-in delay-400",
                "Semantic Search"
            }
            Link {
                to: Route::KnowledgeGraph {},
                class: "nav-link animate-fade-in delay-300",
                "Codebase Graph"
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
    let url = format!("{}/api/v1/runners", get_api_base_url());
    reqwest::get(&url).await?.json::<forgeyard_api::ListRunnersResponse>().await
}

async fn fetch_status() -> Result<forgeyard_api::GetStatusResponse, reqwest::Error> {
    let url = format!("{}/api/v1/status/latest", get_api_base_url());
    reqwest::get(&url).await?.json::<forgeyard_api::GetStatusResponse>().await
}

async fn fetch_runs() -> Result<forgeyard_api::ListRunsResponse, reqwest::Error> {
    let url = format!("{}/api/v1/runs", get_api_base_url());
    reqwest::get(&url).await?.json::<forgeyard_api::ListRunsResponse>().await
}

async fn fetch_run_details(id: String) -> Result<forgeyard_api::GetStatusResponse, reqwest::Error> {
    let url = format!("{}/api/v1/status/{}", get_api_base_url(), id);
    reqwest::get(&url).await?.json::<forgeyard_api::GetStatusResponse>().await
}

#[allow(dead_code)]
async fn fetch_run_logs(id: String) -> Result<forgeyard_api::GetLogsResponse, reqwest::Error> {
    let url = format!("{}/api/v1/logs/{}", get_api_base_url(), id);
    reqwest::get(&url).await?.json::<forgeyard_api::GetLogsResponse>().await
}

#[derive(serde::Deserialize, Clone, PartialEq, Debug)]
struct PipelineMetrics {
    total_runs: i64,
    total_jobs: i64,
    total_logs: i64,
    cache_hit_ratio: f64,
}

#[derive(serde::Deserialize, Clone, PartialEq, Debug)]
struct GraphSummaryResponse {
    summary: String,
}

async fn fetch_metrics() -> Result<PipelineMetrics, reqwest::Error> {
    let url = format!("{}/api/v1/metrics", get_api_base_url());
    reqwest::get(&url).await?.json::<PipelineMetrics>().await
}

async fn fetch_graph() -> Result<GraphSummaryResponse, reqwest::Error> {
    let url = format!("{}/api/v1/graph", get_api_base_url());
    reqwest::get(&url).await?.json::<GraphSummaryResponse>().await
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

    let metrics_future = use_resource(move || async move { fetch_metrics().await });
    let total_runs = match metrics_future.read_unchecked().as_ref() {
        Some(Ok(m)) => m.total_runs,
        _ => 0,
    };
    let hit_ratio = match metrics_future.read_unchecked().as_ref() {
        Some(Ok(m)) => m.cache_hit_ratio,
        _ => 0.0,
    };

    rsx! {
        div {
            class: "animate-fade-in",
            h1 { class: "page-title", "Dashboard" }
            p { class: "page-subtitle", "Overview of your local CI/CD cluster" }

            div {
                class: "grid-cols-4",
                div {
                    class: "card",
                    div { class: "card-header", "Total Runs" }
                    div { class: "card-value", "{total_runs}" }
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
                div {
                    class: "card",
                    div { class: "card-header", "Cache Hit Ratio" }
                    div { class: "card-value", "{hit_ratio:.1}%" }
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
fn KnowledgeGraph() -> Element {
    let graph_future = use_resource(move || async move { fetch_graph().await });

    rsx! {
        div {
            class: "animate-fade-in",
            h1 { class: "page-title", "Codebase Knowledge Graph" }
            p { class: "page-subtitle", "AST Relationships & Token-Efficient Agent Context" }

            div {
                class: "card",
                div { class: "card-header", style: "margin-bottom: 1rem;", "Graphify AST Context Summary" }
                pre {
                    style: "background: #111; padding: 1.25rem; border-radius: 8px; font-family: monospace; color: #64748b; line-height: 1.6; white-space: pre-wrap;",
                    match graph_future.read_unchecked().as_ref() {
                        Some(Ok(g)) => rsx! { "{g.summary}" },
                        Some(Err(err)) => rsx! { "Failed to fetch AST Graph: {err}" },
                        None => rsx! { "Extracting AST Graph metrics via Tree-Sitter..." },
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
    let status_future = use_resource(move || { let id = id_clone.clone(); async move { fetch_run_details(id).await } });
    #[allow(unused_mut)]
    let mut logs = use_signal(Vec::new);

    let id_ws = id.clone();
    use_coroutine(move |_: UnboundedReceiver<()>| {
        #[allow(unused_mut)]
        let mut logs = logs;
        let run_id = id_ws.clone();
        async move {
            let base = get_api_base_url().replace("http://", "ws://").replace("https://", "wss://");
            let ws_url = format!("{}/api/v1/logs/stream/{}", base, run_id);
            if let Ok((mut ws_stream, _)) = tokio_tungstenite::connect_async(&ws_url).await {
                use futures_util::StreamExt;
                while let Some(msg) = ws_stream.next().await {
                    if let Ok(tokio_tungstenite::tungstenite::Message::Text(text)) = msg {
                        logs.write().push(text);
                    }
                }
            }
        }
    });

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

            PipelineGraph { run_id: id.clone() }

            div {
                class: "card",
                h2 { class: "card-header", "Logs" }
                pre {
                    style: "background: #111; padding: 1rem; border-radius: 8px; overflow-x: auto; font-family: monospace; font-size: 0.875rem; color: #ddd; min-height: 200px;",
                    for log in logs.read().iter() {
                        div { "{log}" }
                    }
                    if logs.read().is_empty() {
                        div { style: "color: #888;", "Waiting for logs..." }
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

#[component]
fn SemanticSearch() -> Element {
    let mut query = use_signal(String::new);
    let mut results = use_signal(Vec::<forgeyard_model::LogEvent>::new);
    let mut is_loading = use_signal(|| false);
    let mut error_msg = use_signal(String::new);

    let mut submit_search_1 = move || {
        let q = query.read().clone();
        if q.is_empty() { return; }
        
        is_loading.set(true);
        error_msg.set(String::new());
        
        spawn(async move {
            let client = reqwest::Client::new();
            let url = format!("{}/api/v1/search", get_api_base_url());
            
            #[derive(serde::Serialize)]
            struct SearchReq { query: String }
            
            match client.post(&url).json(&SearchReq { query: q }).send().await {
                Ok(resp) => {
                    if let Ok(data) = resp.json::<serde_json::Value>().await {
                        if let Some(res_arr) = data.get("results").and_then(|v| v.as_array()) {
                            let mut parsed = Vec::new();
                            for r in res_arr {
                                if let Ok(event) = serde_json::from_value(r.clone()) {
                                    parsed.push(event);
                                }
                            }
                            results.set(parsed);
                        }
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Search failed: {}", e));
                }
            }
            is_loading.set(false);
        });
    };

    let mut submit_search_2 = move || {
        let q = query.read().clone();
        if q.is_empty() { return; }
        
        is_loading.set(true);
        error_msg.set(String::new());
        
        spawn(async move {
            let client = reqwest::Client::new();
            let url = format!("{}/api/v1/search", get_api_base_url());
            
            #[derive(serde::Serialize)]
            struct SearchReq { query: String }
            
            match client.post(&url).json(&SearchReq { query: q }).send().await {
                Ok(resp) => {
                    if let Ok(data) = resp.json::<serde_json::Value>().await {
                        if let Some(res_arr) = data.get("results").and_then(|v| v.as_array()) {
                            let mut parsed = Vec::new();
                            for r in res_arr {
                                if let Ok(event) = serde_json::from_value(r.clone()) {
                                    parsed.push(event);
                                }
                            }
                            results.set(parsed);
                        }
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Search failed: {}", e));
                }
            }
            is_loading.set(false);
        });
    };

    rsx! {
        div {
            class: "search-container animate-fade-in",
            h1 { class: "page-title", "Semantic AI Search" }
            p { class: "page-subtitle", "Query Stoolap to find relevant logs and codebase hints." }
            
            div {
                class: "search-bar-wrapper glass-panel",
                input {
                    class: "search-input",
                    placeholder: "e.g. Why did the link fail during compilation?",
                    value: "{query}",
                    oninput: move |evt| query.set(evt.value().clone()),
                    onkeydown: move |evt| {
                        if evt.key().to_string() == "Enter" {
                            submit_search_1();
                        }
                    }
                }
                button {
                    class: "btn-primary search-btn",
                    onclick: move |_| submit_search_2(),
                    if *is_loading.read() {
                        "Searching..."
                    } else {
                        "Ask AI"
                    }
                }
            }
            
            if !error_msg.read().is_empty() {
                div { class: "error-toast", "{error_msg}" }
            }
            
            div {
                class: "search-results",
                for res in results.read().iter() {
                    div {
                        class: "result-card glass-panel animate-slide-up",
                        div { class: "result-header",
                            span { class: "result-job", "Job: {res.job_id.0}" }
                            span { class: "result-time", "{res.timestamp}" }
                        }
                        div { class: "result-body",
                            pre { class: "log-snippet", "{res.message}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PipelineGraph(run_id: String) -> Element {
    let mut jobs = use_signal(Vec::<forgeyard_api::JobStatusInfo>::new);
    
    use_coroutine(move |_: UnboundedReceiver<()>| {
        let run_id = run_id.clone();
        async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));
            let client = reqwest::Client::new();
            loop {
                interval.tick().await;
                let url = format!("{}/api/v1/status/{}", get_api_base_url(), run_id);
                if let Ok(resp) = client.get(&url).send().await {
                    if let Ok(status) = resp.json::<forgeyard_api::GetStatusResponse>().await {
                        jobs.set(status.jobs);
                    }
                }
            }
        }
    });

    let jobs_read = jobs.read();
    
    // Very naive topological layout for DAG:
    let mut depths = std::collections::HashMap::new();
    let mut ordered = Vec::new();
    let mut coords = std::collections::HashMap::new();
    
    for (i, job) in jobs_read.iter().enumerate() {
        let depth = job.dependencies.len() as i32;
        depths.insert(job.job_name.clone(), depth);
        ordered.push(job.clone());
        
        let cx = 50 + (depth * 250);
        let cy = 50 + (i as i32 * 80);
        coords.insert(job.job_name.clone(), (cx, cy));
    }
    
    rsx! {
        div {
            class: "card",
            style: "margin-bottom: 1.5rem; overflow-x: auto;",
            h2 { class: "card-header", "Pipeline Graph" }
            
            svg {
                width: "100%",
                height: "500px",
                style: "min-width: 600px;",
                
                // Edges
                for job in ordered.iter() {
                    if let Some(&(tx, ty)) = coords.get(&job.job_name) {
                        for dep in job.dependencies.iter() {
                            if let Some(&(sx, sy)) = coords.get(dep) {
                                // Draw a curved line from (sx, sy) to (tx, ty)
                                // Adjust by node radius (20)
                                path {
                                    d: "M {sx+20} {sy} C {sx+100} {sy}, {tx-100} {ty}, {tx-20} {ty}",
                                    stroke: "hsla(var(--border-strong), 0.7)",
                                    stroke_width: "2",
                                    fill: "none",
                                    class: "animate-dash"
                                }
                            }
                        }
                    }
                }
                
                // Nodes
                for job in ordered.iter() {
                    if let Some(&(cx, cy)) = coords.get(&job.job_name) {
                        g {
                            transform: "translate({cx}, {cy})",
                            
                            circle {
                                r: "20",
                                fill: match job.state.as_str() {
                                    "Succeeded" => "#10b981",
                                    "Failed" => "#ef4444",
                                    "Running" => "#3b82f6",
                                    "Cached" => "#8b5cf6",
                                    "Blocked" => "#f59e0b",
                                    _ => "#475569",
                                },
                                stroke: "white",
                                stroke_width: "2",
                            }
                            
                            text {
                                x: "30",
                                y: "5",
                                fill: "white",
                                font_family: "Inter, sans-serif",
                                font_size: "14px",
                                "{job.job_name}"
                            }
                            
                            text {
                                x: "30",
                                y: "25",
                                fill: "#94a3b8",
                                font_family: "Inter, sans-serif",
                                font_size: "12px",
                                "{job.state}"
                            }
                        }
                    }
                }
            }
        }
    }
}
