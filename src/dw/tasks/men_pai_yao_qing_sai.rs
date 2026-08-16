//! 门派邀请赛
//!
//! 报名、领取段位奖励、免费挑战、兑换
//!
//! 如果免费次数还剩5次则兑换

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "门派邀请赛";

#[derive(Deserialize)]
struct Response {
    result: String,
    msg: String,
}

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default)]
        in_group: String, // 是否已组队
        #[serde(default)]
        in_sign_up_time: String, // 是否报名期
        #[serde(default)]
        in_fight_time: String, // 是否战斗期
        #[serde(default)]
        left_fight_times: String, // 剩余挑战次数
    }

    let data: Query = match d.get("cmd=secttournament").await {
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

    // 报名期
    if data.in_sign_up_time == "1" {
        // 还没有组队
        if data.in_group == "0" {
            报名(d).await;
            领取段位奖励(d).await;
        }
        return;
    }

    // 非战斗期
    if data.in_fight_time != "1" {
        return;
    }

    let left_fight_times: u8 = match data.left_fight_times.parse() {
        Ok(n) => n,
        Err(e) => {
            d.log(TASK, &format!("解析 left_fight_times 字段失败：{e}"));
            return;
        }
    };

    // 前5次免费，算剩余免费次数
    let free_fight_times = left_fight_times.saturating_sub(5);
    挑战(d, free_fight_times).await;
    if free_fight_times == 5 {
        商店(d).await;
    }
}

async fn 报名(d: &DaLeDou) {
    // 报名
    let data: Response = match d.get("cmd=secttournament&op=signup").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 领取段位奖励(d: &DaLeDou) {
    // 领取段位奖励
    let data: Response = match d.get("cmd=secttournament&op=getrankandrankingreward").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 挑战(d: &DaLeDou, n: u8) {
    for _ in 0..n {
        // 挑战
        let data: Response = match d.get("cmd=secttournament&op=fight").await {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("{e}"));
                return;
            }
        };

        d.log(TASK, &data.msg);
        if data.result != "0" {
            return;
        }
    }
}

async fn 商店(d: &DaLeDou) {
    let exchange = &d.config().门派邀请赛.兑换;

    // 所有物品配置数量都为 0，无需兑换
    if exchange.炼气石 == 0 && exchange.门派强化书 == 0 {
        return;
    }

    #[derive(Deserialize)]
    struct Exchange {
        result: String,
        msg: String,
        values: Values,
        items: Vec<Items>,
    }

    #[derive(Deserialize)]
    struct Values {
        #[serde(rename = "11")]
        score: String, // 商店积分
    }

    #[derive(Deserialize)]
    struct Items {
        #[serde(rename = "type")]
        t: String,
        name: String,
        needs_num: String, // 消耗积分
    }

    // 商店
    let data: Exchange = match d.get("cmd=exchange&subtype=16").await {
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

    let mut score: u32 = match data.values.score.parse() {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("解析 score 字段失败：{e}"));
            return;
        }
    };

    // 商店积分低于单价
    if score < 40 {
        return;
    }

    for item in &data.items {
        let want = match item.name.as_str() {
            "炼气石" => exchange.炼气石,
            "门派强化书" => exchange.门派强化书,
            _ => continue,
        };
        if want == 0 {
            continue;
        }

        let needs_num: u32 = match item.needs_num.parse() {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("解析 {} needs_num 字段失败：{e}", item.name));
                continue;
            }
        };

        if needs_num == 0 {
            d.log(TASK, &format!("{} 单价为：{needs_num}", item.name));
            continue;
        }

        let max = want.min(score / needs_num);
        if max == 0 {
            continue;
        }

        let (tens, ones) = (max / 10, max % 10);
        score -= needs_num * max;

        for _ in 0..tens {
            兑换(d, &item.t, 10).await;
        }

        for _ in 0..ones {
            兑换(d, &item.t, 1).await;
        }
    }
}

async fn 兑换(d: &DaLeDou, t: &str, num: u8) {
    // 兑换
    let cmd = format!("cmd=exchange&subtype=2&type={t}&times={num}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
