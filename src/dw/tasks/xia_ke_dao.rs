//! 侠客岛
//!
//! 侠客行优先太玄经、玄铁令（不会被刷掉）
//!
//! 仅使用免费刷新次数

use std::collections::HashSet;

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "侠客岛";

pub async fn run(d: &DaLeDou) {
    let Some(data) = query(d).await else {
        return;
    };

    // 已完成3个任务
    if data.donemission == "3" {
        return;
    }

    for item in &data.mission {
        if item.status == "2" {
            领取(d, &item.pos).await;
        }
    }

    let mut fail: HashSet<String> = HashSet::new();

    loop {
        let Some(data) = query(d).await else {
            return;
        };

        let mut acted = false;

        for item in &data.mission {
            // 已完成或已委派
            if item.status != "0" {
                continue;
            }

            // 之前委派失败，不再尝试
            if fail.contains(&item.pos) {
                continue;
            }

            let is_target = item.reward.starts_with("太玄经") || item.reward.starts_with("玄铁令");
            if is_target || data.fresh == "0" {
                acted = true;
                if 快速委派(d, &item.name, &item.pos).await {
                    开始任务(d, &item.name, &item.pos).await;
                } else {
                    fail.insert(item.pos.clone());
                }
                break;
            }

            acted = true;
            刷新(d, &item.name, &item.pos).await;
            break;
        }

        if !acted {
            break;
        }
    }
}

#[derive(Deserialize)]
struct Query {
    result: String,
    msg: String,
    #[serde(default)]
    donemission: String, // 完成任务
    #[serde(default)]
    fresh: String, // 剩余免费刷新次数
    #[serde(default)]
    mission: Vec<Mission>,
}

#[derive(Deserialize)]
struct Mission {
    pos: String,
    name: String,
    status: String,
    reward: String,
}

async fn query(d: &DaLeDou) -> Option<Query> {
    // 侠客行
    let data: Query = match d.get("cmd=knightisland&op=viewmissionindex").await {
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

#[derive(Deserialize)]
struct Response {
    msg: String,
}

async fn 领取(d: &DaLeDou, pos: &str) {
    // 领取
    let cmd = format!("cmd=knightisland&op=getmissionreward&pos={pos}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 快速委派(d: &DaLeDou, name: &str, pos: &str) -> bool {
    // 快速委派
    let cmd = format!("cmd=knightisland&op=autoassign&pos={pos}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return false;
        }
    };

    d.log(TASK, &format!("{} => {}", name, data.msg));
    data.msg == "快速委派成功"
}

async fn 开始任务(d: &DaLeDou, name: &str, pos: &str) {
    // 开始任务
    let cmd = format!("cmd=knightisland&op=begin&pos={pos}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &format!("{} => {}", name, data.msg));
}

async fn 刷新(d: &DaLeDou, name: &str, pos: &str) {
    // 刷新
    let cmd = format!("cmd=knightisland&op=refreshmission&pos={pos}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &format!("{} => {}", name, data.msg));
}
