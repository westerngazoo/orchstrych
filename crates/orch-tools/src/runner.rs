//! Subprocess-based tool execution.

use std::path::PathBuf;
use std::process::{Command, Stdio};
use crate::error::ToolError;

const BUILT_IN_TOOLS: &[&str] = &["awk", "grep", "sed", "jq", "curl", "cat", "wc", "sort", "uniq"];

/// Runs native subprocesses as tools for the orchestration engine.
pub struct ToolRunner {
    plugins_dir: Option<PathBuf>,
}

impl ToolRunner {
    /// Create a new tool runner.
    pub fn new(plugins_dir: Option<PathBuf>) -> Self {
        Self { plugins_dir }
    }

    /// Execute a tool with the given arguments.
    pub fn execute(&self, tool: &str, args: &[&str]) -> Result<String, ToolError> {
        let cmd_path = self.resolve_tool(tool)?;
        tracing::debug!(tool = tool, args = ?args, "Executing tool");
        let output = Command::new(&cmd_path)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to spawn {tool}: {e}")))?;
        let result = output
            .wait_with_output()
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read output: {e}")))?;
        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(ToolError::ExecutionFailed(format!(
                "{tool} exited with {}: {stderr}",
                result.status
            )));
        }
        String::from_utf8(result.stdout).map_err(|_| ToolError::InvalidOutput)
    }

    /// Execute an awk script against a target file.
    pub fn execute_awk(&self, target_file: &str, awk_script: &str) -> Result<String, ToolError> {
        self.execute("awk", &[awk_script, target_file])
    }

    /// List available tools.
    pub fn available_tools(&self) -> Vec<String> {
        let mut tools: Vec<String> = BUILT_IN_TOOLS.iter().map(|s| (*s).to_string()).collect();
        if let Some(dir) = &self.plugins_dir {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        tools.push(name.to_string());
                    }
                }
            }
        }
        tools
    }

    fn resolve_tool(&self, tool: &str) -> Result<String, ToolError> {
        if let Some(dir) = &self.plugins_dir {
            let plugin_path = dir.join(tool);
            if plugin_path.exists() {
                return Ok(plugin_path.to_string_lossy().to_string());
            }
        }
        if BUILT_IN_TOOLS.contains(&tool) {
            return Ok(tool.to_string());
        }
        Err(ToolError::NotFound(tool.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn available_tools_includes_builtins() {
        let runner = ToolRunner::new(None);
        let tools = runner.available_tools();
        assert!(tools.contains(&"awk".to_string()));
        assert!(tools.contains(&"grep".to_string()));
    }

    #[test]
    fn unknown_tool_errors() {
        let runner = ToolRunner::new(None);
        let result = runner.execute("nonexistent_tool_xyz", &[]);
        assert!(result.is_err());
    }
}
