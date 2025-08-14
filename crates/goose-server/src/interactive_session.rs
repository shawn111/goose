use anyhow::{Context, Result};
use goose::agents::types::RetryConfig;
use goose::agents::{Agent, SessionConfig};
use goose::config::{Config, ExtensionConfig, ExtensionConfigManager};
use goose::conversation::Conversation;
use goose::providers::base::Provider;
use goose::providers::create;
use goose::recipe::{Response, SubRecipe};
use goose::session;
use goose::session::Identifier;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use futures::StreamExt;
use goose::agents::AgentEvent;
use goose::conversation::message::{Message, MessageContent};

// Placeholder for rustyline::EditMode, as it's CLI-specific
#[derive(Debug, Clone, Copy)]
pub enum EditMode {
    Emacs,
    Vi,
}

// Placeholder for CompletionCache, will be adapted later if needed
struct CompletionCache {
    prompts: HashMap<String, Vec<String>>,
    prompt_info: HashMap<String, String>, // Simplified for now
    last_updated: Instant,
}

impl CompletionCache {
    fn new() -> Self {
        Self {
            prompts: HashMap::new(),
            prompt_info: HashMap::new(),
            last_updated: Instant::now(),
        }
    }
}

pub enum RunMode {
    Normal,
    Plan,
}

pub struct Session {
    agent: Agent,
    messages: Conversation,
    session_file: Option<PathBuf>,
    completion_cache: Arc<std::sync::RwLock<CompletionCache>>,
    debug: bool,
    run_mode: RunMode,
    scheduled_job_id: Option<String>,
    max_turns: Option<u32>,
    edit_mode: Option<EditMode>,
    retry_config: Option<RetryConfig>,
    // Channel to send messages to the WebSocket client
    tx: mpsc::Sender<String>,
    // Channel to receive messages from the WebSocket client
    rx: mpsc::Receiver<String>,
}

impl Session {
    pub fn new(
        agent: Agent,
        session_file: Option<PathBuf>,
        debug: bool,
        scheduled_job_id: Option<String>,
        max_turns: Option<u32>,
        edit_mode: Option<EditMode>,
        retry_config: Option<RetryConfig>,
        tx: mpsc::Sender<String>,
        rx: mpsc::Receiver<String>,
    ) -> Self {
        let messages = if let Some(session_file) = &session_file {
            session::read_messages(session_file).unwrap_or_else(|e| {
                tracing::warn!("Failed to load message history: {}", e);
                Conversation::new_unvalidated(Vec::new())
            })
        } else {
            Conversation::new_unvalidated(Vec::new())
        };

        Session {
            agent,
            messages,
            session_file,
            completion_cache: Arc::new(std::sync::RwLock::new(CompletionCache::new())),
            debug,
            run_mode: RunMode::Normal,
            scheduled_job_id,
            max_turns,
            edit_mode,
            retry_config,
            tx,
            rx,
        }
    }

    pub async fn interactive(&mut self, initial_prompt: Option<String>) -> Result<()> {
        if let Some(prompt) = initial_prompt {
            self.tx.send(format!("Initial prompt: {}", prompt)).await?;
            // Process the initial prompt with the agent
            let msg = Message::user().with_text(&prompt);
            self.process_message(msg, CancellationToken::default()).await?;
        }

        self.tx.send("Starting interactive session. Type 'exit' to quit.".to_string()).await?;

        loop {
            // Receive message from WebSocket client
            let input = match self.rx.recv().await {
                Some(msg) => msg,
                None => {
                    // Channel closed, client disconnected
                    self.tx.send("Client disconnected.".to_string()).await?;
                    break;
                }
            };

            if input == "exit" {
                self.tx.send("Exiting interactive session.".to_string()).await?;
                break;
            }

            // Process user input with the agent
            let user_message = Message::user().with_text(&input);
            self.push_message(user_message);

            let session_config = self.session_file.as_ref().map(|s| {
                let session_id = session::Identifier::Path(s.clone());
                SessionConfig {
                    id: session_id.clone(),
                    working_dir: std::env::current_dir().unwrap_or_default(),
                    schedule_id: self.scheduled_job_id.clone(),
                    execution_mode: None,
                    max_turns: self.max_turns,
                    retry_config: self.retry_config.clone(),
                }
            });

            let mut stream = self.agent.reply(
                self.messages.clone(),
                session_config.clone(),
                Some(CancellationToken::default()),
            ).await?;

            // Stream AgentEvents back to the client
            while let Some(event) = stream.next().await {
                match event {
                    Ok(AgentEvent::Message(message)) => {
                        // For now, just send the text content of the message
                        for content in message.content {
                            if let MessageContent::Text(text_content) = content {
                                self.tx.send(format!("Agent: {}", text_content.text)).await?;
                            }
                            // TODO: Handle other message content types (ToolRequest, ToolResponse, etc.)
                        }
                    }
                    Ok(AgentEvent::McpNotification((_id, notification))) => {
                        self.tx.send(format!("Notification: {:?}", notification)).await?;
                    }
                    Ok(AgentEvent::HistoryReplaced(new_messages)) => {
                        self.tx.send(format!("History replaced with {} messages.", new_messages.len())).await?;
                        self.messages = Conversation::new_unvalidated(new_messages);
                    }
                    Ok(AgentEvent::ModelChange { model, mode }) => {
                        self.tx.send(format!("Model changed to {} in {} mode.", model, mode)).await?;
                    }
                    Err(e) => {
                        self.tx.send(format!("Agent error: {}", e)).await?;
                        break; // Break on agent error
                    }
                }
            }
        }

        Ok(())
    }

