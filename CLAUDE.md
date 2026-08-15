# 项目概述

Q宠大乐斗个人版代玩辅助

# 沟通方式

- 所有输出使用中文
- 不明确的问题要提问，不做猜测
- 改代码前先对齐：用户描述问题或提需求时，先复述理解并给出方案（附理由），等用户明确同意再动手；收到「加上」「提交」等明确指令时直接执行

# Rust 规范

- 禁止 `unsafe`、`.unwrap()`、`.expect()`（Cargo.toml 已配置 lint），测试除外
- 错误处理统一用 `anyhow::Result`，上下文附加用 `.context()`，提前返回用 `anyhow::bail!()`
- 优先用 `let-else` 和 `?` 提前返回，避免深层嵌套
- 注释和文档字符串使用中文
- 文档字符串（`///`、`//!`）末尾不加中文句号

# 关键模式

## HTTP 请求

- 大乐斗接口返回 GBK 编码 JSON，解码用 `encoding_rs::GBK.decode()`
- 接口返回码 `result = "-5"` → Cookie 失效；`result = "-10086"` → 系统维护

## 配置管理

### 全局配置 vs 账号配置

- **全局配置**：`<data_dir>/global_config.json`，所有账号共享（如兑换码、八卦迷阵方向）
- **账号配置**：`<data_dir>/config/<qq>.json`，每 QQ 独享（如矿洞楼层、模式）

### 新增配置

1. 定义配置结构体，派生 `Debug, Default, Deserialize, Serialize`，加 `#[serde(default)]`
2. `impl UpdatableConfig for XxxConfig`，实现 `section_title()`
3. 需要字段取值范围校验时 override `validate()`，范围校验用 `validate_range!` 宏，其余用 `bail!()`
4. 子结构体的 `validate()` 不会自动被调，需在父结构体的 `validate()` 里手动 `self.xxx.validate()?;`
5. `load`/`update`/`create_default` 由 trait 默认实现提供
6. `dld 同步配置` 同时更新全局配置和所有已登记账号的配置

### with_context vs bail

- `.with_context()`：给底层错误附加上下文，不丢原始信息。用于 I/O、解析失败
- `bail!()`：自己造新错误直接返回。用于业务逻辑判断

## 任务执行

- 不在启动时预检 Cookie，由任务首次 HTTP 请求自然触发失效检测
- 遇 Cookie 失效或系统维护自动 bail，终止当前账号后续任务
- 任务执行完成后统一打印成功/失败统计和耗时：`5/5 个账号全部执行成功，耗时 12.3s`
- 失败时显示 QQ 和错误原因，便于定位
- 任务完成后自动清理过期日志（按 `日志保留天数`）

## Gitee 镜像

- `gitee-mirror.yml`：push main 自动同步代码到 Gitee 镜像
- `release.yml` 的 gitee-sync job：发版自动同步 Release + 5 平台附件（并行上传）
- Gitee 无 latest 下载路由，install.sh 用 Gitee 列表 API 解析最新版本号
- Gitee API 偶发瞬时 404，workflow 中清理类步骤保持 best-effort 容错
- Gitee 侧 issue/PR 已关闭，反馈引导至 GitHub Issues

# Git Commit 规范

```
type: 中文描述
```

| type | 用途 |
|------|------|
| `feat` | 新功能 |
| `fix` | 修复 bug |
| `refactor` | 重构（不改变功能行为） |
| `docs` | 文档变更 |
| `chore` | 杂务 |
| `perf` | 性能优化 |

- 标题不超过 50 字符，中文描述

# 发布流程

- 用户说「发版」时，先提议版本号（默认补丁位 +1，如 0.1.0 → 0.1.1），用户确认后执行
- 改 `Cargo.toml` 版本号，`cargo check` 同步 `Cargo.lock`
- 执行 `cargo fmt && cargo clippy -- -D warnings && cargo test`
- 提交 `chore: 版本号 x.y.z`，打 tag `vx.y.z` 并推送 main 与 tag
- tag 与版本号必须一致（release.yml 会校验），推 tag 即触发公开 Release，推送前需用户确认

# 代码实现

- 每次改动后执行 `cargo fmt && cargo clippy -- -D warnings && cargo test`，警告视为错误

# TDD

- 新功能/缺陷修复：先写失败测试（red），再写最小实现（green），一次一个切片
- 纯重构不属于 red-green 循环：先补测试锁定行为，重构后测试保持绿色
- 测试只走公共接口，不测私有实现
- 适用范围：core/ 模块和任务中提取的纯逻辑（解析、筛选、判定、校验）
- 不适用范围：HTTP 编排主流程和接口实际行为，靠实测验证

# 代码风格

- 可见性最小化：`publish = false`，模块间共享用 `pub(crate)`，不对 crate 外暴露
- 配置字段优先用中文标识符与 JSON 键名一致；键名含特殊字符（`*`、`type` 等）时用英文标识符加 `#[serde(rename)]`；其余代码遵循 Rust 命名惯例
- 不轻易加第三方依赖，优先标准库
- 测试内联在文件底部 `#[cfg(test)] mod tests`
- install.sh 保持 POSIX sh：trap 清理函数须保证退出码为 0（用 if 包裹），否则 dash 会把 trap 退出码当作脚本退出码
