//! 大乐斗 HTTP 客户端：发起 GET 请求，自动处理 GBK 解码和 Cookie 状态
//!
//! # 关键行为
//!
//! - 响应为 GBK 编码 JSON，自动解码为 UTF-8 后反序列化
//! - 自动修复接口脏数据（尾随逗号、字符串内非法转义）
//! - 解析失败时错误信息附带错误位置附近的响应片段
//! - 接口返回 `result = "-5"` → 标记 Cookie 失效
//! - 接口返回 `result = "-10086"` → 标记系统维护

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use reqwest::Client;
use reqwest::header;
use serde::Deserialize;
use serde_json::Deserializer;
use tokio::time;

const DALEDOU_BASE_URL: &str = "https://fight.pet.qq.com/cgi-bin/petpk?";
const DALEDOU_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/144.0.0.0 Safari/537.36 Edg/144.0.0.0";

/// 创建带超时和 User-Agent 的 HTTP 客户端
pub fn default_http_client() -> Result<Client> {
    let client = match Client::builder()
        .user_agent(DALEDOU_USER_AGENT)
        .timeout(Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(e) => bail!("创建 HTTP 客户端失败: {e}"),
    };
    Ok(client)
}

pub struct DaLeDouClient {
    cookie: String,
    client: Client,
    maintenance: Arc<AtomicBool>,
    invalid: Arc<AtomicBool>,
}

