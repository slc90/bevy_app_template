# 当前架构

- `app/`：可执行程序、配置、日志装配与窗口。
- `crates/assets/`：应用 Asset 的统一所有者，公开 `AppAssetPlugin`，可供其他 Crate 使用。资源目录为 `crates/assets/src/assets/`，插件不执行资源加载。
- `crates/test_utils/`：共享测试与 Benchmark 工具，仅供测试或 Benchmark 使用。
- `bevy_widgetry`：Git 默认分支依赖，提供窗口、UI Scale 和 App Logging 等能力。
- `bevy_brp_runtime`：Git 默认分支依赖，提供按配置启用的 BRP 运行时。
