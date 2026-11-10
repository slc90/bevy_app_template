# 当前架构

- `app/`：可执行程序、配置、日志装配、窗口与应用专属 Asset。
- `crates/test_utils/`：共享测试与 Benchmark 工具，仅供测试或 Benchmark 使用。
- `bevy_widgetry`：Git 依赖，提供窗口、UI Scale 和 App Logging 等能力。
- `bevy_brp_runtime`：Git 依赖，提供按配置启用的 BRP 运行时。
