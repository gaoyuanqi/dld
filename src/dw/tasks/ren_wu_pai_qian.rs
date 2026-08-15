//! 任务派遣中心
//!
//! 优先尝试 S、B 级任务，全部未成功时若有免费刷新则刷新重试，
//! 无免费刷新时降级接受 A 级

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "任务派遣中心";

pub async fn run(d: &DaLeDou) {
    let Some(data) = query(d).await else {
        return;
    };

    for item in &data.accepted_mission_info {
        // 已完成
        if item.remain_time == "0" {
            领取奖励(d, &item.mission_id).await;
        }
    }

    for _ in 0..4 {
        let Some(data) = query(d).await else {
            return;
        };

        // 今日接受任务数已到上限
        if data.accepted_mission_num == data.day_mission_num {
            return;
        }

        // 最多接受3个任务
        if data.accepted_mission_info.len() >= 3 {
            return;
        }

        // 优先尝试 S、B 级任务，遍历完当前全部待接任务
        for item in &data.standy_mission_info {
            if item.accepted == "1" || !(item.t == "1" || item.t == "3") {
                continue;
            }

            if 快速委派(d, &item.mission_name, &item.mission_id).await
                && !开始任务(d, &item.mission_name, &item.mission_id).await
            {
                return;
            }
        }

        let Some(data) = query(d).await else {
            return;
        };

        // 今日接受任务数已到上限
        if data.accepted_mission_num == data.day_mission_num {
            return;
        }

        // 最多接受3个任务
        if data.accepted_mission_info.len() >= 3 {
            return;
        }

        // 有免费刷新则刷新
        if data.refresh_doudou_num == "0" {
            刷新任务(d).await;
            continue;
        }

        // 无免费刷新，降级接受 A 级
        for item in &data.standy_mission_info {
            if item.accepted == "1" || item.t != "2" {
                continue;
            }

            if 快速委派(d, &item.mission_name, &item.mission_id).await
                && !开始任务(d, &item.mission_name, &item.mission_id).await
            {
                return;
            }
        }

        return;
    }
}

#[derive(Deserialize)]
struct Query {
    result: String,
    msg: String,
    #[serde(rename = "acceptedMissionNum")]
    accepted_mission_num: String, // 今日已接受任务数量
    #[serde(rename = "dayMissionNum")]
    day_mission_num: String, // 每日可接受任务上限
    #[serde(rename = "refreshDoudouNum")]
    refresh_doudou_num: String, // 刷新消耗斗豆数量
    #[serde(rename = "acceptedMissionInfo")]
    accepted_mission_info: Vec<AcceptedMissionInfo>, // 进行中的任务列表
    #[serde(rename = "standyMissionInfo")]
    standy_mission_info: Vec<StandyMissionInfo>, // 可接受任务列表
}

#[derive(Deserialize)]
struct AcceptedMissionInfo {
    #[serde(rename = "missionId")]
    mission_id: String, // 任务id
    #[serde(rename = "remainTime")]
    remain_time: String, // 剩余时间
}

#[derive(Deserialize)]
struct StandyMissionInfo {
    accepted: String, // 是否已接受
    #[serde(rename = "missionId")]
    mission_id: String, // 任务id
    #[serde(rename = "type")]
    t: String, // 任务类型 S1 A2 B3
    #[serde(rename = "missionName")]
    mission_name: String, // 任务名称
}

async fn query(d: &DaLeDou) -> Option<Query> {
    // 任务派遣中心
    let data: Query = match d.get("cmd=assignment&op=0").await {
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
    result: String,
    msg: String,
}

async fn 领取奖励(d: &DaLeDou, id: &str) {
    // 领取奖励
    let cmd = format!("cmd=assignment&op=8&mission_id={id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 刷新任务(d: &DaLeDou) {
    // 刷新任务
    let data: Response = match d.get("cmd=assignment&op=1").await {
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

    d.log(TASK, "刷新成功");
}

async fn 快速委派(d: &DaLeDou, name: &str, id: &str) -> bool {
    // 快速委派
    let cmd = format!("cmd=assignment&op=6&mission_id={id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return false;
        }
    };

    d.log(TASK, &format!("{} => {}", name, data.msg));
    data.msg == "设置佣兵成功"
}

async fn 开始任务(d: &DaLeDou, name: &str, id: &str) -> bool {
    // 开始任务
    let cmd = format!("cmd=assignment&op=7&mission_id={id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return false;
        }
    };

    d.log(TASK, &format!("{} => {}", name, data.msg));
    data.msg == "任务开始执行"
}
