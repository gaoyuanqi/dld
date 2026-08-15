<div align="center">

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![CI](https://github.com/gaoyuanqi/dld/actions/workflows/ci.yml/badge.svg)](https://github.com/gaoyuanqi/dld/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/gaoyuanqi/dld)](https://github.com/gaoyuanqi/dld/releases)
[![Edition](https://img.shields.io/badge/edition-2024-blue.svg)](https://doc.rust-lang.org/edition-guide/rust-2024/)
[![MSRV](https://img.shields.io/badge/MSRV-%E2%89%A51.97.0-blue.svg)](https://releases.rs/)

<b>Q宠大乐斗个人版代玩辅助</b>

</div>

## 安装

### 方式一：一键安装（推荐）

> ⚠️ **安全提示**：切勿直接执行任何未经审查的脚本。强烈建议运行前先浏览器打开链接查看源码。所有安装脚本均开源、无混淆，可放心审计
>
> 🚀 脚本自动优先从 [Gitee 镜像](https://gitee.com/gaoyuanqi/dld) 下载（国内加速），失败自动回退 GitHub，无需额外配置。支持平台：Linux x86_64 / arm64、macOS x86_64 / arm64、Windows x86_64

**Linux / macOS**

```bash
curl -fsSL https://gitee.com/gaoyuanqi/dld/raw/main/install.sh | sh
```

**Windows**

首次使用 PowerShell 执行安装脚本前，可能需要调整执行策略（仅需一次）：

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

然后在 PowerShell 中执行：

```powershell
irm https://gitee.com/gaoyuanqi/dld/raw/main/install.ps1 | iex
```

### 方式二：cargo install

需要安装好 [Rust 工具链](https://rust-lang.org/zh-CN/)（≥1.97.0）

**直接安装（推荐）**

```bash
cargo install --git https://github.com/gaoyuanqi/dld.git --locked
```

> Android 可借助 [Termux](https://termux.dev/) 直接安装

**克隆后安装**

```bash
git clone https://github.com/gaoyuanqi/dld.git
cd dld
cargo install --path . --locked
```

安装后 `dld` 命令可直接使用

### 验证安装

```bash
dld --version
```

输出类似 `dld 0.1.0` 说明安装成功

## 快速开始

### 1. 帮助命令

```bash
dld -h
```

### 2. 登记账号

从浏览器复制大乐斗 Cookie（不要忘记两边加上双引号 `"`）：

```bash
dld 登记 "openId=xxx; accessToken=yyy; newuin=123456"
```

### 3. 执行任务

执行所有任务（每天可以执行任意次）：

```bash
dld 代玩
```

> 不要忘记每天 `13:00` 和 `20:00` 各执行一次

### 同步配置

程序更新后执行此命令同步配置：

```bash
dld 同步配置
```

> 修改配置后也可运行此命令检查格式

### 打印数据目录

```bash
dld 标准目录
```

列出配置文件、日志等数据文件的位置

## 文档

生成并浏览 API 文档：

```bash
cargo doc --open
```

文档分层：

- **`core`** — 核心层：配置管理、HTTP 客户端、日志、账号管理
- **`dw`** — 玩法层：任务调度与各玩法任务实现
- **`cli`** — 命令行解析

## 许可证

[MIT](LICENSE) © 2026 雨园

本项目基于 MIT 许可证开源，可自由使用、修改和分发

## 问题反馈

遇到问题请到 [GitHub Issues](https://github.com/gaoyuanqi/dld/issues) 提交反馈

## 常见问题

### 如何卸载？

- **一键脚本安装**：删除 `~/.local/bin/dld`（Windows 上为 `%USERPROFILE%\.local\bin\dld.exe`）
- **cargo 安装**：`cargo uninstall dld`

### 两种方式都装了，执行的是哪个？

Shell 按 `PATH` 中目录的先后顺序查找，先找到的先执行。查看当前调用的路径：

```bash
# macOS / Linux
which dld

# Windows (CMD)
where dld

# Windows (PowerShell)
(Get-Command dld).Source
```

### 如何获取大乐斗Cookie

大乐斗文字版链接：

```
https://dld.qzapp.z.qq.com/qpet/cgi-bin/phonepk?zapp_uin=&sid=&channel=0&g_ut=1&cmd=index
```

如果有电脑，可以从浏览器开发者工具里面直接复制

还有一个更便捷的方式，以安卓为例：

1、首先应用商店安装 `Via浏览器` 并将其设为默认浏览器

2、然后使用Via访问大乐斗文字版链接，选择 `一键登录`，不要选择账号密码登录

3、成功登录后等待5秒，Via左上角会出现一个类似 `✓` 的图标，点击它

4、可以看到一个 `查看cookies`，复制里面的cookie即可

### 大乐斗 JSON 接口域名

程序只使用json接口域名：

```
https://fight.pet.qq.com/cgi-bin/petpk?
```

程序不会使用文字版域名：

```
https://dld.qzapp.z.qq.com/qpet/cgi-bin/phonepk?
```

而且它们的 `cmd=` 参数并不总是一致

### 大乐斗Cookie失效

例子：

```
00:00:00 | 123456 | xx：(-5) 登陆校验失败，建议使用微端或其他浏览器登录游戏
```

不一定失效，可能服务端误判或者极个别任务返回 `-5` 状态码，应该重新执行命令来确认

### 解析响应失败

例子：

```
00:00:00 | 123456 | xx：解析响应失败
```

原因：系统繁忙、等级太低任务未开放、帮派门派问题

### JSON 解析失败

例子：

```
00:00:00 | 123456 | xx：JSON 解析失败
```

原因：大乐斗返回的json不合法
