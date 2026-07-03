//! orchstrych — Rhizomatic Orchestration Engine.
//!
//! A local-first, Rust-native multi-agent orchestrator that models agents as
//! nodes in a Geometric Algebra–typed DAG.

use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

use orch_core::{AgentNode, AgentType, GeoGraph, TaskEdge, ga};
use orch_db::OrchDb;
use orch_engine::{Pipeline, Scheduler};

/// Rhizomatic Orchestration Engine
#[derive(Parser)]
#[command(name = "orchstrych", version, about)]
struct Cli {
    /// Path to the database file.
    #[arg(long, default_value = "orchstrych.redb")]
    db: String,

    /// Run the demo pipeline.
    #[arg(long)]
    demo: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let cli = Cli::parse();

    tracing::info!("Rhizomatic Orchestration Engine Online.");

    let db = Arc::new(OrchDb::open(&cli.db)?);

    if cli.demo {
        run_demo(db).await?;
    } else {
        tracing::info!("No command specified. Use --demo to run the demo pipeline.");
        tracing::info!("Database: {}", cli.db);
    }

    Ok(())
}

/// Run a demo pipeline: reasoner → coder → reviewer.
async fn run_demo(db: Arc<OrchDb>) -> Result<()> {
    tracing::info!("Building demo pipeline: reasoner → coder → reviewer");

    let reasoner = AgentNode::new("reasoner", AgentType::Reasoner, 0.9, 0.1, 0.0);
    let coder = AgentNode::new("coder", AgentType::CodeGen, 0.1, 0.2, 0.9);
    let reviewer = AgentNode::new("reviewer", AgentType::Reviewer, 0.7, 0.2, 0.3);

    let edge1 = TaskEdge::between(&reasoner, &coder, "analyze → generate");
    let edge2 = TaskEdge::between(&coder, &reviewer, "generate → review");

    let mut pipeline = Pipeline::new("demo");
    pipeline.add_agent(reasoner.clone());
    pipeline.add_agent(coder.clone());
    pipeline.add_agent(reviewer.clone());
    pipeline.graph.add_edge(edge1)?;
    pipeline.graph.add_edge(edge2)?;

    // Demonstrate GA-powered agent selection
    let reasoning_task = ga::vector(1.0, 0.0, 0.0);
    let tool_task = ga::vector(0.0, 1.0, 0.0);
    let gen_task = ga::vector(0.0, 0.0, 1.0);

    tracing::info!(
        "Best agent for reasoning: {:?}",
        pipeline.best_agent_for(&reasoning_task).map(|(id, s)| (
            pipeline.graph.node(&id).map(|a| a.name.clone()),
            s
        ))
    );
    tracing::info!(
        "Best agent for generation: {:?}",
        pipeline.best_agent_for(&gen_task).map(|(id, s)| (
            pipeline.graph.node(&id).map(|a| a.name.clone()),
            s
        ))
    );
    tracing::info!(
        "Qualified for tool-use (threshold 0.15): {:?}",
        pipeline
            .qualified_agents(&tool_task, 0.15)
            .iter()
            .filter_map(|id| pipeline.graph.node(id).map(|a| a.name.clone()))
            .collect::<Vec<_>>()
    );

    // Save the graph
    let mut graph = GeoGraph::new();
    let r2 = reasoner.clone();
    let c2 = coder.clone();
    let rev2 = reviewer.clone();
    graph.add_node(r2);
    graph.add_node(c2);
    graph.add_node(rev2);
    let e1 = TaskEdge::between(&reasoner, &coder, "analyze → generate");
    let e2 = TaskEdge::between(&coder, &reviewer, "generate → review");
    graph.add_edge(e1)?;
    graph.add_edge(e2)?;
    db.save_graph(&graph)?;

    // Execute the pipeline
    let scheduler = Scheduler::new(db);
    let results = scheduler.execute(&pipeline).await;
    for result in &results {
        tracing::info!("Task result: {} → {}", result.output, format!("{:?}", result.state));
    }

    tracing::info!("Demo pipeline complete. {} tasks executed.", results.len());

    Ok(())
}
