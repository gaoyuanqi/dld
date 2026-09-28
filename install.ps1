#!/usr/bin/env pwsh
#requires -Version 5.1

param(
    # 如需走镜像，传入 -Mirror "https://ghproxy.com/"
    [string]$Mirror = "",
    # 设置 $false 可跳过 PATH 自动配置
    [bool]$ModifyPath = $true,
    # 安装指定版本，默认 latest
    [string]$Version = "latest",
    # 跳过 SHA256 校验（风险自负）
    [switch]$SkipVerify
)

$Repo = "gaoyuanqi/dld"
$Gitee = "https://gitee.com"
$Github = if ($Mirror) { "$($Mirror.TrimEnd('/'))/https://github.com" } else { "https://github.com" }

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
            Invoke-WebRequest -Uri $url -OutFile $TmpFile -UseBasicParsing -TimeoutSec 120
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

    # === SHA256 校验 ===
    if (-not $SkipVerify) {
        # 校验文件仅从 GitHub 直连下载（独立信任根，不走 Mirror 代理）
        if ($version -eq "latest") {
            $SumsUrl = "https://github.com/${Repo}/releases/latest/download/SHA256SUMS"
        } else {
            $SumsUrl = "https://github.com/${Repo}/releases/download/${version}/SHA256SUMS"
        }
        Write-Host "下载校验文件: ${SumsUrl}"
        try {
            $Sums = Invoke-WebRequest -Uri $SumsUrl -UseBasicParsing -TimeoutSec 60
        } catch {
            throw "校验文件下载失败，无法验证二进制来源。跳过校验（风险自负）：-SkipVerify"
        }

        $Line = ($Sums.Content -split "`n") | Where-Object { $_.Trim() -like "*dld-windows-${arch}.exe" } | Select-Object -First 1
        # 期望与实际哈希统一小写，不依赖比较运算符的大小写语义
        $Expected = if ($Line) { ($Line.Trim() -split '\s+')[0].ToLower() } else { "" }
        $Actual = (Get-FileHash -Algorithm SHA256 $TmpFile).Hash.ToLower()
        if (-not $Expected) {
            throw "SHA256 校验失败：校验文件中无 dld-windows-${arch}.exe 条目。跳过校验（风险自负）：-SkipVerify"
        }
        if ($Expected -ne $Actual) {
            throw "SHA256 校验失败，二进制可能被篡改或损坏。期望：${Expected}，实际：${Actual}。跳过校验（风险自负）：-SkipVerify"
        }
        Write-Host "SHA256 校验通过"
    } else {
        Write-Host "已跳过 SHA256 校验（-SkipVerify）"
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
    # 下载失败或校验失败时清理临时文件；安装成功后 TmpFile 已 Move 走，此处自然跳过
    if (Test-Path $TmpFile) {
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
    Write-Host "      或本次直接运行：powershell -ExecutionPolicy Bypass -File .\install.ps1"
}
