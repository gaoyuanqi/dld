//! 乐斗能量
//!
//! 领取、反向历练
//!
//! 历练：领取礼包后反向乐斗每关最后两个NPC

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "乐斗能量";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default, rename = "giftInfo")]
        gift_info: Vec<GiftInfo>,
    }

    #[derive(Deserialize)]
    struct GiftInfo {
        id: String,
        status: String,
    }

    // {"result":"0","msg":"","giftInfo":[{"id":"1","status":"1"},{"id":"2","status":"1"},{"id":"3","status":"0"}]}
    let data: Query = match d.get("cmd=newAct&subtype=100").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    // 不在活动时间
    if data.result == "-1" {
        return;
    }

    if data.result != "0" {
        d.log(TASK, &data.msg);
        return;
    }

    for item in &data.gift_info {
        // 可领取
        if item.status == "1" {
            领取(d, &item.id).await;
            历练(d).await;
        }
    }
}

async fn 领取(d: &DaLeDou, id: &str) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 领取
    let cmd = format!("cmd=newAct&subtype=100&op=1&id={id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 历练(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct LiLian {
        result: String,
        msg: String,
        cur_npc_id: String,
        map_id: String,
    }

    // {"result":"0","msg":"OK","cur_npc_id":"6394","next_npc_id":"0","max_npc_id":"6394","play_fight":"1","map_id":"20"}
    let data: LiLian = match d.get("cmd=mappush&type=0").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    if data.result != "0" {
        d.log(TASK, &format!("历练 => {}", data.msg));
        return;
    }

    let cur_npc_id: u32 = match data.cur_npc_id.parse() {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("解析 cur_npc_id 字段失败：{e}"));
            return;
        }
    };

    let current_map_id: u32 = match data.map_id.parse() {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("解析 map_id 字段失败：{e}"));
            return;
        }
    };

    let mut count = 0u8;
    let max_count = 3u8;
    'outer: for map_id in (1..=current_map_id).rev() {
        let Some(data) = boss_info(d, map_id).await else {
            continue;
        };

        for boss_npc_id in get_npc_id(map_id) {
            // 该关 boss 还没首次通关，不能乐斗，也不打它之后的 NPC
            if boss_npc_id > cur_npc_id {
                continue;
            }

            let boss_npc_id = boss_npc_id.to_string();
            let Some(info) = data
                .monster_infos_
                .iter()
                .find(|i| i.monster_id_ == boss_npc_id)
            else {
                continue;
            };

            if info.challenge_times_ == "0" {
                continue;
            }

            let challenge_times: u8 = match info.challenge_times_.parse() {
                Ok(v) => v,
                Err(e) => {
                    d.log(TASK, &format!("解析 challenge_times_ 字段失败：{e}"));
                    return;
                }
            };

            for _ in 0..challenge_times {
                乐斗(d, &info.monster_id_).await;
                count += 1;
                if count >= max_count {
                    break 'outer;
                }
            }
        }
    }
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

async fn boss_info(d: &DaLeDou, map_id: u32) -> Option<MapData> {
    // 场景
    let cmd = format!("cmd=mappush&type=2&mapid={map_id}");
    let data: MapData = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return None;
        }
    };

    if data.result != "0" {
        d.log(TASK, &format!("历练 => {}", data.msg));
        return None;
    }

    Some(data)
}

/// 每关最后两个NPC的id，mapid 1~20
fn get_npc_id(map_id: u32) -> [u32; 2] {
    // 第1关最后一个NPC的id是6014，每关20个NPC
    const FIRST_MAP_LAST_NPC: u32 = 6014;
    const NPCS_PER_MAP: u32 = 20;

    let last = FIRST_MAP_LAST_NPC + (map_id - 1) * NPCS_PER_MAP;
    [last, last - 1]
}

async fn 乐斗(d: &DaLeDou, npc_id: &str) {
    #[derive(Deserialize)]
    struct Fight {
        result: String,
        msg: String,
        repid: String,
    }

    // 乐斗
    let cmd = format!("cmd=mappush&type=1&npcid={npc_id}");
    let data: Fight = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    if data.result != "0" {
        d.log(TASK, &format!("历练 => {}", data.msg));
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
        d.log(TASK, &format!("历练 => {}", data.msg));
        return;
    }

    for item in &data.info {
        if item.url == repid {
            d.log(TASK, &format!("历练 => {}", item.desc));
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_npc_id() {
        assert_eq!(get_npc_id(20), [6394, 6393]);
        assert_eq!(get_npc_id(10), [6194, 6193]);
        assert_eq!(get_npc_id(1), [6014, 6013]);
    }

    #[test]
    fn test_get_npc_id_完整序列() {
        let ids: Vec<u32> = (1..=20).rev().flat_map(get_npc_id).collect();
        let expected = [
            6394, 6393, 6374, 6373, 6354, 6353, 6334, 6333, 6314, 6313, 6294, 6293, 6274, 6273,
            6254, 6253, 6234, 6233, 6214, 6213, 6194, 6193, 6174, 6173, 6154, 6153, 6134, 6133,
            6114, 6113, 6094, 6093, 6074, 6073, 6054, 6053, 6034, 6033, 6014, 6013,
        ];
        assert_eq!(ids, expected);
    }

    #[test]
    fn test_get_npc_id_成对连续() {
        for mapid in 1..=20 {
            let [last, second_last] = get_npc_id(mapid);
            assert_eq!(last - second_last, 1, "mapid {mapid} 最后两个 NPC 应相邻");
        }
    }
}
