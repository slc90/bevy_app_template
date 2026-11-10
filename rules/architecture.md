# 架构规则

## Workspace / Crate 职责与依赖管理

当前整体设计与模块职责关系见 `docs/design/architecture.md`，模块核心设计见 `docs/design/modules/`。本节约束 crate 的职责边界、依赖方向及依赖管理方式。

### Crate 职责与依赖方向

- `app/`：可运行程序入口、应用业务、窗口和 UI 装配及运行配置，不得作为下层库 crate 的依赖。
- `crates/assets/`：统一管理应用 Asset，可供其他 Crate 使用。
- `crates/test_utils/`：跨 Crate 共享的测试及 Benchmark 基础设施，不能进入应用或其他生产代码的正常依赖路径。
- `crates/` 中将来增加的内部 Library：按实际可交付能力划分。

### 外部依赖管理

能够统一管理的外部依赖，其版本在根目录 `Cargo.toml` 的 `[workspace.dependencies]` 中定义。

Workspace 成员通过 `workspace = true` 引用，不得在多个 Crate 中分别维护同一外部依赖的版本。

各 Crate 可以根据实际需要配置自身使用的 Dependency Feature，不要求启用完全相同的 Feature 集合。

新增依赖必须服务于当前明确需求，不得仅为了实现方便或假设中的未来用途引入依赖。

### 上游问题处理

当问题根源位于 Bevy 或其他上游依赖时，优先通过公开 API、配置调整或其他不修改上游源码的方式解决。

禁止为了修复上游问题而引入或维护修改版上游源码，包括：

- 将上游源码全部或部分复制、Vendor 到当前仓库。
- Fork 上游仓库并自行修改，通过 Cargo Patch、Git Dependency 或其他方式使用修改后的 Fork。

允许正常升级至上游官方发布的修复版本。

如果无法在上述约束内合理解决问题，则保留该问题，不得通过引入修改版上游源码强行修复。

未解决的问题记录在 `docs/known_issues.md`，包含问题现象、涉及的上游依赖及版本、实际影响、已确认的原因，以及暂不解决的理由。未经证实的原因应明确标为推测，不得作为既定事实。

问题不影响的其他任务内容可以继续完成，但必须如实报告相关限制。

### 内部依赖与命名

Workspace 内部 Crate 默认使用 Path Dependency 并遵守必要的职责与依赖方向约束。目录用简洁、能反映职责的名称；Package 名以实际语义为准。

## Assets

应用 Asset 统一由 `crates/assets/` 管理，资源文件位于 `crates/assets/src/assets/`，`crates/assets/src/lib.rs` 公开 `AppAssetPlugin` 负责应用需要的注册。

其他代码通过语义名称引用应用 Asset，不到处散布物理/Embedded 路径；Asset 的注册和路径选择集中在实际的 Asset Owner 中。

新增独立库确实需要自有 Asset 时，根据其职责决定所在位置，不预设统一的跨 Crate Asset Owner 规则。

## 源码与 Module 组织

### Crate 入口

Library 使用 `lib.rs`，Application 使用 `main.rs`。`lib.rs` 主要负责 Module 声明、Re-export 和必要的 Crate 级装配，不堆放大量具体实现。

### Module 组织

Module 根据实际职责和语义划分，不机械采用一种 Type 对应一个文件的结构。

不得仅因为文件较长、希望目录整齐或希望各 crate 结构一致，就拆分或重组 Module。

除 Integration Test 目录外，源码采用现代 Rust Module 文件组织方式，不使用 `mod.rs`：

```text
src/
├── lib.rs
├── foo.rs
└── foo/
    ├── bar.rs
    └── baz.rs
```

`tests/` 下的 Integration Test 可以根据实际需要使用 `mod.rs`。

### Visibility

声明使用满足当前需求的最小 Visibility：

- `private`：仅当前 Module 使用。
- `pub(crate)`：仅 Crate 内共享。
- `pub`：明确需要跨 Crate 访问或属于对外 API。

不得仅为了实现或测试方便而扩大 Visibility，也不得在普通开发任务中顺手增加公共 API。
