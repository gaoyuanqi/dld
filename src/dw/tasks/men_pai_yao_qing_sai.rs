//! 门派邀请赛
//!
//! 报名、领取段位奖励、免费挑战

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
            d.log(TASK, &format!("解析 left_fight_times 失败: {e}"));
            return;
        }
    };

    // 前5次免费，算剩余免费次数
    let free_fight_times = left_fight_times.saturating_sub(5);
    挑战(d, free_fight_times).await;
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
    let cmd = "cmd=secttournament&op=getrankandrankingreward";
    let data: Response = match d.get(cmd).await {
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
            d.log(TASK, &data.msg);
            return;
        }
    }
}
