//! 工具展示示例
//!
//! 展示如何使用 PocketFlow-rs 的各种工具功能

use pocketflow_rs::compose_tools;
use pocketflow_rs::core::NodeId;
use pocketflow_rs::tools::{
    ToolsBuilder,
    configuration::{ConfigManager, ConfigTool, ConfigValue, DeploymentConfig, EnvironmentConfig},
    debugging::{DebugTool, Debugger},
    monitoring::{MonitorTool, PerformanceMonitor},
    persistence::{PersistenceTool, StatePersister},
    visualization::{FlowVisualizer, VisualizationTool},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== PocketFlow-rs 工具展示 ===\n");

    // 1. 调试工具展示
    debug_tool_showcase()?;

    // 2. 监控工具展示
    monitoring_tool_showcase()?;

    // 3. 可视化工具展示
    visualization_tool_showcase()?;

    // 4. 持久化工具展示
    persistence_tool_showcase()?;

    // 5. 配置工具展示
    configuration_tool_showcase()?;

    // 6. 工具组合展示
    composition_showcase()?;

    Ok(())
}

fn debug_tool_showcase() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 调试工具展示");
    println!("================");

    let mut debugger = Debugger::new();
    let node1 = NodeId::new("node_1".to_string());
    let node2 = NodeId::new("node_2".to_string());

    // 设置断点
    debugger.set_breakpoint(&node1);
    debugger.set_breakpoint(&node2);

    println!("✓ 在节点 {} 和 {} 设置断点", node1, node2);

    // 检查断点状态
    println!(
        "✓ 节点 {} 断点状态: {}",
        node1,
        debugger.is_at_breakpoint(&node1)
    );

    // 查看调试状态
    let state = debugger.inspect_state();
    println!("✓ 当前断点数量: {}", state.breakpoints.len());

    // 单步执行
    debugger.step_next();
    println!("✓ 执行单步调试");

    println!("调试信息: {}\n", debugger.get_debug_info());

    Ok(())
}

fn monitoring_tool_showcase() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 监控工具展示");
    println!("================");

    let mut monitor = PerformanceMonitor::new();
    let node1 = NodeId::new("monitored_node_1".to_string());
    let node2 = NodeId::new("monitored_node_2".to_string());

    // 开始监控
    monitor.start_monitoring(&node1);
    println!("✓ 开始监控节点: {}", node1);

    // 模拟一些操作
    std::thread::sleep(std::time::Duration::from_millis(10));

    monitor.start_monitoring(&node2);
    println!("✓ 开始监控节点: {}", node2);

    // 记录自定义指标
    monitor.record_metric("custom_metric", 123.45);
    monitor.record_metric("throughput", 98.7);
    println!("✓ 记录自定义性能指标");

    // 停止监控
    monitor.stop_monitoring(&node1);
    monitor.stop_monitoring(&node2);
    println!("✓ 停止监控节点");

    // 获取性能报告
    let report = monitor.get_performance_report();
    println!("✓ 总执行时间: {:?}", report.total_execution_time);
    println!("✓ 监控的节点数量: {}", report.node_metrics.len());
    println!("✓ 自定义指标数量: {}\n", report.custom_metrics.len());

    Ok(())
}

fn visualization_tool_showcase() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 可视化工具展示");
    println!("==================");

    let mut visualizer = FlowVisualizer::new();
    let node1 = NodeId::new("viz_node_1".to_string());
    let node2 = NodeId::new("viz_node_2".to_string());
    let node3 = NodeId::new("viz_node_3".to_string());

    // 设置节点位置
    visualizer.set_node_position(node1.clone(), 0.0, 0.0);
    visualizer.set_node_position(node2.clone(), 100.0, 0.0);
    visualizer.set_node_position(node3.clone(), 50.0, 100.0);
    println!("✓ 设置节点布局位置");

    // 添加连接
    visualizer.add_connection(node1.clone(), node2.clone());
    visualizer.add_connection(node2.clone(), node3.clone());
    println!("✓ 添加节点连接");

    // 生成流程图 (简化版本，不需要实际的Store)
    let _flow_diagram = "简化的流程图表示";
    println!("✓ 生成流程图 (模拟)");

    // 生成状态图 (简化版本)
    let _state_diagram = "简化的状态图表示";
    println!("✓ 生成状态图 (模拟)\n");

    Ok(())
}

