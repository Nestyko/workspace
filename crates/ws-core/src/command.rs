use crate::context::CommandContext;
use crate::error::WorkspaceError;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

#[async_trait]
pub trait AiCommand: Send + Sync + 'static {
    const ID: &'static str;
    const DESCRIPTION: &'static str;

    type Input: serde::de::DeserializeOwned
        + serde::Serialize
        + schemars::JsonSchema
        + Send
        + Sync;

    type Output: serde::Serialize
        + schemars::JsonSchema
        + Send
        + Sync;

    async fn run(
        &self,
        ctx: CommandContext,
        input: Self::Input,
    ) -> Result<Self::Output, WorkspaceError>;
}

#[async_trait]
pub trait ErasedAiCommand: Send + Sync {
    fn id(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn input_schema(&self) -> serde_json::Value;
    fn output_schema(&self) -> serde_json::Value;

    async fn run_erased(
        &self,
        ctx: CommandContext,
        input: serde_json::Value,
    ) -> Result<serde_json::Value, WorkspaceError>;
}

#[async_trait]
impl<T> ErasedAiCommand for T
where
    T: AiCommand + Send + Sync,
{
    fn id(&self) -> &'static str {
        Self::ID
    }

    fn description(&self) -> &'static str {
        Self::DESCRIPTION
    }

    fn input_schema(&self) -> serde_json::Value {
        let generator = schemars::gen::SchemaSettings::draft07().into_generator();
        let schema = generator.into_root_schema_for::<T::Input>();
        serde_json::to_value(&schema).unwrap_or(serde_json::Value::Null)
    }

    fn output_schema(&self) -> serde_json::Value {
        let generator = schemars::gen::SchemaSettings::draft07().into_generator();
        let schema = generator.into_root_schema_for::<T::Output>();
        serde_json::to_value(&schema).unwrap_or(serde_json::Value::Null)
    }

    async fn run_erased(
        &self,
        ctx: CommandContext,
        input: serde_json::Value,
    ) -> Result<serde_json::Value, WorkspaceError> {
        let parsed_input: T::Input = serde_json::from_value(input)
            .map_err(|e| WorkspaceError::Validation(format!("Invalid input schema: {}", e)))?;
        let output = self.run(ctx, parsed_input).await?;
        let serialized_output = serde_json::to_value(output)?;
        Ok(serialized_output)
    }
}

pub struct CommandRegistry {
    commands: HashMap<String, Arc<dyn ErasedAiCommand>>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    pub fn register<T: AiCommand>(&mut self, command: T) {
        let id = T::ID.to_string();
        self.commands.insert(id, Arc::new(command));
    }

    pub fn register_dyn(&mut self, command: Arc<dyn ErasedAiCommand>) {
        self.commands.insert(command.id().to_string(), command);
    }

    pub fn get(&self, id: &str) -> Option<&Arc<dyn ErasedAiCommand>> {
        self.commands.get(id)
    }

    pub fn list(&self) -> Vec<Arc<dyn ErasedAiCommand>> {
        self.commands.values().cloned().collect()
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}
