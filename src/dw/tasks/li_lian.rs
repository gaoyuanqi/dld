//! 历练
//!
//! 满活力值即开始乐斗，不自动使用活力药水
//!
//! 如果已通关所有场景，则按配置顺序遍历所有 BOSS，每个有剩余次数的 BOSS 乐斗一次
//! 否则继续往后乐斗

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "历练";

pub async fn run(d: &DaLeDou) {
    let Some(visit_data) = visit(d).await else {
        return;
    };

    if visit_data.baseinfo.energy != "50" {
        return;
    }

    let Some(data) = query(d).await else {
        return;
    };

    // 还没有通关所有场景
    if data.next_npc_id != "0" {
        for _ in 0..5 {
            let Some(data) = query(d).await else {
                return;
            };

            乐斗(d, &data.cur_npc_id).await;
        }

        return;
    }

    let mut count = 0u8;
    for &boss in &d.config().历练.乐斗顺序 {
        let Some(data) = boss_info(d, boss.mapid()).await else {
            continue;
        };
        let Some(info) = data.monster_infos_.last() else {
            continue;
        };

        if info.challenge_times_ == "0" {
            continue;
        }

        乐斗(d, &info.monster_id_).await;
        count += 1;
        if count >= 5 {
            break;
        }
    }
}

#[derive(Deserialize)]
struct Visit {
    result: String,
    msg: String,
    baseinfo: BaseInfo,
}

#[derive(Deserialize)]
struct BaseInfo {
    energy: String, // 活力值
}

async fn visit(d: &DaLeDou) -> Option<Visit> {
    // 个人资料
    let cmd = format!("cmd=visit&puin={}&kind=1", d.qq());
    let data: Visit = match d.get(&cmd).await {
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
struct Query {
    result: String,
    msg: String,
    cur_npc_id: String,
    next_npc_id: String,
}

async fn query(d: &DaLeDou) -> Option<Query> {
    let data: Query = match d.get("cmd=mappush&type=0").await {
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
struct MapData {
    result: String,
    msg: String,
    monster_infos_: Vec<Infos>,
}

#[derive(Deserialize)]
struct Infos {
    monster_id_: String,
    challenge_times_: String, // 剩余挑战次数
}

async fn boss_info(d: &DaLeDou, mapid: &str) -> Option<MapData> {
    // 场景
    let cmd = format!("cmd=mappush&type=2&mapid={mapid}");
    let data: MapData = match d.get(&cmd).await {
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

async fn 乐斗(d: &DaLeDou, npcid: &str) {
    #[derive(Deserialize)]
    struct Fight {
        result: String,
        msg: String,
        repid: String,
    }

    // 乐斗
    let cmd = format!("cmd=mappush&type=1&npcid={npcid}");
    let data: Fight = match d.get(&cmd).await {
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

    乐斗记录(d, &data.repid).await;
}

async fn 乐斗记录(d: &DaLeDou, repid: &str) {
    #[derive(Deserialize)]
    struct View {
        result: String,
        msg: String,
        info: Vec<ViewInfo>,
    }

    #[derive(Deserialize)]
    struct ViewInfo {
        url: String,
        desc: String,
    }

    let cmd = format!("cmd=view&kind=2&sub=1&selfuin={}", d.qq());
    let data: View = match d.get(&cmd).await {
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

    for item in &data.info {
        if item.url == repid {
            d.log(TASK, &item.desc);
            return;
        }
    }
}
