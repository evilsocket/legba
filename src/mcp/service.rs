use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use rmcp::model::{
    CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
};
use rmcp::{ServerHandler, tool};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Service {
    cache: Arc<RwLock<HashMap<String, String>>>,
    sessions: Arc<RwLock<crate::api::Sessions>>,
}

#[tool(tool_box)]
impl Service {
    #[allow(dead_code)]
    pub fn new(concurrency: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(crate::api::Sessions::new(concurrency))),
        }
    }

    #[tool(description = "Wait for a given amount of seconds.")]
    async fn sleep(
        &self,
        #[tool(param)]
        #[schemars(description = "Amount of seconds to wait")]
        seconds: u64,
    ) -> Result<CallToolResult, rmcp::Error> {
        log::info!("sleeping for {} seconds ...", seconds);
        tokio::time::sleep(Duration::from_secs(seconds)).await;
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Waited for {} seconds.",
            seconds
        ))]))
    }

    #[tool(description = "List all available plugins.")]
    async fn list_plugins(&self) -> Result<CallToolResult, rmcp::Error> {
        Ok(CallToolResult::success(vec![Content::text(
            include_str!("plugins.prompt").to_string(),
        )]))
    }

    #[tool(description = "Get information about a plugin.")]
    async fn plugin_info(
        &self,
        #[tool(param)]
        #[schemars(description = "Plugin identifier")]
        plugin_id: String,
    ) -> Result<CallToolResult, rmcp::Error> {
        if let Some(info) = self.cache.read().await.get(&plugin_id) {
            return Ok(CallToolResult::success(vec![Content::text(info.clone())]));
        }

        let url = format!(
            "https://raw.githubusercontent.com/evilsocket/legba/main/docs/plugins/{}.md",
            plugin_id
        );

        log::info!("fetching plugin info from {} ...", url);

        let info = reqwest::get(url)
            .await
            .map_err(|e| {
                rmcp::Error::invalid_params(format!("Failed to fetch plugin info: {}", e), None)
            })?
            .text()
            .await
            .map_err(|e| {
                rmcp::Error::invalid_params(format!("Failed to fetch plugin info: {}", e), None)
            })?;

        self.cache.write().await.insert(plugin_id, info.clone());

        Ok(CallToolResult::success(vec![Content::text(info)]))
    }

    #[tool(description = "Get the number of currently available workers.")]
    async fn get_available_workers(&self) -> Result<CallToolResult, rmcp::Error> {
        let guard = &*self.sessions.read().await;
        Ok(CallToolResult::success(vec![
            Content::json(guard.get_available_workers()).unwrap(),
        ]))
    }

    #[tool(
        description = "List basic information of all existing sessions. Sessions that have with_findings set to true have found something, otherwise they are not worth looking at."
    )]
    async fn list_sessions(&self) -> Result<CallToolResult, rmcp::Error> {
        let guard = &*self.sessions.read().await;
        let sessions = guard
            .get_sessions()
            .iter()
            .map(|(_, session)| session.get_listing())
            .collect::<Vec<_>>();
        Ok(CallToolResult::success(vec![
            Content::json(sessions).unwrap(),
        ]))
    }

    async fn did_session_complete(&self, session_id: &uuid::Uuid) -> Result<bool, rmcp::Error> {
        let guard = &*self.sessions.read().await;
        match guard.get_session(session_id) {
            Some(session) => Ok(session.is_completed()),
            None => Err(rmcp::Error::invalid_params("Session not found.", None)),
        }
    }

    #[tool(
        description = "Wait until a specific session is completed or a given amount of seconds has passed."
    )]
    async fn wait_for_session(
        &self,
        #[tool(param)]
        #[schemars(description = "Session id")]
        session_id: String,
        #[tool(param)]
        #[schemars(description = "Amount of seconds to wait")]
        seconds: u64,
    ) -> Result<CallToolResult, rmcp::Error> {
        let session_id = match uuid::Uuid::parse_str(&session_id) {
            Ok(uuid) => uuid,
            Err(_) => return Err(rmcp::Error::invalid_params("Session id not valid.", None)),
        };

        let start_time = std::time::Instant::now();

        loop {
            if self.did_session_complete(&session_id).await? {
                return Ok(CallToolResult::success(vec![Content::text(
                    "Session completed.".to_string(),
                )]));
            }

            log::info!("waiting for session {} to complete ...", session_id);
            tokio::time::sleep(Duration::from_secs(1)).await;

            if start_time.elapsed().as_secs() >= seconds {
                return Ok(CallToolResult::success(vec![Content::text(
                    if self.did_session_complete(&session_id).await? {
                        "Session completed.".to_string()
                    } else {
                        "Session is still running.".to_string()
                    },
                )]));
            }
        }
    }

    #[tool(description = "Show the entire session data given the session id.")]
    async fn show_session(
        &self,
        #[tool(param)]
        #[schemars(description = "Session id")]
        session_id: String,
    ) -> Result<CallToolResult, rmcp::Error> {
        let session_id = match uuid::Uuid::parse_str(&session_id) {
            Ok(uuid) => uuid,
            Err(_) => return Err(rmcp::Error::invalid_params("Session id not valid.", None)),
        };

        let guard = &*self.sessions.read().await;
        match guard.get_session(&session_id) {
            Some(session) => Ok(CallToolResult::success(vec![
                Content::json(session.get_brief()).unwrap(),
            ])),
            None => Err(rmcp::Error::invalid_params("Session not found.", None)),
        }
    }

    #[tool(description = "Stop a session by id.")]
    async fn stop_session(
        &self,
        #[tool(param)]
        #[schemars(description = "Session id")]
        session_id: String,
    ) -> Result<CallToolResult, rmcp::Error> {
        let session_id = match uuid::Uuid::parse_str(&session_id) {
            Ok(uuid) => uuid,
            Err(_) => return Err(rmcp::Error::invalid_params("Session id not valid.", None)),
        };

        let guard = &*self.sessions.read().await;
        match guard.stop_session(&session_id) {
            Ok(_) => Ok(CallToolResult::success(vec![Content::text(
                "Session stopped.".to_string(),
            )])),
            Err(_) => Err(rmcp::Error::invalid_params("Session not found.", None)),
        }
    }

    #[tool(description = "Create a new session with the given command line arguments.")]
    async fn start_session(
        &self,
        #[tool(param)]
        #[schemars(description = "Command line arguments for the session")]
        argv: Vec<String>,
    ) -> Result<CallToolResult, rmcp::Error> {
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
            Ok(session_id) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Session created with id: {}",
                session_id
            ))])),
            Err(e) => Err(rmcp::Error::invalid_params(
                format!("Failed to create session: {}", e),
                None,
            )),
        }
    }
}

#[tool(tool_box)]
impl ServerHandler for Service {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                // TODO: add loot to resources
                // .enable_resources()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(include_str!("service_info.prompt").to_string()),
        }
    }
}
