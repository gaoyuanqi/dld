//! 深渊之潮
//!
//! 领取巡礼、秘境
//!
//! 秘境副本战败则退出副本然后重新挑战

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "深渊之潮";

#[derive(Deserialize)]
struct Response {
    result: String,
    msg: String,
}

pub async fn run(d: &DaLeDou) {
    帮派巡礼(d).await;
    深渊秘境(d).await;
}

async fn 帮派巡礼(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Gift {
        result: String,
        msg: String,
        #[serde(default)]
        if_can_get_gift: String, // 是否可领取巡礼
    }

    // 帮派巡礼
    let data: Gift = match d.get("cmd=abyss_tide&op=viewfactiongift").await {
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

    if data.if_can_get_gift == "1" {
        领取巡游赠礼(d).await;
    }
}

async fn 领取巡游赠礼(d: &DaLeDou) {
    // 领取巡游赠礼
    let data: Response = match d.get("cmd=abyss_tide&op=getfactiongift").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 深渊秘境(d: &DaLeDou) {
    let Some(data) = query(d).await else {
        return;
    };

    // 有副本正在进行
    if data.status == "1" {
        // 已战败或者已通关
        if data.fight_status == "2" || data.fight_status == "3" {
            if !退出副本(d).await {
                return;
            }
        } else if data.fight_status == "1" && !开始挑战(d).await {
            return;
        }
    }

    let Some(data) = query(d).await else {
        return;
    };

    if data.status != "0" {
        return;
    }

    if data.access_time == "0" && data.can_buy_times == "0" {
        return;
    }

    let is_exchange = d.config().深渊之潮.深渊秘境.兑换;
    if data.access_time == "0" && !is_exchange {
        return;
    }

    let id = d.config().深渊之潮.深渊秘境.副本.api_id();
    if data.can_buy_times != "0" && is_exchange {
        兑换次数(d).await;
    }

    let Some(data) = query(d).await else {
        return;
    };

    if data.access_time == "0" {
        return;
    }

    let access_time: u8 = match data.access_time.parse() {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("解析 access_time 字段失败：{e}"));
            return;
        }
    };

    for _ in 0..access_time {
        if !进入副本(d, id).await {
            return;
        }
        if !开始挑战(d).await {
            return;
        }
    }
}

#[derive(Deserialize)]
struct Query {
    result: String,
    msg: String,
    status: String,        // 副本状态
    access_time: String,   // 可进入副本次数
    can_buy_times: String, // 可兑换次数
    #[serde(default)]
    fight_status: String, // 战斗状态
}

async fn query(d: &DaLeDou) -> Option<Query> {
    // 深渊秘境
    let data: Query = match d.get("cmd=abyss_tide&op=viewallabyss").await {
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

async fn 进入副本(d: &DaLeDou, id: &str) -> bool {
    // 进入副本
    let cmd = format!("cmd=abyss_tide&op=enterabyss&id={id}");
    let data: Response = match d.get(&cmd).await {
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

    true
}

async fn 开始挑战(d: &DaLeDou) -> bool {
    #[derive(Deserialize)]
    struct Fight {
        result: String,
        msg: String,
        fight_status: String, // 战斗状态
    }

    for _ in 0..5 {
        // 开始挑战
        let data: Fight = match d.get("cmd=abyss_tide&op=beginfight").await {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("{e}"));
                return false;
            }
        };

        d.log(TASK, &data.msg);
        if data.result != "0" {
            return false;
        }

        // 已战败或者已通关
        if data.fight_status == "2" || data.fight_status == "3" {
            return 退出副本(d).await;
        }
    }

    true
}

async fn 退出副本(d: &DaLeDou) -> bool {
    // 退出副本
    let data: Response = match d.get("cmd=abyss_tide&op=endabyss").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return false;
        }
    };

    d.log(TASK, &data.msg);
    data.result == "0"
}

async fn 兑换次数(d: &DaLeDou) {
    for _ in 0..2 {
        // 兑换
        let data: Response = match d.get("cmd=abyss_tide&op=addaccess").await {
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
