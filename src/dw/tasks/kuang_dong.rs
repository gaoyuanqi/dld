//! 矿洞
//!
//! 每天挑战、领取排名奖励、开启副本

use std::time::Duration;

use serde::Deserialize;
use tokio::time;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "矿洞";

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
        privilege_granted: String,
        #[serde(default)]
        fight_times: String,
        #[serde(default)]
        reward_message: String,
        #[serde(default)]
        current_dungeon_floor: String,
    }

    for _ in 0..10 {
        // 矿洞
        let data: Query = match d.get("cmd=factionmine").await {
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

        if !data.reward_message.is_empty() {
            领取奖励(d).await;
            continue;
        }

        if data.current_dungeon_floor == "0" {
            // 无权限开启
            if data.privilege_granted == "0" {
                return;
            }
            if !开启副本(d).await {
                return;
            }
            continue;
        }

        // 已经挑战3次最大次数
        if data.fight_times == "3" {
            return;
        }

        if !挑战(d).await {
            return;
        }
    }
}

async fn 领取奖励(d: &DaLeDou) {
    // 领取奖励
    let data: Response = match d.get("cmd=factionmine&op=reward").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
    time::sleep(Duration::from_secs(2)).await;
}

async fn 开启副本(d: &DaLeDou) -> bool {
    let cfg = d.config();
    let cmd = format!(
        "cmd=factionmine&op=start&floor={}&mode={}",
        cfg.矿洞.开启副本.层数.api_value(),
        cfg.矿洞.开启副本.模式.api_value()
    );
    // 开启副本
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return false;
        }
    };

    d.log(TASK, &data.msg);
    time::sleep(Duration::from_secs(2)).await;
    data.result == "0"
}

async fn 挑战(d: &DaLeDou) -> bool {
    #[derive(Deserialize)]
    struct Fight {
        result: String,
        msg: String,
        repid: String,
    }

    // 挑战
    let data: Fight = match d.get("cmd=factionmine&op=fight").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return false;
        }
    };

    if data.result != "0" {
        d.log(TASK, &data.msg);
        return false;
    }

    挑战记录(d, &data.repid).await;
    time::sleep(Duration::from_secs(2)).await;
    true
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
    let data: QueryRecord = match d.get("cmd=factionmine&op=record").await {
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
