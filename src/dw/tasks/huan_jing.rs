//! 幻境
//!
//! 自动选择最高场景乐斗、领取奖励

use std::cmp;

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "幻境";

#[derive(Deserialize)]
struct Response {
    result: String,
    msg: String,
}

#[derive(Deserialize)]
struct Query {
    result: String,
    msg: String,
    #[serde(default)]
    cur_stage: String, // 当前阶段
    #[serde(default)]
    max_stage: String, // 最大阶段id
    #[serde(default)]
    stage_num: String, // 阶段id
    #[serde(default)]
    challenge_times: String, // 挑战次数
    #[serde(default)]
    can_return: String, // 是否可退出
    #[serde(default)]
    box_mark: Vec<BoxMark>, // 宝箱标记
}

#[derive(Deserialize)]
struct BoxMark {
    id: String,
    status: String, //是否可领取宝箱
}

pub async fn run(d: &DaLeDou) {
    let Some(data) = query(d).await else {
        return;
    };

    // 正在场景中
    if data.cur_stage != "0" {
        // 不可退出，说明当前场景还未挑战
        if data.can_return == "0" {
            挑战(d).await;
        }
        退出幻境(d).await;
    }

    // 没有挑战次数
    if data.challenge_times == "0/1" {
        return;
    }

    let max_stage: u32 = match data.max_stage.parse() {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("解析 max_stage 字段失败：{e}"));
            return;
        }
    };

    let stage_num: u32 = match data.stage_num.parse() {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("解析 stage_num 字段失败：{e}"));
            return;
        }
    };

    let target_stage = cmp::min(stage_num, max_stage);
    let target_id = target_stage.to_string();
    if !进入(d, &target_id).await {
        return;
    }

    挑战(d).await;
    退出幻境(d).await;
}

async fn query(d: &DaLeDou) -> Option<Query> {
    let data: Query = match d.get("cmd=misty").await {
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

async fn 进入(d: &DaLeDou, id: &str) -> bool {
    // 进入
    let cmd = format!("cmd=misty&op=start&stage_id={id}");
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

async fn 挑战(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Response {
        result: String,
        msg: String,
        reward_points: String, // 奖励积分
    }

    for _ in 0..5 {
        // 挑战
        let data: Response = match d.get("cmd=misty&op=fight").await {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("{e}"));
                return;
            }
        };

        d.log(TASK, &data.msg);

        领取(d).await;

        if data.result != "0" {
            return;
        }

        // 挑战没有获得积分，战败
        if data.reward_points == "0" {
            return;
        }
    }
}

async fn 领取(d: &DaLeDou) {
    let Some(data) = query(d).await else {
        return;
    };

    for item in &data.box_mark {
        // 已领取或者未激活
        if item.status != "0" {
            continue;
        }

        // 领取
        let cmd = format!("cmd=misty&op=reward&box_id={}", item.id);
        let data: Response = match d.get(&cmd).await {
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

async fn 退出幻境(d: &DaLeDou) {
    // 退出
    let data: Response = match d.get("cmd=misty&op=return").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    if data.result != "0" {
        d.log(TASK, &data.msg);
    }
}
