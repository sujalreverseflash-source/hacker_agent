use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Definition of a prompt argument for MCP `prompts/*` APIs.
#[derive(Debug, Serialize)]
pub struct PromptArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
    #[serde(rename = "schema")]
    pub schema: Value,
}

/// Definition of a prompt for MCP `prompts/*` APIs.
#[derive(Debug, Serialize)]
pub struct PromptDef {
    pub name: String,
    pub description: String,
    pub arguments: Vec<PromptArgument>,
}

/// Parameters for `prompts/get`.
#[derive(Debug, Deserialize)]
pub struct PromptGetParams {
    pub name: String,
    #[serde(default)]
    pub arguments: Value,
}

/// Return all built-in prompts exposed by this MCP server.
pub fn list_prompts() -> Vec<PromptDef> {
    vec![PromptDef {
        name: "explain_openvas_scan_configs".to_string(),
        description:
            "Explain OpenVAS scan configurations returned by openvas_list_scan_configs in a structured, human-readable way."
                .to_string(),
        arguments: vec![
            PromptArgument {
                name: "configs_json".to_string(),
                description:
                    "The JSON object returned by the openvas_list_scan_configs tool (its `output` field)."
                        .to_string(),
                required: true,
                schema: json!({ "type": "object" }),
            },
            PromptArgument {
                name: "user_goal".to_string(),
                description:
                    "Optional short description of what the user wants (e.g. 'quick production check')."
                        .to_string(),
                required: false,
                schema: json!({ "type": "string" }),
            },
        ],
    }]
}

/// Look up a prompt by name and return a full prompt object including messages.
pub fn get_prompt(name: &str, _arguments: Value) -> Result<Value> {
    let prompts = list_prompts();
    let def = prompts
        .into_iter()
        .find(|p| p.name == name)
        .ok_or_else(|| anyhow::anyhow!(format!("Unknown prompt: {name}")))?;

    // Messages follow the MCP prompt message shape: role + content array.
    let messages = json!([
        {
            "role": "user",
            "content": [
                {
                    "type": "text",
                    "text":
"You are helping a user understand OpenVAS / Greenbone scan configurations.\n\n\
You are given the JSON result from the MCP tool `openvas_list_scan_configs` \
in the argument `configs_json`.\n\n\
1. Treat `configs_json.configs` as the single source of truth. Do NOT invent configs that are not present.\n\
2. Present the result in this exact Markdown structure:\n\n\
### Available OpenVAS scan configurations\n\n\
For each config in `configs_json.configs` in order:\n\n\
1. <name> (`<id>`)\n\
   - **Summary**: Short plain-English explanation based on the `comment` field.\n\
   - **Typical use cases**: 2–3 bullet points describing when to use this profile.\n\
   - **Scan depth & performance**: One sentence on how deep/fast it is relative to others.\n\
   - **Risk to targets**: One sentence indicating if it is low-risk or may be disruptive.\n\n\
3. After listing all configs, add a section:\n\n\
### Recommendations\n\n\
- **Quick, safe production check**: Recommend 1–2 configs by name and ID and explain why.\n\
- **Deep assessment of a critical system**: Recommend 1–2 configs by name and ID and explain why.\n\
- **First discovery on a new network**: Recommend 1–2 configs by name and ID and explain why.\n\n\
If `user_goal` is provided, prioritize recommendations that match that goal and mention it explicitly.\n\
Use clear language, no XML, and do not show raw JSON."
                }
            ]
        }
    ]);

    Ok(json!({
        "name": def.name,
        "description": def.description,
        "arguments": def.arguments,
        "messages": messages
    }))
}

