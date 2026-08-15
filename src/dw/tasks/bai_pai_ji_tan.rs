//! 帮派祭坛
//!
//! 转动轮盘
//!
//! 掠夺|偷取帮派优先级：复仇列表、随机分配、宣战帮派

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "帮派祭坛";

#[derive(Deserialize)]
struct Response {
    result: String,
    msg: String,
}

pub async fn run(d: &DaLeDou) {
    for _ in 0..30 {
        let Some(data) = query(d).await else {
            return;
        };

        if data.last_reward_level != "0" {
            领取(d).await;
            continue;
        }

        match data.current_action_id.as_str() {
            "1003" if !掠夺帮派(d).await => return,
            "1004" if !偷取帮派(d).await => return,
            _ => {
                if !转动轮盘(d).await {
                    return;
                }
            }
        }
    }
}

#[derive(Deserialize)]
struct Query {
    result: String,
    msg: String,
    #[serde(default)]
    left_free_wheel_times: String, // 剩余次数
    #[serde(default)]
    current_action_id: String,
    #[serde(default)]
    last_reward_level: String,
}

async fn query(d: &DaLeDou) -> Option<Query> {
    let data: Query = match d.get("cmd=altar").await {
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

    // 没有次数并且没有正在进行的任务
    if data.left_free_wheel_times == "0" && data.current_action_id == "0" {
        return None;
    }

    Some(data)
}

async fn 领取(d: &DaLeDou) {
    // 领取
    let data: Response = match d.get("cmd=altar&op=drawreward").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 转动轮盘(d: &DaLeDou) -> bool {
    // 转动轮盘
    let data: Response = match d.get("cmd=altar&op=spinwheel").await {
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

    if !data.msg.is_empty() {
        d.log(TASK, &data.msg);
    }

    true
}

async fn 掠夺帮派(d: &DaLeDou) -> bool {
    let Some(data) = 帮派(d).await else {
        return false;
    };

    for item in &data.revenge_targets {
        if 掠夺(d, &item.id).await {
            return true;
        }
    }

    if !data.random_faction.id.is_empty() && 掠夺(d, &data.random_faction.id).await {
        return true;
    }

    for item in &data.enemies {
        if 掠夺(d, &item.id).await {
            return true;
        }
    }

    false
}

async fn 掠夺(d: &DaLeDou, id: &str) -> bool {
    // 掠夺
    let cmd = format!("cmd=altar&op=rob&id={id}");
    let data: Response = match d.get(&cmd).await {
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

    true
}

async fn 偷取帮派(d: &DaLeDou) -> bool {
    let Some(data) = 帮派(d).await else {
        return false;
    };

    for item in &data.revenge_targets {
        if 偷取(d, &item.id).await {
            return true;
        }
    }

    if !data.random_faction.id.is_empty() && 偷取(d, &data.random_faction.id).await {
        return true;
    }

    for item in &data.enemies {
        if 偷取(d, &item.id).await {
            return true;
        }
    }

    false
}

async fn 偷取(d: &DaLeDou, id: &str) -> bool {
    // 偷取
    let cmd = format!("cmd=altar&op=steal&id={id}");
    let data: Response = match d.get(&cmd).await {
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

    true
}

#[derive(Deserialize)]
struct BangPai {
    result: String,
    msg: String,
    random_faction: Id,       // 随机分配
    enemies: Vec<Id>,         // 宣战帮派
    revenge_targets: Vec<Id>, // 复仇列表
}

#[derive(Deserialize)]
struct Id {
    #[serde(default)]
    id: String,
}

async fn 帮派(d: &DaLeDou) -> Option<BangPai> {
    // 掠夺|选择帮派列表
    let data: BangPai = match d.get("cmd=altar&op=showspecialtargets").await {
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
