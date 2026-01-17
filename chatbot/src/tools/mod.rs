mod nmap_normal_scan_tool;
mod simple_echo_tool;

use crate::ToolRegistry;

/// Register all tools that this MCP server exposes.
pub fn register_all_tools(registry: &mut ToolRegistry) {
    registry.register(simple_echo_tool::EchoTool);
    registry.register(nmap_normal_scan_tool::NmapOpenPortsTool);
}

