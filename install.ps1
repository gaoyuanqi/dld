#!/usr/bin/env pwsh
#requires -Version 5.1

param(
    # 如需走镜像，传入 -Mirror "https://ghproxy.com/"
    [string]$Mirror = "",
    # 设置 $false 可跳过 PATH 自动配置
    [bool]$ModifyPath = $true,
    # 安装指定版本，默认 latest
    [string]$Version = "latest"
)

$Repo = "gaoyuanqi/dld"
$Gitee = "https://gitee.com"
$Github = if ($Mirror) { "${Mirror}https://github.com" } else { "https://github.com" }

# Gitee 与 GitHub 均支持 latest 下载路由（格式不同），无需解析版本号
$version = $Version

# 检测架构
if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") {
    Write-Host "错误：Windows ARM64 暂不提供预编译包，请使用 cargo install 从源码安装"
    Write-Host "      https://github.com/gaoyuanqi/dld"
    exit 1
}
$arch = "x86_64"

# === 构造下载源（Gitee 优先，GitHub 回退；latest 路由格式两者不同） ===
$urls = @()
if ($version -eq "latest") {
    $urls += "${Gitee}/${Repo}/releases/download/latest/dld-windows-${arch}.exe"
    $urls += "${Github}/${Repo}/releases/latest/download/dld-windows-${arch}.exe"
} else {
    $urls += "${Gitee}/${Repo}/releases/download/${version}/dld-windows-${arch}.exe"
    $urls += "${Github}/${Repo}/releases/download/${version}/dld-windows-${arch}.exe"
}

$InstallDir = "$env:USERPROFILE\.local\bin"
$ExePath = "$InstallDir\dld.exe"

Write-Host "Q宠大乐斗代玩辅助 — 一键安装"
Write-Host "平台: windows ${arch}"
Write-Host "版本: ${version}"

# === 安装目录 ===
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

# === 下载（临时文件 + 多源回退 + 空文件校验） ===
$TmpFile = "$InstallDir\.dld.tmp.$PID"
$downloaded = $false
$originalProgressPreference = $ProgressPreference

try {
    $ProgressPreference = 'SilentlyContinue'
    foreach ($url in $urls) {
        Write-Host "下载: ${url}"
        try {
            Invoke-WebRequest -Uri $url -OutFile $TmpFile -UseBasicParsing
            if ((Test-Path $TmpFile) -and ((Get-Item $TmpFile).Length -gt 0)) {
                $downloaded = $true
                break
            }
        } catch {
            Write-Host "下载失败，尝试下一个源"
        }
    }
    if (-not $downloaded) {
        throw "所有下载源均失败"
    }

    # 处理目标文件被占用的情况（如 dld.exe 正在运行）
    if (Test-Path $ExePath) {
        try {
            Remove-Item -Force $ExePath -ErrorAction Stop
        } catch {
            $BackupPath = "$ExePath.old"
            Remove-Item -Force $BackupPath -ErrorAction SilentlyContinue
            Move-Item -Force $ExePath $BackupPath
        }
    }
    Move-Item -Force $TmpFile $ExePath
} catch {
    Write-Host "错误：$($_.Exception.Message)"
    exit 1
} finally {
    $ProgressPreference = $originalProgressPreference
    if (-not $downloaded -and (Test-Path $TmpFile)) {
        Remove-Item -Force $TmpFile
    }
}

# === 添加到 PATH ===
if ($ModifyPath) {
    $currentPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    $pathEntries = if ($currentPath) { $currentPath -split ';' } else { @() }
    if ($InstallDir -notin $pathEntries) {
        $newPath = if ($currentPath) { "$InstallDir;$currentPath" } else { $InstallDir }
        [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
        Write-Host "已将 $InstallDir 添加到用户 PATH（如果新打开的终端无法识别 dld 命令，请重启终端或重新登录）"
    }
    if ($InstallDir -notin ($env:Path -split ';')) {
        $env:Path = "$InstallDir;$env:Path"
    }
}

Write-Host ""
Write-Host "✅ 安装完成"
Write-Host "安装路径: $ExePath"
& $ExePath --version

if (-not (Get-Command dld -ErrorAction SilentlyContinue)) {
    Write-Host ""
    Write-Host "提示：当前会话无法直接运行 dld，请检查 PATH 是否包含安装目录"
    Write-Host "      若之前执行 install.ps1 遇到权限拦截（.ps1 脚本需要执行策略授权），可执行："
    Write-Host "      Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser"
}