fn persistence_tool_showcase() -> Result<(), Box<dyn std::error::Error>> {
    println!("💾 持久化工具展示");
    println!("==================");

    let mut persister = StatePersister::new();

    // 保存状态
    let checkpoint1 = persister.save_state("启动状态")?;
    println!("✓ 保存检查点: {}", checkpoint1);

    let checkpoint2 = persister.save_state("处理完成")?;
    println!("✓ 保存检查点: {}", checkpoint2);

    // 列出检查点
    let checkpoints = persister.list_checkpoints();
    println!("✓ 检查点列表:");
    for checkpoint in &checkpoints {
        println!("   - {} ({})", checkpoint.name, checkpoint.id);
    }

    // 加载状态
    let state = persister.load_state(&checkpoint1)?;
    println!("✓ 从检查点 {} 加载状态", checkpoint1);
    println!("   节点数量: {}", state.node_states.len());
    println!("   时间戳: {}", state.timestamp);

    // 启用自动检查点
    persister.enable_auto_checkpoint(60);
    println!("✓ 启用自动检查点 (每60秒)");

    // 获取统计信息
    let stats = persister.get_stats();
    println!("✓ 持久化统计:");
    println!("   总保存次数: {}", stats.total_saves);
    println!("   总加载次数: {}", stats.total_loads);
    println!("   检查点数量: {}\n", stats.total_checkpoints);

    Ok(())
}

fn configuration_tool_showcase() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚙️  配置工具展示");
    println!("=================");

    // 基本配置管理
    let mut config = ConfigManager::new();
    config.set_value("custom.setting", ConfigValue::String("value1".to_string()));

    if let Some(value) = config.get_value("flow.max_concurrent_nodes") {
        println!("✓ 获取配置值: flow.max_concurrent_nodes = {}", value);
    }

    let keys = config.get_all_keys();
    println!("✓ 配置键数量: {}", keys.len());

    // 环境配置
    let mut env_config = EnvironmentConfig::new("development");
    if let Some(log_level) = env_config.get_value("logging.level") {
        println!("✓ 开发环境日志级别: {}", log_level);
    }

    env_config.switch_environment("production");
    if let Some(log_level) = env_config.get_value("logging.level") {
        println!("✓ 生产环境日志级别: {}", log_level);
    }

    // 部署配置
    let mut deploy_config = DeploymentConfig::new("local");
    if let Some(workers) = deploy_config.get_value("deploy.workers") {
        println!("✓ 本地部署工作线程: {}", workers);
    }

    deploy_config.switch_deployment("cluster");
    if let Some(workers) = deploy_config.get_value("deploy.workers") {
        println!("✓ 集群部署工作线程: {}", workers);
    }

    // 配置验证
    let validation = config.validate_config();
    println!(
        "✓ 配置验证: {}",
        if validation.is_ok() {
            "通过"
        } else {
            "失败"
        }
    );

    let summary = config.get_config_summary();
    println!("✓ 配置摘要: {} 个配置项\n", summary.total_keys);

    Ok(())
}

fn composition_showcase() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 工具组合展示");
    println!("================");

    // 使用构建器模式组合工具
    let tools = ToolsBuilder::new()
        .with_debugging(Debugger::new())
        .with_monitoring(PerformanceMonitor::new())
        .with_visualization(FlowVisualizer::new())
        .with_persistence(StatePersister::new())
        .with_configuration(ConfigManager::new())
        .build();

    println!("✓ 使用构建器创建工具组合");

    // 检查工具可用性
    if let Some(_debugger) = tools.debugger() {
        println!("✓ 调试工具 - 可用");
    }

    if let Some(_monitor) = tools.monitor() {
        println!("✓ 监控工具 - 可用");
    }

    if let Some(_visualizer) = tools.visualizer() {
        println!("✓ 可视化工具 - 可用");
    }

    if let Some(_persister) = tools.persister() {
        println!("✓ 持久化工具 - 可用");
    }

    if let Some(_config) = tools.config() {
        println!("✓ 配置工具 - 可用");
    }

    // 使用宏创建工具组合
    let _macro_tools = compose_tools!(
        with_debugging(Debugger::new()),
        with_monitoring(PerformanceMonitor::new())
    );

    println!("✓ 使用宏创建部分工具组合\n");

    println!("🎉 所有工具演示完成！");
    println!("====================================");
    println!("PocketFlow-rs 工具模块现在已完全实现，包括：");
    println!("- 调试工具 (断点、单步执行、状态检查)");
    println!("- 监控工具 (性能监控、资源跟踪、指标记录)");
    println!("- 可视化工具 (流程图、状态图、执行跟踪)");
    println!("- 持久化工具 (状态保存、检查点管理、流程恢复)");
    println!("- 配置工具 (配置管理、环境配置、部署配置)");
    println!("- 工具组合 (构建器模式、组合容器、便利宏)");

    Ok(())
}
