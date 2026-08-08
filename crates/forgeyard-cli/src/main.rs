use clap::{Parser, Subcommand};
use miette::{IntoDiagnostic, Result};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Parser)]
#[command(
    name = "forgeyard",
    version,
    about = "Local-First, Cross-Platform CI/CD Platform"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new forgeyard project
    Init,
    /// Inspect the current directory
    Inspect,
    /// Run the build pipeline
    Run {
        /// Watch the run until it finishes
        #[arg(short, long)]
        watch: bool,
    },
    /// View the generated pipeline plan without executing
    Plan,
    /// View the status of a pipeline run
    Status {
        #[arg(short, long)]
        run_id: Option<String>,
        #[arg(short, long)]
        watch: bool,
    },
    /// View logs for a run
    Logs {
        run_id: String,
        #[arg(short, long)]
        follow: bool,
    },
    /// Manage runners
    Runner {
        #[command(subcommand)]
        action: RunnerAction,
    },
    /// View build matrix
    Matrix,
    /// Import pipeline from GitHub Actions or GitLab CI
    Import {
        /// Platform (github or gitlab)
        #[arg(short, long, default_value = "github")]
        platform: String,
        /// Path to workflow file
        file: String,
    },
}

#[derive(Subcommand)]
enum RunnerAction {
    /// List all registered runners
    List,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            println!("Initializing Forgeyard...");
            let config_path = "forgeyard.ron";
            if std::path::Path::new(config_path).exists() {
                println!("forgeyard.ron already exists.");
            } else {
                let default_config = r#"
ForgeyardConfig(
    version: 1,
    project: ProjectConfig(
        name: "my-project",
    ),
    pipelines: {
        "default": PipelineConfig(
            triggers: [],
            stages: ["build", "test"],
            jobs: {
                "build": JobConfig(
                    needs: [],
                    command: ["cargo", "build"],
                    matrix: None,
                ),
                "test": JobConfig(
                    needs: ["build"],
                    command: ["cargo", "test"],
                    matrix: None,
                ),
            },
        ),
    },
)
"#;
                std::fs::write(config_path, default_config.trim()).into_diagnostic()?;
                println!("Created default forgeyard.ron.");
            }
        }
        Commands::Inspect => {
            println!("Inspecting repository...");
            let analyzer = forgeyard_detector::WorkspaceAnalyzer::new();
            match analyzer.analyze(std::path::Path::new(".")).await {
                Ok(evidence) => {
                    if evidence.is_empty() {
                        println!("No specific project types detected.");
                    } else {
                        for ev in evidence {
                            println!("Detected technology: {:?}", ev.kind);
                            println!("  Frameworks: {:?}", ev.frameworks);
                            println!("  Targets: {:?}", ev.intended_targets);
                        }
                    }
                }
                Err(e) => {
                    miette::bail!("Failed to inspect workspace: {}", e);
                }
            }
        }
        Commands::Run { watch } => {
            let base_url = daemon_url();
            let auth_token = std::env::var("FORGEYARD_TOKEN").unwrap_or_else(|_| "default_token".to_string());
            println!("Submitting run to local Forgeyard Daemon at {}...", base_url);
            let client = reqwest::Client::new();
            let req = forgeyard_api::SubmitRunRequest {
                workspace_path: std::env::current_dir().unwrap().to_string_lossy().to_string(),
                variables: std::collections::HashMap::new(),
                override_branch: None,
            };

            let run_id = match client.post(format!("{}/api/v1/run", base_url))
                .header("Authorization", format!("Bearer {}", auth_token))
                .json(&req)
                .send()
                .await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        let result: forgeyard_api::SubmitRunResponse = resp.json().await.into_diagnostic()?;
                        println!("Run {} accepted! Status: {}", result.run_id, result.status);
                        result.run_id
                    } else {
                        miette::bail!("Daemon rejected run: {}", resp.status());
                    }
                },
                Err(e) => {
                    miette::bail!("Failed to contact Forgeyard daemon at {}. Error: {}", base_url, e);
                }
            };

            if watch {
                watch_status(&client, &run_id).await?;
            }
        }
        Commands::Plan => {
            println!("Generating Forgeyard execution plan...");
            let mut config = if std::path::Path::new("forgeyard.ron").exists() {
                forgeyard_config::ForgeyardConfig::load("forgeyard.ron").into_diagnostic()?
            } else {
                forgeyard_config::ForgeyardConfig {
                    version: 1,
                    project: forgeyard_config::ProjectConfig { name: "default".to_string() },
                    pipelines: std::collections::BTreeMap::new(),
                }
            };
            config = forgeyard_adapter_cargo::CargoAdapter::inject_into_config(config, ".");
            match forgeyard_pipeline::PipelineCompiler::compile(&config, "default") {
                Ok(ir) => {
                    println!("Pipeline ID: {}", ir.pipeline_id.0);
                    println!("Total Jobs: {}", ir.jobs.len());
                    for (_id, job) in ir.jobs {
                        println!(" - {}", job.name);
                        println!("     Dependencies: {}", job.dependencies.len());
                    }
                }
                Err(e) => miette::bail!("Compilation failed: {}", e)
            }
        }
        Commands::Status { run_id, watch } => {
            let client = reqwest::Client::new();
            let target_run = run_id.unwrap_or_else(|| "latest".to_string());
            
            if watch {
                watch_status(&client, &target_run).await?;
            } else {
                print_status(&client, &target_run).await?;
            }
        }
        Commands::Logs { run_id, follow } => {
            let client = reqwest::Client::new();
            if follow {
                let mut seen_lines = 0;
                loop {
                    if let Ok(resp) = client.get(format!("http://127.0.0.1:8080/api/v1/logs/{}", run_id)).send().await {
                        if resp.status().is_success() {
                            if let Ok(res) = resp.json::<forgeyard_api::GetLogsResponse>().await {
                                for i in seen_lines..res.logs.len() {
                                    println!("{}", res.logs[i]);
                                }
                                seen_lines = res.logs.len();
                            }
                        }
                    }
                    sleep(Duration::from_millis(500)).await;
                }
            } else {
                match client.get(format!("http://127.0.0.1:8080/api/v1/logs/{}", run_id)).send().await {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            let res: forgeyard_api::GetLogsResponse = resp.json().await.into_diagnostic()?;
                            for log in res.logs {
                                println!("{}", log);
                            }
                        } else {
                            miette::bail!("Failed to fetch logs: {}", resp.status());
                        }
                    }
                    Err(e) => miette::bail!("API Error: {}", e)
                }
            }
        }
        Commands::Runner { action } => {
            match action {
                RunnerAction::List => {
                    let client = reqwest::Client::new();
                    match client.get("http://127.0.0.1:8080/api/v1/runners").send().await {
                        Ok(resp) => {
                            if resp.status().is_success() {
                                let res: forgeyard_api::ListRunnersResponse = resp.json().await.into_diagnostic()?;
                                println!("{0: <40} | {1: <10} | {2: <10} | {3: <30} | {4: <15}", "Runner ID", "OS", "Arch", "Capabilities", "Last Seen (s)");
                                println!("{0:-<40}-+-{0:-<10}-+-{0:-<10}-+-{0:-<30}-+-{0:-<15}", "");
                                let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                                for r in res.runners {
                                    let ago = now.saturating_sub(r.last_seen);
                                    let caps = r.capabilities.join(", ");
                                    println!("{0: <40} | {1: <10} | {2: <10} | {3: <30} | {4: <15}", r.runner_id, r.os, r.arch, caps, format!("{}s ago", ago));
                                }
                            } else {
                                miette::bail!("Failed to list runners: {}", resp.status());
                            }
                        }
                        Err(e) => miette::bail!("API Error: {}", e)
                    }
                }
            }
        }
        Commands::Matrix => {
            println!("Generating build matrix for current directory...");
            let mut config = if std::path::Path::new("forgeyard.ron").exists() {
                forgeyard_config::ForgeyardConfig::load("forgeyard.ron").into_diagnostic()?
            } else {
                forgeyard_config::ForgeyardConfig {
                    version: 1,
                    project: forgeyard_config::ProjectConfig { name: "default".to_string() },
                    pipelines: std::collections::BTreeMap::new(),
                }
            };
            config = forgeyard_adapter_cargo::CargoAdapter::inject_into_config(config, ".");
            // Ideally we'd hit the Daemon here with a full Matrix Endpoint, but local evaluation is valid for matrix pre-flight
            let ir = forgeyard_pipeline::PipelineCompiler::compile(&config, "default").into_diagnostic()?;
            
            println!("{0: <20} | {1: <30} | {2: <20}", "Job", "Command", "Matrix Dimension");
            println!("{0:-<20}-+-{0:-<30}-+-{0:-<20}", "");
            for (_id, job) in ir.jobs {
                let cmd = match &job.execution {
                    forgeyard_model::ExecutionSpec::Command { program, arguments, .. } => {
                        format!("{} {}", program, arguments.join(" "))
                    }
                    forgeyard_model::ExecutionSpec::Container { program, arguments, .. } => {
                        format!("{} {}", program, arguments.join(" "))
                    }
                    _ => "Unknown".to_string()
                };
                let cmd_trunc = if cmd.len() > 28 { format!("{}...", &cmd[0..25]) } else { cmd };
                
                println!("{0: <20} | {1: <30} | {2: <20}", job.name, cmd_trunc, "N/A");
            }
        }
        Commands::Import { platform, file } => {
            println!("Importing {} workflow from {}...", platform, file);
            let content = std::fs::read_to_string(&file).into_diagnostic()?;
            let config = if platform.to_lowercase() == "gitlab" {
                forgeyard_config::GitLabCIConverter::convert_yaml("imported-project", &content)
            } else {
                forgeyard_config::GitHubWorkflowConverter::convert_yaml("imported-project", &content)
            };
            let ron_str = ron::ser::to_string_pretty(&config, ron::ser::PrettyConfig::default()).into_diagnostic()?;
            std::fs::write("forgeyard.ron", ron_str).into_diagnostic()?;
            println!("Successfully imported pipeline into forgeyard.ron!");
        }
    }

    Ok(())
}

