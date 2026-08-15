//! 踢馆
//!
//! 试炼、挑战、报名、周六领奖和领取排行奖励

use std::time::Duration;

use chrono::{Datelike, Local};
use serde::Deserialize;
use tokio::time;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "踢馆";

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
        #[serde(default, rename = "CanSign")]
        can_sign: String, // 是否可以报名
        #[serde(default, rename = "isFightTime")]
        is_fight_time: String, // 是否战斗时间
        #[serde(default, rename = "isAwardTime")]
        is_award_time: String, // 是否领奖时间
        #[serde(default, rename = "figntNpcTimes")]
        fight_npc_times: String, // 已使用试炼次数
        #[serde(default, rename = "MaxFightNpcTimes")]
        max_fight_npc_times: String, // 试炼最大次数上限
        #[serde(default, rename = "lifeNum")]
        life_num: String, // 挑战复活次数
    }

    // 踢馆
    let data: Query = match d.get("cmd=fac_challenge").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    if data.result != "0" {
        d.log(TASK, &data.msg);
        return;
    };

    if data.is_fight_time == "1" {
        if data.fight_npc_times != data.max_fight_npc_times {
            试炼(d).await;
        }

        if data.life_num != "0" {
            挑战(d).await;
        }

        return;
    };

    if data.can_sign == "1" {
        报名(d).await;
    };

    // 不是星期六
    if Local::now().weekday() != chrono::Weekday::Sat {
        return;
    }

    if data.is_award_time == "1" {
        领奖(d).await;
        time::sleep(Duration::from_millis(200)).await;
        领取排行奖励(d).await;
    };
}

async fn 试炼(d: &DaLeDou) {
    for _ in 0..5 {
        // 试炼
        let data: Response = match d.get("cmd=fac_challenge&subtype=2").await {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("{e}"));
                return;
            }
        };

        d.log(TASK, &data.msg);
        time::sleep(Duration::from_millis(200)).await;
        if data.result != "0" {
            return;
        }
    }
}

async fn 挑战(d: &DaLeDou) {
    for _ in 0..30 {
        // 挑战
        let data: Response = match d.get("cmd=fac_challenge&subtype=3").await {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("{e}"));
                return;
            }
        };

        d.log(TASK, &data.msg);
        time::sleep(Duration::from_millis(200)).await;
        if data.result != "0" {
            return;
        }
    }
}

async fn 报名(d: &DaLeDou) {
    // 报名
    let data: Response = match d.get("cmd=fac_challenge&subtype=1").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 领奖(d: &DaLeDou) {
    // 领奖
    let data: Response = match d.get("cmd=fac_challenge&subtype=9").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 领取排行奖励(d: &DaLeDou) {
    // 领取排行奖励
    let data: Response = match d.get("cmd=fac_challenge&subtype=10").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
