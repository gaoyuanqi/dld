//! 华山论剑
//!
//! 每月1~25号挑战，其它时间领取排位奖励

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "华山论剑";

#[derive(Deserialize)]
struct Response {
    msg: String,
}

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default)]
        next_season_time: String, // 下赛季开始时间
        #[serde(default)]
        left_free_times: String, // 免费挑战次数
        #[serde(default)]
        can_draw_rank_reward: String, // 是否可领取排名奖励
    }

    let data: Query = match d.get("cmd=knightarena").await {
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

    // 休赛期
    if data.can_draw_rank_reward == "1" {
        领取段位奖励(d).await;
        return;
    }

    // 赛季进行中
    if data.next_season_time != "-1" {
        return;
    }

    let free_times: u8 = match data.left_free_times.parse() {
        Ok(n) => n,
        Err(e) => {
            d.log(TASK, &format!("解析 left_free_times 失败：{e}"));
            return;
        }
    };

    开始挑战(d, free_times).await;
}

async fn 开始挑战(d: &DaLeDou, free_times: u8) {
    #[derive(Deserialize)]
    struct Challenge {
        result: String,
        msg: String,
        #[serde(default)]
        repid: String,
    }

    for _ in 0..free_times {
        // 开始挑战
        let data: Challenge = match d.get("cmd=knightarena&op=challenge").await {
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

        if !data.repid.is_empty() {
            挑战记录(d, &data.repid).await;
        }
    }
}

async fn 挑战记录(d: &DaLeDou, repid: &str) {
    #[derive(Deserialize)]
    struct QueryRecord {
        result: String,
        msg: String,
        records: Vec<Records>,
    }

    #[derive(Deserialize)]
    struct Records {
        repid: String,
        desc: String,
    }

    // 挑战记录
    let data: QueryRecord = match d.get("cmd=knightarena&op=queryrecord").await {
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

    for item in &data.records {
        if item.repid == repid {
            d.log(TASK, &item.desc);
            return;
        }
    }
}

async fn 领取段位奖励(d: &DaLeDou) {
    // 领取段位奖励
    let data: Response = match d.get("cmd=knightarena&op=rankingreward").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
