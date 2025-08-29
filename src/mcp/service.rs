use std::sync::Arc;
use std::time::Duration;

use include_dir::{Dir, include_dir};
use rmcp::handler::server::tool::{Parameters, ToolRouter};
use rmcp::model::{ServerCapabilities, ServerInfo};
use rmcp::{ServerHandler, schemars, tool, tool_handler, tool_router};
use tokio::sync::RwLock;

const PLUGINS_DOCS_DIR: Dir = include_dir!("docs/plugins");

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct SleepRequest {
    #[schemars(description = "Amount of seconds to wait")]
    pub seconds: u64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct PluginInfoRequest {
    #[schemars(description = "Plugin identifier")]
    pub plugin_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct DidSessionCompleteRequest {
    #[schemars(description = "Session id")]
    pub session_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct WaitForSessionRequest {
    #[schemars(description = "Session id")]
    pub session_id: String,
    #[schemars(description = "Amount of seconds to wait")]
    pub seconds: u64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ShowSessionRequest {
    #[schemars(description = "Session id")]
    pub session_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct StopSessionRequest {
    #[schemars(description = "Session id")]
    pub session_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct StartSessionRequest {
    #[schemars(description = "Command line arguments for the session")]
    pub argv: Vec<String>,
}

#[derive(Clone)]
pub struct Service {
    sessions: Arc<RwLock<crate::api::Sessions>>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl Service {
    #[allow(dead_code)]
    pub fn new(concurrency: usize) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(crate::api::Sessions::new(concurrency))),
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Wait for a given amount of seconds.")]
    async fn sleep(
        &self,
        Parameters(SleepRequest { seconds }): Parameters<SleepRequest>,
    ) -> String {
        log::info!("sleeping for {} seconds ...", seconds);
        tokio::time::sleep(Duration::from_secs(seconds)).await;
        format!("Waited for {} seconds.", seconds)
    }

    #[tool(description = "List all available plugins.")]
    async fn list_plugins(&self) -> String {
        // Populate the prompt with the list of plugins.
        let mut plugins = Vec::new();
        for plugin in PLUGINS_DOCS_DIR.files() {
            plugins.push(format!(
                "* {}",
                plugin
                    .path()
                    .to_str()
                    .unwrap()
                    .to_string()
                    .replace(".md", "")
            ));
        }
        include_str!("plugins.prompt").replace("##PLUGIN_LIST##", &plugins.join("\n"))
    }

    #[tool(description = "Get information about a plugin.")]
    async fn plugin_info(
        &self,
        Parameters(PluginInfoRequest { plugin_id }): Parameters<PluginInfoRequest>,
    ) -> String {
        match PLUGINS_DOCS_DIR.get_file(format!("{}.md", plugin_id.to_lowercase())) {
            Some(plugin_info) => plugin_info.contents_utf8().unwrap().to_string(),
            None => format!("Plugin {} not found.", plugin_id),
        }
    }

    #[tool(description = "Get the number of currently available workers.")]
    async fn get_available_workers(&self) -> String {
        let guard = &*self.sessions.read().await;
        format!("{}", guard.get_available_workers())
    }

    #[tool(
        description = "List basic information of all existing sessions. Sessions that have with_findings set to true have found something, otherwise they are not worth looking at."
    )]
    async fn list_sessions(&self) -> String {
        let guard = &*self.sessions.read().await;
        let sessions = guard
            .get_sessions()
            .values()
            .map(|session| session.get_listing())
            .collect::<Vec<_>>();
        serde_json::to_string(&sessions).unwrap()
    }

    #[tool(description = "Check if a session is completed.")]
    async fn did_session_complete(
        &self,
        Parameters(DidSessionCompleteRequest { session_id }): Parameters<DidSessionCompleteRequest>,
    ) -> String {
        match self.did_session_complete_internal(&session_id).await {
            Ok(true) => "true".to_string(),
            Ok(false) => "false".to_string(),
            Err(e) => e,
        }
    }

    async fn did_session_complete_internal(&self, session_id: &str) -> Result<bool, String> {
        let session_id = match uuid::Uuid::parse_str(session_id) {
            Ok(uuid) => uuid,
            Err(_) => return Err("Session id not valid.".to_string()),
        };

        let guard = &*self.sessions.read().await;
        match guard.get_session(&session_id) {
            Some(session) => Ok(session.is_completed()),
            None => Err("Session not found.".to_string()),
        }
    }

    #[tool(
        description = "Wait until a specific session is completed or a given amount of seconds has passed."
    )]
    async fn wait_for_session(
        &self,
        Parameters(WaitForSessionRequest {
            session_id,
            seconds,
        }): Parameters<WaitForSessionRequest>,
    ) -> String {
        let start_time = std::time::Instant::now();

        loop {
            match self.did_session_complete_internal(&session_id).await {
                Ok(true) => return "Session completed.".to_string(),
                Ok(false) => {}
                Err(e) => return e,
            }

            log::info!("waiting for session {} to complete ...", session_id);
            tokio::time::sleep(Duration::from_secs(1)).await;

            if start_time.elapsed().as_secs() >= seconds {
                return if self
                    .did_session_complete_internal(&session_id)
                    .await
                    .unwrap_or(false)
                {
                    "Session completed.".to_string()
                } else {
                    "Session is still running.".to_string()
                };
            }
        }
    }

    #[tool(description = "Show the entire session data given the session id.")]
    async fn show_session(
        &self,
        Parameters(ShowSessionRequest { session_id }): Parameters<ShowSessionRequest>,
    ) -> String {
        let session_id = match uuid::Uuid::parse_str(&session_id) {
            Ok(uuid) => uuid,
            Err(_) => return "Session id not valid.".to_string(),
        };

        let guard = &*self.sessions.read().await;
        match guard.get_session(&session_id) {
            Some(session) => serde_json::to_string(&session.get_brief()).unwrap(),
            None => "Session not found.".to_string(),
        }
    }

    #[tool(description = "Stop a session by id.")]
    async fn stop_session(
        &self,
        Parameters(StopSessionRequest { session_id }): Parameters<StopSessionRequest>,
    ) -> String {
        let session_id = match uuid::Uuid::parse_str(&session_id) {
            Ok(uuid) => uuid,
            Err(_) => return "Session id not valid.".to_string(),
        };

        let guard = &*self.sessions.read().await;
        match guard.stop_session(&session_id) {
            Ok(_) => "Session stopped.".to_string(),
            Err(_) => "Session not found.".to_string(),
        }
    }

    #[tool(description = "Create a new session with the given command line arguments.")]
    async fn start_session(
        &self,
        Parameters(StartSessionRequest { argv }): Parameters<StartSessionRequest>,
    ) -> String {
        let argv = if !argv.is_empty() && argv[0] == "legba" {
            argv[1..].to_vec()
        } else {
            argv
        };

        match self
            .sessions
            .write()
            .await
            .start_new_session("mcp_client".to_string(), argv)
            .await
        {
            Ok(session_id) => format!("Session created with id: {}", session_id),
            Err(e) => format!("Failed to create session: {}", e),
        }
    }
}

#[tool_handler]
impl ServerHandler for Service {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(include_str!("service_info.prompt").to_owned()),
            // TODO: add loot to resources?
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
