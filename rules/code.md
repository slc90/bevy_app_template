# 代码规则

## 主动 Panic

所有 Crate 的非测试代码禁止主动触发 panic，包括内部 Invariant 被破坏或发生无法恢复的错误时。

测试代码允许使用断言、panic、`unwrap()` 和 `expect()`，但测试基础设施仍须正确处理运行期失败。

具体的禁止项及测试例外由 Workspace Lint 配置执行。

## 错误处理与传播

### 错误语义

不得静默吞掉真实错误，也不得以日志记录代替错误传播。

- 当前层能够正确处理的错误，应在当前层处理；无法处理的错误必须向上传播。
- 内部 Invariant 被破坏或发生无法恢复的失败时，必须作为真实错误处理，不得主动触发 Panic，也不得仅记录日志后继续执行。
- 正常的 `Option` 缺失、Asset 等待、Observer 目标过滤、交互 Guard，以及公开 API 明确定义的合法 No-op，不应人为转换为错误。
- 不得为了避免产生 ERROR 日志而将真实失败伪装成正常状态。


## Lint Suppression

只有当 Lint 在当前语义下确实不适用，或存在无法合理消除的必要例外时，才允许使用 `#[allow(...)]` 等局部 Suppression。

- 优先修正产生 Lint 的代码，而不是 Suppress Lint。
- Suppression 必须限制在必要的最小范围。
- 不得通过大范围 Suppression 掩盖实际问题。
- 根目录 `Cargo.toml` 中明确配置的 Workspace 级 Lint 例外不受局部 Suppression 限制。

不强制为每处 Suppression 添加解释性注释；仅当例外原因不明显且有必要说明时，按照 `rules/documentation.md` 记录。

## Magic Number

代码中具有明确业务或算法含义的数字字面量，应提取为具有语义的命名常量或变量，包括功能代码和测试代码。

没有独立语义的数字，例如循环索引、初始值 `0`、简单数学运算中的系数等，可以直接使用，无需提取。

参数化测试中的测试输入与预期结果允许直接使用数字字面量，但具有独立业务或算法含义的数值仍应遵循上述规则。

命名应准确表达数字的实际含义，不要求统一使用 `const`，局部变量同样可以。

## 命名约定

- Bevy Component 类型不追加 `Component` 后缀。
- Bevy Resource 类型不追加 `Resource` 后缀。
- Bevy Plugin 类型使用 `Plugin` 后缀。
- UI Style 数据或解析后 Style 值的类型使用 `Style` 后缀。

这些约定不强加给与 Bevy、UI 无关的后台 Crate。其他命名遵循 Rust 惯例，以语义清晰为准。
