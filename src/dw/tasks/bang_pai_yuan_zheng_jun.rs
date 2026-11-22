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
    msg: String,
}

pub async fn run(d: &DaLeDou) {
    loop {
        let Some(data) = query(d, "-1").await else {
            return;
        };

        // 已推进到最后一个节点：领奖后结束
        if data.fight_info.process == data.fight_info.total_point_num {
            // 岛屿奖励可领或者已领取表示已通关所有节点
            if data.fight_info.island_award_status == "1"
                || data.fight_info.island_award_status == "2"
            {
                领取奖励(d).await;
                return;
            }
        }

        // 战败：领奖（可能免费复活）后若仍未复活，结束任务
        if data.fight_info.revive == "1" {
            // 如果成功领取上一个节点奖励则会免费复活
            领取奖励(d).await;

            let Some(data) = query(d, "-1").await else {
                return;
            };
            if data.fight_info.revive == "1" {
                return;
            }
        }

        // 找进行中的节点，没有则结束任务
        let Some(info) = data
            .fight_info
            .island_info
            .iter()
            .find(|info| info.point_status == "1")
        else {
            return;
        };

        // 拉取驻守玩家并按战力从低到高排序
        let Some(players) = 拉取驻守玩家(d, &info.point_id).await else {
            return;
        };
        let defenders = 排序驻守玩家(players);

        // 逐个攻击
        for defender in defenders {
            let Some(msg) = 攻击(d, &info.point_id, &defender.opp_uin).await else {
                return;
            };

            // {"result":"-1","msg":"该敌人似乎逃跑啦~ 更换一名敌人进行战斗吧！","replays":""}
            if msg.starts_with("勇士，恭喜您战胜") || msg.starts_with("该敌人似乎逃跑啦")
            {
                continue;
            }

            // {"result":"-1","msg":"加入帮派第一周不能参与帮派远征军","replays":""}
            if msg.starts_with("加入帮派第一周") {
                return;
            }

            // 勇士，您未能战胜敌军
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

/// 领取所有可领的岛屿奖励和节点奖励
async fn 领取奖励(d: &DaLeDou) {
    for island_id in 0..5 {
        let Some(data) = query(d, &format!("{island_id}")).await else {
            return;
        };

        // 可领取
        if data.fight_info.island_award_status == "1" {
            领取岛屿奖励(d, &format!("{island_id}")).await;
        }

        for item in &data.fight_info.island_info {
            // 还没有推进到该节点或者正在进行
            if item.point_status != "2" {
                return;
            }

            // 可领取
            if item.award_status == "1" {
                领取节点奖励(d, &item.point_id).await;
            }
        }
    }
}

async fn 领取岛屿奖励(d: &DaLeDou, island_id: &str) {
    // 岛屿奖励
    let cmd = format!("cmd=factionarmy&op=getIslandAward&island_id={island_id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 领取节点奖励(d: &DaLeDou, point_id: &str) {
    // 节点奖励
    let cmd = format!("cmd=factionarmy&op=getPointAward&point_id={point_id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

/// 拉取节点全部页的驻守玩家
async fn 拉取驻守玩家(d: &DaLeDou, point_id: &str) -> Option<Vec<SettleUsrInfo>> {
    let data = 参战(d, point_id, "1").await?;

    let total_pages: u8 = match data.total_pages.parse() {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("解析 total_pages 字段失败：{e}"));
            return None;
        }
    };

    // 收集所有驻守玩家（第一页已包含）
    let mut all_defenders = data.settle_usr_info;
    for page in 2..=total_pages {
        let data = 参战(d, point_id, &page.to_string()).await?;
        all_defenders.extend(data.settle_usr_info);
    }

    Some(all_defenders)
}

/// 过滤已阵亡的驻守玩家，并按战力从低到高排序
fn 排序驻守玩家(players: Vec<SettleUsrInfo>) -> Vec<SettleUsrInfo> {
    let mut defenders: Vec<SettleUsrInfo> =
        players.into_iter().filter(|p| p.is_dead == "0").collect();
    defenders.sort_by(|a, b| {
        let a_val = a.fight_capacity.parse::<f64>().unwrap_or(f64::INFINITY);
        let b_val = b.fight_capacity.parse::<f64>().unwrap_or(f64::INFINITY);
        a_val.partial_cmp(&b_val).unwrap_or(Ordering::Equal)
    });
    defenders
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

/// 攻击，返回接口 msg，请求失败返回 None
async fn 攻击(d: &DaLeDou, point_id: &str, opp_uin: &str) -> Option<String> {
    // 攻击
    let cmd = format!("cmd=factionarmy&op=fightWithUsr&point_id={point_id}&opp_uin={opp_uin}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return None;
        }
    };

    d.log(TASK, &data.msg);
    Some(data.msg)
}

#[cfg(test)]
mod tests {
    use super::*;

    // 构造驻守玩家
    fn 玩家(opp_uin: &str, is_dead: &str, fight_capacity: &str) -> SettleUsrInfo {
        SettleUsrInfo {
            opp_uin: opp_uin.into(),
            is_dead: is_dead.into(),
            fight_capacity: fight_capacity.into(),
        }
    }

    // 提取排序后的 QQ 列表
    fn qqs(players: &[SettleUsrInfo]) -> Vec<&str> {
        players.iter().map(|p| p.opp_uin.as_str()).collect()
    }

    // 过滤已阵亡的驻守玩家
    #[test]
    fn test_排序驻守玩家_过滤阵亡() {
        let players = vec![
            玩家("1", "1", "100"),
            玩家("2", "0", "200"),
            玩家("3", "0", "300"),
        ];
        assert_eq!(qqs(&排序驻守玩家(players)), ["2", "3"]);
    }

    // 按战力从低到高排序
    #[test]
    fn test_排序驻守玩家_战力升序() {
        let players = vec![
            玩家("1", "0", "300"),
            玩家("2", "0", "100"),
            玩家("3", "0", "200"),
        ];
        assert_eq!(qqs(&排序驻守玩家(players)), ["2", "3", "1"]);
    }

    // 战力解析失败排最后
    #[test]
    fn test_排序驻守玩家_战力解析失败排最后() {
        let players = vec![
            玩家("1", "0", "无效"),
            玩家("2", "0", "100"),
            玩家("3", "0", "50"),
        ];
        assert_eq!(qqs(&排序驻守玩家(players)), ["3", "2", "1"]);
    }
}