    // Placeholder for push_message
    fn push_message(&mut self, message: goose::conversation::message::Message) {
        self.messages.push(message);
    }

    // Placeholder for process_agent_response (no longer directly used in interactive)
    async fn process_message(
        &mut self,
        message: goose::conversation::message::Message,
        _cancel_token: CancellationToken,
    ) -> Result<()> {
        self.push_message(message);
        Ok(())
    }

    // Placeholder for update_completion_cache
    async fn update_completion_cache(&mut self) -> Result<()> {
        Ok(())
    }

    // Placeholder for display_context_usage
    async fn display_context_usage(&self) -> Result<()> {
        Ok(())
    }

    // Placeholder for handle_prompt_command
    async fn handle_prompt_command(&mut self, _opts: input::PromptCommandOptions) -> Result<()> {
        Err(anyhow::anyhow!("Not implemented"))
    }

    // Placeholder for save_recipe
    fn save_recipe(
        &self,
        _recipe: &goose::recipe::Recipe,
        _filepath_str: &str,
    ) -> anyhow::Result<PathBuf> {
        Err(anyhow::anyhow!("Not implemented"))
    }

    // Placeholder for headless
    pub async fn headless(&mut self, _prompt: String) -> Result<()> {
        Err(anyhow::anyhow!("Not implemented"))
    }

    // Placeholder for render_message_history
    pub fn render_message_history(&self) {
        // Not implemented for server-side
    }

    // Placeholder for get_metadata
    pub fn get_metadata(&self) -> Result<session::SessionMetadata> {
        Err(anyhow::anyhow!("Not implemented"))
    }

    // Placeholder for get_total_token_usage
    pub fn get_total_token_usage(&self) -> Result<Option<i32>> {
        Err(anyhow::anyhow!("Not implemented"))
    }

    // Placeholder for add_extension
    pub async fn add_extension(&mut self, _extension_command: String) -> Result<()> {
        Ok(())
    }

    // Placeholder for add_remote_extension
    pub async fn add_remote_extension(&mut self, _extension_url: String) -> Result<()> {
        Ok(())
    }

    // Placeholder for add_streamable_http_extension
    pub async fn add_streamable_http_extension(&mut self, _extension_url: String) -> Result<()> {
        Ok(())
    }

    // Placeholder for add_builtin
    pub async fn add_builtin(&mut self, _builtin_name: String) -> Result<()> {
        Ok(())
    }

    // Placeholder for list_prompts
    pub async fn list_prompts(
        &mut self,
        _extension: Option<String>,
    ) -> Result<HashMap<String, Vec<String>>> {
        Ok(HashMap::new())
    }

    // Placeholder for get_prompt_info
    pub async fn get_prompt_info(&mut self, _name: &str) -> Result<Option<String>> {
        Ok(None)
    }

    // Placeholder for get_prompt
    pub async fn get_prompt(&mut self, _name: &str, _arguments: serde_json::Value) -> Result<Vec<goose::conversation::message::Message>> {
        Ok(Vec::new())
    }

    // Placeholder for invalidate_completion_cache
    async fn invalidate_completion_cache(&self) {
        // Not implemented for server-side
    }

    // Placeholder for message_history
    pub fn message_history(&self) -> Conversation {
        Conversation::new_unvalidated(Vec::new())
    }
}

// This function will be adapted to send confirmation requests over WebSocket
async fn offer_extension_debugging_help(
    _extension_name: &str,
    error_message: &str,
    _provider: Arc<dyn goose::providers::base::Provider>,
    interactive: bool,
) -> Result<(), anyhow::Error> {
    if !interactive {
        return Ok(())
    }
    // For now, just log the error. In a real implementation, this would send a WebSocket message
    // to the client asking for confirmation and waiting for a response.
    tracing::warn!("Extension debugging help offered for error: {}", error_message);
    Ok(())
}

