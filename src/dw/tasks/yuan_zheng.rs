//! 帮派远征军
//!
//! 攻击、领取岛屿和节点奖励
//!
//! 战败时才领取奖励，这样可以免费复活一次

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "帮派远征军";

#[derive(Deserialize)]
struct Response {
    result: String,
    msg: String,
}

pub async fn run(d: &DaLeDou) {
    loop {
        let Some(data) = query(d, "-1").await else {
            return;
        };

        // 已推进到最后一个节点
        if data.fight_info.process == data.fight_info.total_point_num {
            // 岛屿奖励可领或者已领取表示已通关所有节点
            if data.fight_info.island_award_status == "1"
                || data.fight_info.island_award_status == "2"
            {
                领取奖励(d).await;
                return;
            }
        }

        // 你已战败并且没有免费复活
        if data.fight_info.revive == "1" && !领取奖励(d).await {
            return;
        }

        for item in &data.fight_info.island_info {
            // 未激活
            if item.point_status == "0" {
                break;
            }

            // 已通关
            if item.point_status == "2" {
                continue;
            }

            let Some(point_data) = 参战(d, &item.point_id).await else {
                break;
            };

            for p in point_data.settle_usr_info.iter().rev() {
                // 驻守玩家已战败
                if p.is_dead == "1" {
                    continue;
                }

                if !攻击(d, &item.point_id, &p.opp_uin).await && !领取奖励(d).await {
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
    #[serde(default, rename = "fightInfo")]
    fight_info: FightInfo, // 战斗信息
}

#[derive(Default, Deserialize)]
struct FightInfo {
    revive: String, // 是否需复活
    #[serde(rename = "islandAwardStatus")]
    island_award_status: String, // 岛屿奖励状态
    process: String, // 当前节点
    #[serde(rename = "totalPointNum")]
    total_point_num: String, // 总节点数量
    #[serde(rename = "islandInfo")]
    island_info: Vec<IslandInfo>, // 岛屿信息
}

#[derive(Deserialize)]
struct IslandInfo {
    #[serde(rename = "pointId")]
    point_id: String, // 节点id
    #[serde(rename = "pointStatus")]
    point_status: String, // 节点状态
    #[serde(rename = "awardStatus")]
    award_status: String, // 节点奖励状态
}

async fn query(d: &DaLeDou, island_id: &str) -> Option<Query> {
    let cmd = format!("cmd=factionarmy&op=viewIndex&island_id={island_id}");
    let data: Query = match d.get(&cmd).await {
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

async fn 领取奖励(d: &DaLeDou) -> bool {
    'outer: for island_id in 0..5 {
        let Some(data) = query(d, &format!("{island_id}")).await else {
            return false;
        };

        if data.fight_info.island_award_status == "1" && !岛屿奖励(d, &format!("{island_id}")).await
        {
            return false;
        }

        for item in &data.fight_info.island_info {
            // 还没有推进到该节点或者正在进行
            if item.point_status != "2" {
                break 'outer;
            }

            // 可领取奖励
            if item.award_status == "1" && !节点奖励(d, &item.point_id).await {
                return false;
            }
        }
    }

    let Some(data) = query(d, "-1").await else {
        return false;
    };

    // 你复活了
    data.fight_info.revive == "0"
}

async fn 岛屿奖励(d: &DaLeDou, island_id: &str) -> bool {
    // 岛屿奖励
    let cmd = format!("cmd=factionarmy&op=getIslandAward&island_id={island_id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return false;
        }
    };

    d.log(TASK, &data.msg);
    data.result == "0"
}

async fn 节点奖励(d: &DaLeDou, point_id: &str) -> bool {
    // 节点奖励
    let cmd = format!("cmd=factionarmy&op=getPointAward&point_id={point_id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return false;
        }
    };

    d.log(TASK, &data.msg);
    data.result == "0"
}

#[derive(Deserialize)]
struct Point {
    result: String,
    msg: String,
    #[serde(rename = "settleUsrInfo")]
    settle_usr_info: Vec<SettleUsrInfo>, // 驻守玩家信息
}

#[derive(Deserialize)]
struct SettleUsrInfo {
    #[serde(rename = "oppUin")]
    opp_uin: String, // QQ
    #[serde(rename = "isDead")]
    is_dead: String, // 是否战败
}

async fn 参战(d: &DaLeDou, point_id: &str) -> Option<Point> {
    // 参战
    let cmd = format!("cmd=factionarmy&op=viewpoint&point_id={point_id}");
    let data: Point = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return None;
        }
    };

    // 点未解锁：当前岛屿已通关
    if data.result == "-1" {
        return None;
    }

    if data.result != "0" {
        d.log(TASK, &data.msg);
        return None;
    }

    Some(data)
}

async fn 攻击(d: &DaLeDou, point_id: &str, opp_uin: &str) -> bool {
    // 攻击
    let cmd = format!("cmd=factionarmy&op=fightWithUsr&point_id={point_id}&opp_uin={opp_uin}");
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

    data.msg.starts_with("勇士，恭喜您战胜")
}
