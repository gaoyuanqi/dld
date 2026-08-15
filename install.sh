#!/bin/sh
set -eu

# 如需自定义源，设 GITEE="https://gitee.com" GITHUB="https://github.com"
# 设置 MODIFY_PATH=0 可跳过 PATH 自动配置
# MODIFY_PATH=0
# 安装指定版本（默认 latest）
# VERSION=v1.0.0

GITEE="${GITEE:-https://gitee.com}"
GITHUB="${GITHUB:-https://github.com}"
REPO="gaoyuanqi/dld"
INSTALL_DIR="${HOME}/.local/bin"
MODIFY_PATH="${MODIFY_PATH:-1}"
VERSION="${VERSION:-latest}"

echo "Q宠大乐斗代玩辅助 — 一键安装"

# === 检测平台 ===
case "$(uname -s)" in
    Linux)  os="linux" ;;
    Darwin) os="macos" ;;
    *)
        echo "不支持的操作系统: $(uname -s)"
        exit 1
        ;;
esac

# === 检测架构 ===
arch="$(uname -m)"
case "${os}-${arch}" in
    linux-x86_64)                    arch="x86_64" ;;
    linux-arm64|linux-aarch64)       arch="arm64" ;;
    macos-x86_64)                    arch="x86_64" ;;
    macos-arm64|macos-aarch64)       arch="arm64" ;;
    *)
        echo "不支持的平台架构组合: ${os} ${arch}"
        echo "支持的平台: Linux x86_64 / Linux arm64 / macOS x86_64 / macOS arm64"
        exit 1
        ;;
esac

BINARY="dld-${os}-${arch}"

# === 构造下载源（Gitee 优先，GitHub 回退；latest 路由格式两者不同） ===
download_urls=""
if [ "${VERSION}" = "latest" ]; then
    download_urls="${download_urls} ${GITEE}/${REPO}/releases/download/latest/${BINARY}"
    download_urls="${download_urls} ${GITHUB}/${REPO}/releases/latest/download/${BINARY}"
else
    download_urls="${download_urls} ${GITEE}/${REPO}/releases/download/${VERSION}/${BINARY}"
    download_urls="${download_urls} ${GITHUB}/${REPO}/releases/download/${VERSION}/${BINARY}"
fi

echo "平台: ${os} ${arch}"
echo "版本: ${VERSION}"

# === 安装目录 ===
mkdir -p "${INSTALL_DIR}"

# === 下载（临时文件 + 多源回退 + 空文件校验） ===
tmpfile="${INSTALL_DIR}/.dld.tmp.$$"
cleanup_tmp() { if [ -f "${tmpfile}" ]; then rm -f "${tmpfile}"; fi; }
trap cleanup_tmp EXIT

# === 检测下载器 ===
if command -v curl > /dev/null 2>&1; then
    downloader="curl"
elif command -v wget > /dev/null 2>&1; then
    downloader="wget"
else
    echo "请先安装 curl 或 wget"
    exit 1
fi

downloaded=false
for url in ${download_urls}; do
    echo "下载: ${url}"
    if [ "${downloader}" = "curl" ]; then
        curl -fsSL --retry 3 --connect-timeout 10 --max-time 120 -o "${tmpfile}" "${url}" && downloaded=true
    else
        wget -q --tries=3 --timeout=10 -O "${tmpfile}" "${url}" && downloaded=true
    fi
    if [ "${downloaded}" = true ] && [ -s "${tmpfile}" ]; then
        break
    fi
    downloaded=false
    echo "下载失败，尝试下一个源"
done

if [ "${downloaded}" != true ]; then
    echo "错误：所有下载源均失败"
    exit 1
fi

mv "${tmpfile}" "${INSTALL_DIR}/dld"
chmod +x "${INSTALL_DIR}/dld"

# === 添加到 PATH（可选 + 精确匹配去重） ===
if [ "${MODIFY_PATH}" -eq 1 ]; then
    rc_file=""
    shell_name=""

    # 优先检测当前交互式 shell
    if [ -n "${ZSH_VERSION:-}" ]; then
        shell_name="zsh"
    elif [ -n "${BASH_VERSION:-}" ]; then
        shell_name="bash"
    else
        shell_name="$(basename "${SHELL:-/bin/sh}")"
    fi

    case "${shell_name}" in
        zsh)  rc_file="${HOME}/.zshrc" ;;
        bash) rc_file="${HOME}/.bashrc" ;;
        fish)
            rc_file="${HOME}/.config/fish/config.fish"
            mkdir -p "${HOME}/.config/fish"
            line="set -gx PATH \"${INSTALL_DIR}\" \$PATH"
            if ! grep -Fq "${INSTALL_DIR}" "${rc_file}" 2>/dev/null; then
                echo "${line}" >> "${rc_file}"
                echo "已将 ${INSTALL_DIR} 添加到 ${rc_file}（需要重启终端）"
            fi
            ;;
    esac

    # bash / zsh
    if [ -n "${rc_file}" ] && [ "${rc_file}" != "${HOME}/.config/fish/config.fish" ]; then
        line="export PATH=\"${INSTALL_DIR}:\${PATH}\""
        if grep -Fq "${INSTALL_DIR}" "${rc_file}" 2>/dev/null; then
            : # 已配置
        else
            echo "${line}" >> "${rc_file}"
            echo "已将 ${INSTALL_DIR} 添加到 ${rc_file}"
            echo "执行 source ${rc_file} 或重启终端后即可使用"
        fi
    fi
fi

echo ""
echo "✅ 安装完成"
echo "安装路径: ${INSTALL_DIR}/dld"
if ! "${INSTALL_DIR}/dld" --version; then
    echo ""
    echo "警告：dld 执行异常，二进制可能不兼容或文件损坏"
fi

if ! command -v dld > /dev/null 2>&1; then
    echo ""
    echo "提示：${INSTALL_DIR} 不在当前 PATH 中"
    echo "手动添加到 PATH: export PATH=\"${INSTALL_DIR}:\${PATH}\""
fi