pub async fn build_session(session_config: SessionBuilderConfig, tx: mpsc::Sender<String>, rx: mpsc::Receiver<String>) -> Result<Session> {
    // Load config and get provider/model
    let config = Config::global();

    let provider_name = session_config
        .provider
        .or_else(|| {
            session_config
                .settings
                .as_ref()
                .and_then(|s| s.goose_provider.clone())
        })
        .or_else(|| config.get_param("GOOSE_PROVIDER").ok())
        .ok_or_else(|| anyhow::anyhow!("No provider configured. Run 'goose configure' first"))?;

    let model_name = session_config
        .model
        .or_else(|| {
            session_config
                .settings
                .as_ref()
                .and_then(|s| s.goose_model.clone())
        })
        .or_else(|| config.get_param("GOOSE_MODEL").ok())
        .ok_or_else(|| anyhow::anyhow!("No model configured. Run 'goose configure' first"))?;

    let temperature = session_config.settings.as_ref().and_then(|s| s.temperature);

    let model_config = goose::model::ModelConfig::new(&model_name)?
        .with_temperature(temperature);

    // Create the agent
    let agent: Agent = Agent::new();

    if let Some(sub_recipes) = session_config.sub_recipes {
        agent.add_sub_recipes(sub_recipes).await;
    }

    if let Some(final_output_response) = session_config.final_output_response {
        agent.add_final_output_tool(final_output_response).await;
    }

    let new_provider = match create(&provider_name, model_config) {
        Ok(provider) => provider,
        Err(e) => {
            return Err(anyhow::anyhow!(
                "Error {}.\n\nPlease check your system keychain and run 'goose configure' again.\n\nIf your system is unable to use the keyring, please try setting secret key(s) via environment variables.\n\nFor more info, see: https://block.github.io/goose/docs/troubleshooting/#keychainkeyring-errors",
                e
            ));
        }
    };
    // Keep a reference to the provider for display_session_info
    let provider_for_display = Arc::clone(&new_provider);

    // Log model information at startup
    if let Some(lead_worker) = new_provider.as_lead_worker() {
        let (lead_model, worker_model) = lead_worker.get_model_info();
        tracing::info!(
            "🤖 Lead/Worker Mode Enabled: Lead model (first 3 turns): {}, Worker model (turn 4+): {}, Auto-fallback on failures: Enabled",
            lead_model,
            worker_model
        );
    } else {
        tracing::info!("🤖 Using model: {}", model_name);
    }

    agent
        .update_provider(new_provider)
        .await?;

    // Configure tool monitoring if max_tool_repetitions is set
    if let Some(max_repetitions) = session_config.max_tool_repetitions {
        agent.configure_tool_monitor(Some(max_repetitions)).await;
    }

    // Handle session file resolution and resuming
    let session_file: Option<std::path::PathBuf> = if session_config.no_session {
        None
    } else if session_config.resume {
        if let Some(identifier) = session_config.identifier {
            let session_file = session::get_path(identifier)?;
            if !session_file.exists() {
                return Err(anyhow::anyhow!(
                    "Cannot resume session {} - no such session exists",
                    session_file.display()
                ));
            }
            Some(session_file)
        } else {
            // Try to resume most recent session
            session::get_most_recent_session().ok()
        }
    } else {
        // Create new session with provided name/path or generated name
        let id = match session_config.identifier {
            Some(identifier) => identifier,
            None => Identifier::Name(session::generate_session_id()),
        };

        // Just get the path - file will be created when needed
        session::get_path(id).ok()
    };

    if session_config.resume {
        if let Some(session_file) = session_file.as_ref() {
            // Read the session metadata
            let metadata = session::read_metadata(session_file)?;

            let current_workdir =
                std::env::current_dir().expect("Failed to get current working directory");
            if current_workdir != metadata.working_dir {
                // For now, just log the warning. In a real implementation, this would send a WebSocket message
                // to the client asking for confirmation and waiting for a response.
                tracing::warn!(
                    "Original working directory was {} but current is {}. User needs to confirm switch.",
                    metadata.working_workdir.display(),
                    current_workdir.display()
                );
            }
        }
    }

    // Setup extensions for the agent
    // Extensions need to be added after the session is created because we change directory when resuming a session
    // If we get extensions_override, only run those extensions and none other
    let extensions_to_run: Vec<_> = if let Some(extensions) = session_config.extensions_override {
        agent.disable_router_for_recipe().await;
        extensions.into_iter().collect()
    } else {
        ExtensionConfigManager::get_all()? 
            .into_iter()
            .filter(|ext| ext.enabled)
            .map(|ext| ext.config)
            .collect()
    };

    for extension in extensions_to_run {
        if let Err(e) = agent.add_extension(extension.clone()).await {
            let err = e.to_string();
            tracing::warn!(
                "Failed to start extension '{}': {}. Continuing without it.",
                extension.name(),
                err
            );

            // Offer debugging help
            if let Err(debug_err) = offer_extension_debugging_help(
                &extension.name(),
                &err,
                Arc::clone(&provider_for_display),
                session_config.interactive,
            )
            .await
            {
                tracing::warn!("Could not start debugging session: {}", debug_err);
            }
        }
    }

    // Determine editor mode - CLI specific, so remove or adapt
    // For now, we'll just use a default or ignore.
    let edit_mode = None; // Removed CLI-specific EditMode logic

    // Create new session
    let mut session = Session::new(
        agent,
        session_file.clone(),
        session_config.debug,
        session_config.scheduled_job_id.clone(),
        session_config.max_turns,
        edit_mode,
        session_config.retry_config.clone(),
        tx,
        rx,
    );

    // Add extensions if provided
    for extension_str in session_config.extensions {
        if let Err(e) = session.add_extension(extension_str.clone()).await {
            tracing::warn!(
                "Failed to start extension '{}': {}. Continuing without it.",
                extension_str, e
            );

            // Offer debugging help
            if let Err(debug_err) = offer_extension_debugging_help(
                &extension_str,
                &e.to_string(),
                Arc::clone(&provider_for_display),
                session_config.interactive,
            )
            .await
            {
                tracing::warn!("Could not start debugging session: {}", debug_err);
            }
        }
    }

    // Add remote extensions if provided
    for extension_str in session_config.remote_extensions {
        if let Err(e) = session.add_remote_extension(extension_str.clone()).await {
            tracing::warn!(
                "Failed to start remote extension '{}': {}. Continuing without it.",
                extension_str, e
            );

            // Offer debugging help
            if let Err(debug_err) = offer_extension_debugging_help(
                &extension_str,
                &e.to_string(),
                Arc::clone(&provider_for_display),
                session_config.interactive,
            )
            .await
            {
                tracing::warn!("Could not start debugging session: {}", debug_err);
            }
        }
    }

    // Add streamable HTTP extensions if provided
    for extension_str in session_config.streamable_http_extensions {
        if let Err(e) = session
            .add_streamable_http_extension(extension_str.clone())
            .await
        {
            tracing::warn!(
                "Failed to start streamable HTTP extension '{}': {}. Continuing without it.",
                extension_str, e
            );

            // Offer debugging help
            if let Err(debug_err) = offer_extension_debugging_help(
                &extension_str,
                &e.to_string(),
                Arc::clone(&provider_for_display),
                session_config.interactive,
            )
            .await
            {
                tracing::warn!("Could not start debugging session: {}", debug_err);
            }
        }
    }

    // Add builtin extensions
    for builtin in session_config.builtins {
        if let Err(e) = session.add_builtin(builtin.clone()).await {
            tracing::warn!(
                "Failed to start builtin extension '{}': {}. Continuing without it.",
                builtin, e
            );

            // Offer debugging help
            if let Err(debug_err) = offer_extension_debugging_help(
                &builtin,
                &e.to_string(),
                Arc::clone(&provider_for_display),
                session_config.interactive,
            )
            .await
            {
                tracing::warn!("Could not start debugging session: {}", debug_err);
            }
        }
    }

    // Add CLI-specific system prompt extension - this needs to be adapted or removed
    // For now, I'll remove the CLI-specific prompt and only use additional_system_prompt.
    // session.agent.extend_system_prompt(super::prompt::get_cli_prompt()).await;

    if let Some(additional_prompt) = session_config.additional_system_prompt {
        session.agent.extend_system_prompt(additional_prompt).await;
    }

    // Only override system prompt if a system override exists
    let system_prompt_file: Option<String> = config.get_param("GOOSE_SYSTEM_PROMPT_FILE_PATH").ok();
    if let Some(ref path) = system_prompt_file {
        let override_prompt = std::fs::read_to_string(path)?;
        session.agent.override_system_prompt(override_prompt).await;
    }

    // Display session information unless in quiet mode - CLI specific, so remove or adapt
    // For now, just log the info.
    if !session_config.quiet {
        tracing::info!(
            "Session Info: resume={}, provider={}, model={}, session_file={:?}",
            session_config.resume,
            provider_name,
            model_name,
            session_file,
        );
    }
    Ok(session)
}