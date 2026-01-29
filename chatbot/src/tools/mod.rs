mod nmap_normal_scan_tool;
mod advanced_nmap_tool;
mod openvas_get_version_tool;
mod openvas_list_configs_tool;
mod simple_echo_tool;

use crate::ToolRegistry;

/// Register all tools that this MCP server exposes.
pub fn register_all_tools(registry: &mut ToolRegistry) {
    registry.register(simple_echo_tool::EchoTool);
    registry.register(nmap_normal_scan_tool::NmapOpenPortsTool);
    registry.register(advanced_nmap_tool::AdvancedNmapTool);
    registry.register(advanced_nmap_tool::QuickScanTool);
    registry.register(advanced_nmap_tool::StealthScanTool);
    registry.register(openvas_get_version_tool::OpenVASGetVersionTool);
    registry.register(openvas_list_configs_tool::OpenVASListConfigsTool);
}

