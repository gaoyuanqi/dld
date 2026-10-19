//! 今日活跃度
//!
//! 回流好友召回、领取个人和帮派总活跃礼包

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "今日活跃度";

const GIFT_CONFIG: &[(u8, u32)] = &[(1, 20), (2, 50), (3, 80), (4, 115)];

#[derive(Deserialize)]
struct Response {
    msg: String,
}

pub async fn run(d: &DaLeDou) {
    let Some(data) = query(d).await else {
        return;
    };

    // daycall 为已召回次数，非法值时打日志并跳过召回
    let remaining = match data.daycall.parse::<u8>() {
        Ok(already_called) => remaining_calls(already_called),
        Err(e) => {
            d.log(TASK, &format!("解析 daycall 字段失败：{e}"));
            0
        }
    };

    // 触发召回会增加活跃度，需重新查询；未触发则复用本次查询结果
    let data = if remaining > 0 {
        回流好友召回(d, remaining).await;
        let Some(data) = query(d).await else {
            return;
        };
        data
    } else {
        data
    };

    if data.giftmarkseq.len() != 32 {
        d.log(
            TASK,
            &format!("giftmarkseq 字段长度不足32位：{}", data.giftmarkseq),
        );
        return;
    }

    let Some(liveness) = parse_liveness(&data.liveness) else {
        d.log(TASK, &format!("liveness 字段格式错误：{}", data.liveness));
        return;
    };

    let mut is_claim = false;
    for id in find_claimable_gifts(liveness, &data.giftmarkseq) {
        领取(d, id).await;
        is_claim = true;
    }

    // 没有领取过个人礼包或者没有加入帮派
    if !is_claim || data.fac_liveness.is_empty() {
        return;
    }

    领取帮派总活跃礼包(d).await;
}

#[derive(Deserialize)]
struct Query {
    result: String,
    msg: String,
    daycall: String,  // 回流好友召回次数
    liveness: String, // 今日活跃度/帮派礼包所需今日活跃度
    #[serde(rename = "facLiveness")]
    fac_liveness: String, // 昨日帮派总活跃度/帮派礼包所需总活跃度
    giftmarkseq: String, // 32位礼包领取标记
}

async fn query(d: &DaLeDou) -> Option<Query> {
    // 今日活跃度
    let data: Query = match d.get("cmd=liveness").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return None;
        }
    };

    if data.result != "0" {
        d.log(TASK, &data.msg);
        return None;
    }

    Some(data)
}

/// 计算剩余可召回次数：每日最多 3 次，已召回 n 次则剩余 3-n
fn remaining_calls(already_called: u8) -> u8 {
    3u8.saturating_sub(already_called)
}

async fn 回流好友召回(d: &DaLeDou, remaining: u8) {
    #[derive(Deserialize)]
    struct Call {
        result: String,
        msg: String,
        canbecall: Vec<CanbeCall>, // 回流好友列表
    }

    #[derive(Deserialize)]
    struct CanbeCall {
        uin: String,
    }

    // 回流好友召回
    let data: Call = match d.get("cmd=callback&subtype=3").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    if data.result != "0" {
        d.log(TASK, &data.msg);
        return;
    }

    let Some(canbecall) = data.canbecall.first() else {
        return;
    };

    for _ in 0..remaining {
        回来玩吧(d, &canbecall.uin).await;
    }
}

async fn 回来玩吧(d: &DaLeDou, uin: &str) {
    // 回来玩吧
    let cmd = format!("cmd=callback&subtype=4&opuin={uin}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

/// 从 liveness 字段提取今日活跃度
/// 支持两种格式：有帮派 `"80/40"`（今日/昨日）和无帮派 `"80"`
fn parse_liveness(s: &str) -> Option<u32> {
    s.split('/').next()?.parse().ok()
}

/// 根据活跃度和 32 位领取标记找出可领取的礼包 ID
/// giftmarkseq 长度不足 32 位时返回空列表
/// 每位 '0' 表示该礼包未领取，'1' 表示已领取
fn find_claimable_gifts(liveness: u32, giftmarkseq: &str) -> Vec<u8> {
    if giftmarkseq.len() != 32 {
        return vec![];
    }
    let mark = giftmarkseq.as_bytes();
    GIFT_CONFIG
        .iter()
        .filter(|&&(id, need)| liveness >= need && mark.get(id as usize) == Some(&b'0'))
        .map(|&(id, _)| id)
        .collect()
}

async fn 领取(d: &DaLeDou, id: u8) {
    // 领取
    let cmd = format!("cmd=liveness&giftbagid={id}&action=1");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 领取帮派总活跃礼包(d: &DaLeDou) {
    // 领取
    let data: Response = match d.get("cmd=factionOp&subtype=4").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── parse_liveness ───

    #[test]
    fn test_parse_liveness_with_faction() {
        assert_eq!(parse_liveness("80/40"), Some(80));
        assert_eq!(parse_liveness("115/100"), Some(115));
    }

    #[test]
    fn test_parse_liveness_without_faction() {
        assert_eq!(parse_liveness("80"), Some(80));
        assert_eq!(parse_liveness("20"), Some(20));
    }

    #[test]
    fn test_parse_liveness_invalid() {
        assert_eq!(parse_liveness(""), None);
        assert_eq!(parse_liveness("abc"), None);
    }

    // ─── find_claimable_gifts ───

    /// 32 位全 '0'（全部未领取），活跃度 115
    const ALL_ZERO: &str = "00000000000000000000000000000000";

    /// 32 位全 '1'（全部已领取）
    const ALL_ONE: &str = "11111111111111111111111111111111";

    #[test]
    fn test_all_claimable() {
        assert_eq!(find_claimable_gifts(115, ALL_ZERO), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_partial_liveness() {
        assert_eq!(find_claimable_gifts(50, ALL_ZERO), vec![1, 2]);
    }

    #[test]
    fn test_liveness_below_minimum() {
        assert!(find_claimable_gifts(10, ALL_ZERO).is_empty());
    }

    #[test]
    fn test_all_already_claimed() {
        assert!(find_claimable_gifts(115, ALL_ONE).is_empty());
    }

    #[test]
    fn test_markseq_too_short() {
        assert!(find_claimable_gifts(115, "0").is_empty());
        assert!(find_claimable_gifts(115, "0000").is_empty());
    }

    #[test]
    fn test_markseq_longer_than_32() {
        assert!(find_claimable_gifts(115, "000000000000000000000000000000000").is_empty());
    }

    #[test]
    fn test_mixed_state() {
        // 仅 id=2 未领取，活跃度达标
        let markseq = "11011111111111111111111111111111";
        assert_eq!(find_claimable_gifts(115, markseq), vec![2]);
    }

    // ─── remaining_calls ───

    #[test]
    fn test_remaining_calls() {
        assert_eq!(remaining_calls(0), 3);
        assert_eq!(remaining_calls(1), 2);
        assert_eq!(remaining_calls(2), 1);
        assert_eq!(remaining_calls(3), 0);
    }

    #[test]
    fn test_remaining_calls_abnormal() {
        // 接口异常返回超过上限时防御下溢
        assert_eq!(remaining_calls(4), 0);
        assert_eq!(remaining_calls(u8::MAX), 0);
    }
}
