mod nmap_normal_scan_tool;
mod advanced_nmap_tool;
mod openvas_get_version_tool;
mod openvas_list_configs_tool;
mod openvas_create_target_tool;
mod openvas_create_task_tool;
mod openvas_start_task_tool;
mod openvas_task_status_tool;
mod openvas_get_report_tool;
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
    registry.register(openvas_create_target_tool::OpenVASCreateTargetTool);
    registry.register(openvas_create_task_tool::OpenVASCreateTaskTool);
    registry.register(openvas_start_task_tool::OpenVASStartTaskTool);
    registry.register(openvas_task_status_tool::OpenVASTaskStatusTool);
    registry.register(openvas_get_report_tool::OpenVASGetReportTool);
}

