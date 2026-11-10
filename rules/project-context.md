# 项目背景与目标环境

## 目标平台

- 操作系统：Windows 64 位。
- Rust Target：`x86_64-pc-windows-msvc`。
- 图形后端：DX12（wgpu backend）。
- 命令执行环境：Windows 下统一使用 PowerShell 7（`pwsh`），不使用 Windows PowerShell 5（`powershell.exe`）。

工程设计、实现和验证均以此环境为前提。

项目不以跨平台兼容为当前目标。除非任务明确要求改变平台支持范围，否则：

- 不为 Linux、macOS、Web 或 32 位系统增加兼容实现、fallback、stub 或条件分支。
- 不为假设中的其他平台、Target 或图形后端增加抽象层、依赖、Cargo Feature 或额外配置。
- 不为了保持非 Windows 平台可编译而增加当前需求不需要的代码。
- 可以直接使用 Windows 平台能力，不必为唯一支持的平台额外包裹 `#[cfg(windows)]`。

这些限制不意味着禁止通用代码或条件编译。没有额外维护成本的跨平台实现可以保留；当前需求确实需要的条件编译也可以使用。

**不为假设中的未来需求增加复杂度。** 如果以后需要支持其他平台、Target 或图形后端，应根据届时的实际需求重新设计，而不是提前预留兼容结构。

## 编译期平台检查

每个 Workspace crate 的入口必须包含编译期 Windows x64 检查：

```rust
#[cfg(not(all(target_os = "windows", target_pointer_width = "64")))]
compile_error!("This application supports Windows x64 only");
```

Library 在 `lib.rs` 中检查，Application 在 `main.rs` 中检查，不在内部 Module 重复添加。

此检查用于明确拒绝不支持的 Target，不要求提供替代实现。

## 项目定位与兼容性原则

项目主要供作者个人学习和使用，不以发布稳定公共 API 为目标。

- 各 crate 不需要发布到 crates.io。
- 不要求保持 API 向后兼容，也不为历史版本建立兼容层、弃用期或迁移方案。
- 允许为合理设计直接进行破坏性重构，以设计合理性、简洁和可维护性为优先。
- 只需保证在当前明确支持的 Bevy 版本下正常工作，不承担旧版 Bevy 兼容义务。
