
use std::sync::Arc;

use rmcp::model::{CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo};
use rmcp::{tool, ServerHandler};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Service {
    sessions: Arc<RwLock<crate::api::Sessions>>,
}

#[tool(tool_box)]
impl Service {
    #[allow(dead_code)]
    pub fn new(concurrency: usize) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(crate::api::Sessions::new(concurrency))),
        }
    }

    #[tool(description = "List all available plugins.")]
    async fn list_plugins(&self) -> Result<CallToolResult, rmcp::Error> {
        Ok(CallToolResult::success(vec![Content::text(include_str!("plugins.prompt").to_string())]))
    }

    #[tool(description = "List currently active sessions.")]
    async fn list_sessions(&self) -> Result<CallToolResult, rmcp::Error> {
        let guard = &*self.sessions.read().await;
        Ok(CallToolResult::success(vec![Content::json(guard).unwrap()]))
    }

    #[tool(description = "Show a session by id.")]
    async fn show_session(&self, 
        #[tool(param)]
        #[schemars(description = "Session id")]
        session_id: String) -> Result<CallToolResult, rmcp::Error> {
        let session_id = match uuid::Uuid::parse_str(&session_id) {
            Ok(uuid) => uuid,
            Err(_) => return Err(rmcp::Error::invalid_params("Session id not valid.", None))
        };
    
        let guard = &*self.sessions.read().await;
        match guard.get_session(&session_id) {
            Some(session) => Ok(CallToolResult::success(vec![Content::json(session).unwrap()])),
            None => Err(rmcp::Error::invalid_params("Session not found.", None))
        }
    }


    #[tool(description = "Stop a session by id.")]
    async fn stop_session(&self, 
        #[tool(param)]
        #[schemars(description = "Session id")]
        session_id: String) -> Result<CallToolResult, rmcp::Error> {
        let session_id = match uuid::Uuid::parse_str(&session_id) {
            Ok(uuid) => uuid,
            Err(_) => return Err(rmcp::Error::invalid_params("Session id not valid.", None))
        };
    
        let guard = &*self.sessions.read().await;
        match guard.stop_session(&session_id) {
            Ok(_) => Ok(CallToolResult::success(vec![Content::text("Session stopped.".to_string())])),
            Err(_) => Err(rmcp::Error::invalid_params("Session not found.", None))
        }
    }

    #[tool(description = "Create a new session with the given command line arguments.")]
    async fn start_session(&self, 
        #[tool(param)]
        #[schemars(description = "Command line arguments for the session")]
        argv: Vec<String>) -> Result<CallToolResult, rmcp::Error> {
        let argv = if !argv.is_empty() && argv[0] == "legba" {
            argv[1..].to_vec()
        } else {
            argv
        };

        match self.sessions.write().await.start_new_session("mcp_client".to_string(), argv).await {
            Ok(session_id) => Ok(CallToolResult::success(vec![Content::json(session_id).unwrap()])),
            Err(e) => Err(rmcp::Error::invalid_params(format!("Failed to create session: {}", e), None))
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