impl DaLeDouClient {
    /// 创建大乐斗 HTTP 客户端
    pub fn new(client: &Client, cookie: &str) -> Self {
        Self {
            cookie: cookie.to_string(),
            client: client.clone(),
            maintenance: Arc::new(AtomicBool::new(false)),
            invalid: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 发起 GET 请求并返回原始 JSON，自动处理 GBK 解码、重试和状态检测
    pub async fn get(&self, cmd_path: &str) -> Result<serde_json::Value> {
        if self.invalid.load(Ordering::SeqCst) {
            bail!("Cookie 已失效");
        }
        if self.maintenance.load(Ordering::SeqCst) {
            bail!("大乐斗维护中");
        }

        let mut attempt = 0u8;
        let json = loop {
            attempt += 1;
            let url = format!("{}{}", DALEDOU_BASE_URL, cmd_path);
            let response = match self
                .client
                .get(&url)
                .header(header::COOKIE, &self.cookie)
                .send()
                .await
            {
                Ok(r) => r,
                Err(e) => bail!("HTTP 请求失败 [{url}]: {e}"),
            };

            let status = response.status();
            if !status.is_success() {
                bail!("接口返回非2xx状态码：状态码={}，URL={}", status, url);
            }

            let bytes = response.bytes().await?;
            let json = match decode_gbk_to_json(&bytes) {
                Ok(v) => v,
                Err(e) => bail!("JSON 解析失败 [{url}]: {e}"),
            };
            let result_code = json.get("result").and_then(|v| v.as_str());

            if let Some("-2") = result_code {
                let msg = json.get("msg").and_then(|v| v.as_str()).unwrap_or("");
                if msg.contains("系统繁忙") && attempt < 3 {
                    time::sleep(Duration::from_millis(100)).await;
                    continue;
                }

                break json;
            }

            // {"msg":"登陆校验失败，建议使用微端或其他浏览器登录游戏","result":"-5"}
            if let Some("-5") = result_code {
                let msg = json
                    .get("msg")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Cookie 无效！");
                // 结拜领斗币返回 "-5"
                if msg != "你已经领取过本届斗币福利！" {
                    self.invalid.store(true, Ordering::SeqCst);
                    bail!("(-5) {msg}");
                }
            }

            if let Some("-10086") = result_code {
                self.maintenance.store(true, Ordering::SeqCst);
                let msg = json
                    .get("msg")
                    .and_then(|v| v.as_str())
                    .unwrap_or("大乐斗维护中！");
                bail!("(-10086) {msg}");
            }

            break json;
        };

        Ok(json)
    }

    /// 验证 Cookie 是否有效（向大乐斗服务器发起校验请求）
    pub async fn verify_cookie(&self) -> Result<()> {
        #[derive(Deserialize)]
        struct LoginCheck {
            ret: i64,
            msg: String,
            duration: u64,
        }

        // { "context": "null", "duration": 0, "guestDuration": 0, "instructions": [ ], "msg": "success", "ret": 0, "traceId": "xxxxx" }
        let json = self.get("cmd=limit&op=login").await?;
        let check: LoginCheck = match serde_json::from_value(json) {
            Ok(v) => v,
            Err(e) => bail!("登录校验响应格式异常: {e}"),
        };

        if check.ret != 0 {
            bail!("（{}）意外情况：{}", check.ret, check.msg);
        }

        if check.duration > 0 {
            bail!("防沉迷限制（剩余 {} 秒）", check.duration);
        }

        Ok(())
    }

    /// Cookie 是否已失效（`result = "-5"`）
    pub fn is_invalid(&self) -> bool {
        self.invalid.load(Ordering::SeqCst)
    }

    /// 大乐斗是否处于维护状态（`result = "-10086"`）
    pub fn is_maintenance(&self) -> bool {
        self.maintenance.load(Ordering::SeqCst)
    }
}

/// 修复接口返回的脏数据 JSON
///
/// - 删除字符串外的尾随逗号（如 `{"a":1,}`）
/// - 字符串内非法转义（如 `"\n"` 漏写 n 只剩 `"\"`）补一个反斜杠成为字面量
///
/// 合法内容原样保留；没有脏数据时输出与输入完全相同
pub(crate) fn repair_dirty_json(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;

    while let Some(c) = chars.next() {
        if in_string {
            out.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                // 合法转义序列保持原样；非法转义（接口偶发丢失转义字符，
                // 如 "\n" 漏写 n）补一个反斜杠，让原反斜杠成为字面量
                let legal = matches!(
                    chars.peek(),
                    Some('"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't' | 'u')
                );
                if legal {
                    escaped = true;
                } else {
                    out.push('\\');
                }
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }

        match c {
            '"' => {
                in_string = true;
                out.push(c);
            }
            ',' => {
                // 向前跳过空白：紧跟 } 或 ] 则为尾随逗号，丢弃；否则保留
                let mut lookahead = chars.clone();
                match lookahead.find(|n| !n.is_whitespace()) {
                    Some('}' | ']') => {}
                    _ => out.push(c),
                }
            }
            _ => out.push(c),
        }
    }

    out
}

/// 截取错误位置（1-based 行列）前后各 20 个字符的响应片段，用于错误信息预览
///
/// column 按字符计（与 serde_json 一致），多字节字符不会被截半；
/// 片段两侧被截断时以省略号标记
pub(crate) fn preview_around(text: &str, line: usize, column: usize) -> String {
    // 定位到第 line 行
    let target_line = text.lines().nth(line.saturating_sub(1)).unwrap_or("");
    // 错误列对应的字节索引，clamp 到行内
    let col_idx = target_line
        .char_indices()
        .nth(column.saturating_sub(1))
        .map(|(i, _)| i)
        .unwrap_or(target_line.len());
    // 向前第 20 个字符的字节索引（不足则从行首开始）
    let start = target_line[..col_idx]
        .char_indices()
        .nth_back(19)
        .map(|(i, _)| i)
        .unwrap_or(0);
    // 向后第 20 个字符的字节索引（不足则到行尾）
    let end = target_line[col_idx..]
        .char_indices()
        .nth(20)
        .map(|(i, _)| col_idx + i)
        .unwrap_or(target_line.len());

    let snippet: String = target_line[start..end]
        .chars()
        .map(|c| if matches!(c, '\n' | '\r') { ' ' } else { c })
        .collect();
    format!(
        "{}{}{}",
        if start > 0 { "…" } else { "" },
        snippet,
        if end < target_line.len() { "…" } else { "" }
    )
}

fn decode_gbk_to_json(bytes: &[u8]) -> Result<serde_json::Value> {
    let (text, _enc, _) = encoding_rs::GBK.decode(bytes);
    // 服务端有时返回多个 JSON 对象粘在一起，用流式解析器只取第一个
    match serde_json::Value::deserialize(&mut Deserializer::from_str(&text)) {
        Ok(value) => Ok(value),
        // 接口偶有尾随逗号等脏数据：清洗后重试一次
        Err(err) => {
            let cleaned = repair_dirty_json(&text);
            serde_json::Value::deserialize(&mut Deserializer::from_str(&cleaned)).with_context(
                || {
                    // 附上错误位置附近的原始响应片段，便于收集新的脏数据样本
                    format!(
                        "原始响应片段：{}",
                        preview_around(&text, err.line(), err.column())
                    )
                },
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{decode_gbk_to_json, preview_around, repair_dirty_json};

    // 对象尾随逗号应被删除
    #[test]
    fn test_strip_trailing_comma_in_object() {
        assert_eq!(repair_dirty_json(r#"{"a":1,}"#), r#"{"a":1}"#);
    }

    // 数组尾随逗号应被删除
    #[test]
    fn test_strip_trailing_comma_in_array() {
        assert_eq!(repair_dirty_json("[1,2,]"), "[1,2]");
    }

    // 尾随逗号夹空白也应被删除，空白保留
    #[test]
    fn test_strip_trailing_comma_with_whitespace() {
        assert_eq!(repair_dirty_json("{\"a\":1 , \n}"), "{\"a\":1  \n}");
    }

    // 字符串内的 ",}" 不是尾随逗号，不应误删
    #[test]
    fn test_keep_comma_inside_string() {
        assert_eq!(repair_dirty_json(r#"{"msg":"a,}b"}"#), r#"{"msg":"a,}b"}"#);
    }

    // 转义引号后的逗号仍在字符串内，不应误删
    #[test]
    fn test_keep_comma_after_escaped_quote() {
        let text = r#"{"msg":"a\",}x"}"#;
        assert_eq!(repair_dirty_json(text), text);
    }

    // 双反斜杠转义结束后，字符串外的尾随逗号仍应被删除
    #[test]
    fn test_strip_comma_after_double_backslash() {
        assert_eq!(repair_dirty_json(r#"{"msg":"a\\",}"#), r#"{"msg":"a\\"}"#);
    }

    // 正常 JSON 原样保留
    #[test]
    fn test_keep_valid_json_unchanged() {
        let text = r#"{"a":1, "b":[1,2], "c":"x,y"}"#;
        assert_eq!(repair_dirty_json(text), text);
    }

    // 连续逗号不在清洗范围内，原样保留（避免静默改变语义）
    #[test]
    fn test_keep_consecutive_commas_unchanged() {
        let text = r#"{"a":1,, "b":2}"#;
        assert_eq!(repair_dirty_json(text), text);
    }

    // 空对象/空数组的尾随逗号删除后成为合法 JSON
    #[test]
    fn test_strip_trailing_comma_in_empty_container() {
        assert_eq!(repair_dirty_json("{,}"), "{}");
        assert_eq!(repair_dirty_json("[,]"), "[]");
    }

    // 字符串内非法转义（如 "\n" 漏写 n 只剩 "\"）应补反斜杠成为字面量
    #[test]
    fn test_fix_invalid_escape_inside_string() {
        assert_eq!(repair_dirty_json(r#"{"msg":"a\qb"}"#), r#"{"msg":"a\\qb"}"#);
    }

    // 合法转义序列原样保留
    #[test]
    fn test_keep_valid_escapes_unchanged() {
        let text = r#"{"msg":"a\nb\tc\"d\\e\u4e2df"}"#;
        assert_eq!(repair_dirty_json(text), text);
    }

    // 字符串外的反斜杠不处理
    #[test]
    fn test_keep_backslash_outside_string_unchanged() {
        let text = r#"{k\q: 1}"#;
        assert_eq!(repair_dirty_json(text), text);
    }

    // ─── decode_gbk_to_json 脏数据回退测试 ───

    // 尾随逗号脏数据：严格解析失败后清洗重试成功
    #[test]
    fn test_decode_gbk_to_json_recovers_trailing_comma() {
        let value = decode_gbk_to_json(b"{\"a\":1,}").unwrap();
        assert_eq!(value["a"], 1);
    }

    // 多对象粘连且带尾随逗号：清洗后仍只取第一个对象
    #[test]
    fn test_decode_gbk_to_json_takes_first_object_with_dirty_data() {
        let value = decode_gbk_to_json(b"{\"a\":1,}{\"b\":2}").unwrap();
        assert_eq!(value, serde_json::json!({"a": 1}));
    }

    // GBK 中文与尾随逗号组合
    #[test]
    fn test_decode_gbk_to_json_gbk_chinese_with_trailing_comma() {
        // {"msg":"大",} 的 GBK 编码
        let bytes: &[u8] = b"{\"msg\":\"\xb4\xf3\",}";
        let value = decode_gbk_to_json(bytes).unwrap();
        assert_eq!(value["msg"], "大");
    }

    // 清洗救不了的输入仍报错，错误信息附带错误位置附近的响应片段
    #[test]
    fn test_decode_gbk_to_json_error_includes_preview() {
        let err = decode_gbk_to_json(b"{\"a\":").unwrap_err();
        let msg = err.to_string();
        // 错误位置在末尾，片段为完整响应（无省略号）
        assert!(
            msg.contains("原始响应片段：{\"a\":"),
            "错误信息应含响应片段：{msg}"
        );
    }

    // 非法转义脏数据：回退后解析成功且反斜杠保留
    #[test]
    fn test_decode_gbk_to_json_fixes_invalid_escape() {
        let value = decode_gbk_to_json(b"{\"msg\":\"line1\\line2\"}").unwrap();
        assert_eq!(value["msg"], "line1\\line2");
    }

    // GBK 中文与漏写 n 的换行转义组合（真实场景：cmd=visit）
    #[test]
    fn test_decode_gbk_to_json_gbk_chinese_with_invalid_escape() {
        // {"msg":"第一行\第二行"} 的 GBK 编码（\ 后原应为 n）
        let bytes: &[u8] = b"{\"msg\":\"\xb5\xda\xd2\xbb\xd0\xd0\\\xb5\xda\xb6\xfe\xd0\xd0\"}";
        let value = decode_gbk_to_json(bytes).unwrap();
        assert_eq!(value["msg"], "第一行\\第二行");
    }

    // ─── preview_around 错误位置片段测试 ───

    // 错误在中间：前后各截 20 字符，两侧加省略号
    #[test]
    fn test_preview_around_middle() {
        let text = "0123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789";
        assert_eq!(
            preview_around(text, 1, 45),
            "…4567890123456789012345678901234567890123…"
        );
    }

    // 错误在开头附近：不截前文，无前省略号
    #[test]
    fn test_preview_around_start() {
        assert_eq!(preview_around("abc", 1, 2), "abc");
    }

    // 错误在末尾附近：不截后文，无后省略号
    #[test]
    fn test_preview_around_end() {
        assert_eq!(preview_around("abc", 1, 3), "abc");
    }

    // column 按字符计：中文等多字节字符不截半
    #[test]
    fn test_preview_around_count_chars_not_bytes() {
        let text = "中文字符测试中文字符测试中文字符测试";
        assert_eq!(preview_around(text, 1, 5), text);
    }

    // 多行响应：按 line 定位到对应行
    #[test]
    fn test_preview_around_multi_line() {
        assert_eq!(preview_around("ab\ncd", 2, 2), "cd");
    }
}