fn daemon_url() -> String {
    std::env::var("FORGEYARD_DAEMON_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".to_string())
}

async fn print_status(client: &reqwest::Client, target_run: &str) -> Result<bool> {
    let auth_token = std::env::var("FORGEYARD_TOKEN").unwrap_or_else(|_| "default_token".to_string());
    match client.get(format!("{}/api/v1/status/{}", daemon_url(), target_run))
        .header("Authorization", format!("Bearer {}", auth_token))
        .send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                let res: forgeyard_api::GetStatusResponse = resp.json().await.into_diagnostic()?;
                println!("\n=== Status for Run: {} ===", res.run_id);
                let mut all_done = true;
                for job in &res.jobs {
                    println!("  [{}] {}", job.state, job.job_name);
                    if job.state != "Succeeded" && job.state != "Failed" && job.state != "Cancelled" {
                        all_done = false;
                    }
                }
                Ok(all_done)
            } else {
                miette::bail!("Failed to fetch status: {}", resp.status());
            }
        }
        Err(e) => miette::bail!("API Error: {}", e)
    }
}

async fn watch_status(client: &reqwest::Client, target_run: &str) -> Result<()> {
    println!("Watching status for run {}...", target_run);
    loop {
        let is_done = print_status(client, target_run).await?;
        if is_done {
            println!("Run finished.");
            break;
        }
        sleep(Duration::from_secs(2)).await;
        // Move cursor up to overwrite previous status
        print!("\x1B[2J\x1B[1;1H");
    }
    Ok(())
}
