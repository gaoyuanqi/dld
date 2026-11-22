//! 帮派远征军
//!
//! 攻击、领取岛屿和节点奖励
//!
//! 按照战力从低到高攻击
//!
//! 战败时才领取奖励，这样可以免费复活一次

use std::cmp::Ordering;

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

        for info in &data.fight_info.island_info {
            // 节点未激活
            if info.point_status == "0" {
                return;
            }

            // 节点已通关
            if info.point_status == "2" {
                continue;
            }

            fight_node(d, info).await;
            break;
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

async fn fight_node(d: &DaLeDou, info: &IslandInfo) {
    let Some(data) = 参战(d, &info.point_id, "1").await else {
        return;
    };

    let total_pages: u8 = match data.total_pages.parse() {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("解析 total_pages 字段失败：{e}"));
            return;
        }
    };

    // 收集所有驻守玩家（第一页已包含）
    let mut all_defenders = data.settle_usr_info;
    for page in 2..=total_pages {
        let Some(data) = 参战(d, &info.point_id, &page.to_string()).await else {
            return;
        };
        all_defenders.extend(data.settle_usr_info);
    }

    // 过滤已战败，并按战力（浮点数）从低到高排序
    let mut defenders: Vec<SettleUsrInfo> = all_defenders
        .into_iter()
        .filter(|p| p.is_dead == "0")
        .collect();
    defenders.sort_by(|a, b| {
        let a_val = a.fight_capacity.parse::<f64>().unwrap_or(f64::INFINITY);
        let b_val = b.fight_capacity.parse::<f64>().unwrap_or(f64::INFINITY);
        a_val.partial_cmp(&b_val).unwrap_or(Ordering::Equal)
    });

    for defender in defenders {
        if !攻击(d, &info.point_id, &defender.opp_uin).await {
            return;
        }
    }
}

#[derive(Deserialize)]
struct Point {
    result: String,
    msg: String,
    #[serde(rename = "totalPages")]
    total_pages: String, // 总页数
    #[serde(rename = "settleUsrInfo")]
    settle_usr_info: Vec<SettleUsrInfo>, // 驻守玩家信息
}

#[derive(Deserialize)]
struct SettleUsrInfo {
    #[serde(rename = "oppUin")]
    opp_uin: String, // QQ
    #[serde(rename = "isDead")]
    is_dead: String, // 驻守玩家是否阵亡
    #[serde(rename = "fightCapacity")]
    fight_capacity: String, // 战力
}

async fn 参战(d: &DaLeDou, point_id: &str, page: &str) -> Option<Point> {
    // 参战
    let cmd = format!("cmd=factionarmy&op=viewpoint&point_id={point_id}&page={page}");
    let data: Point = match d.get(&cmd).await {
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
        // {"result":"-1","msg":"您的血量不足，请重生后在进行战斗","replays":""}
        // {"result":"-1","msg":"该敌人似乎逃跑啦~ 更换一名敌人进行战斗吧！","replays":""}
        return data.msg.starts_with("该敌人似乎逃跑啦");
    }

    data.msg.starts_with("勇士，恭喜您战胜")
}
