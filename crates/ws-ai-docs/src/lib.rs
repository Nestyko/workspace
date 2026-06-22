use ws_core::command::CommandRegistry;

pub fn generate_command_docs(registry: &CommandRegistry) -> String {
    let mut docs = String::new();
    docs.push_str("# AI Command API Documentation\n\n");
    docs.push_str("This document defines the list of typed JSON commands supported by the AI workspace interface.\n\n");

    let mut commands = registry.list();
    commands.sort_by(|a, b| a.id().cmp(b.id()));

    for cmd in commands {
        docs.push_str(&format!("## `{}`\n\n", cmd.id()));
        docs.push_str(&format!("**Description:** {}\n\n", cmd.description()));

        // Input schema
        docs.push_str("### Input Schema\n\n");
        let input_schema = cmd.input_schema();
        if let Ok(pretty) = serde_json::to_string_pretty(&input_schema) {
            docs.push_str(&format!("```json\n{}\n```\n\n", pretty));
        } else {
            docs.push_str("*(No input schema available)*\n\n");
        }

        // Output schema
        docs.push_str("### Output Schema\n\n");
        let output_schema = cmd.output_schema();
        if let Ok(pretty) = serde_json::to_string_pretty(&output_schema) {
            docs.push_str(&format!("```json\n{}\n```\n\n", pretty));
        } else {
            docs.push_str("*(No output schema available)*\n\n");
        }

        docs.push_str("---\n\n");
    }

    docs
}
