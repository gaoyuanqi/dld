//! 万圣节
//!
//! 点亮：按照顺序点亮
//! 历练：如果获得活力则反向乐斗每关最后两个NPC
//! 兑奖：礼包兑换优先级为B、A；周四兑换
//!
//! 每天最多获得10个蜡烛
//! 活动结束蜡烛和南瓜灯数量清零

use chrono::{Datelike, Local};
use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "万圣节";

#[derive(Deserialize)]
struct Query {
    result: String,
    msg: String,
    #[serde(default)]
    candle: String, // 蜡烛数量
    #[serde(default)]
    lantern: String, // 南瓜灯数量
    #[serde(default)]
    state: String, // 10 个南瓜灯状态
}

pub async fn run(d: &DaLeDou) {
    点亮南瓜灯(d).await;
    兑换奖励(d).await;
}

async fn query(d: &DaLeDou) -> Option<Query> {
    // {"result":"0","msg":"","candle":"8","lantern":"0","state":"1|0|0|0|0|0|0|0|0|0"}
    let data: Query = match d.get("cmd=hallowmas").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return None;
        }
    };

    // 不在活动时间
    if data.result == "-1" {
        return None;
    }

    if data.result != "0" {
        d.log(TASK, &data.msg);
        return None;
    }

    Some(data)
}

async fn 点亮南瓜灯(d: &DaLeDou) {
    let Some(data) = query(d).await else {
        return;
    };

    let candle: u8 = match data.candle.parse() {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("解析 candle 字段失败：{e}"));
            return;
        }
    };

    for _ in 0..candle {
        let Some(data) = query(d).await else {
            return;
        };

        // 第一个为1的索引
        let Some(id) = first_lightable_index(&data.state) else {
            d.log(TASK, "没有找到可点亮南瓜灯位置索引");
            return;
        };

        // 南瓜
        // {"result":"0","msg":"恭喜您获得10活力和南瓜灯一个！","candle":"5","lantern":"1","state":"2|1|1|0|0|0|0|0|0|0"}
        // {"result":"0","msg":"真遗憾，南瓜灯变成一个鬼魂飞走了！","candle":"4","lantern":"1","state":"2|3|1|0|0|0|0|0|0|0"}
        // {"result":"-1","msg":"这是个鬼魂，不能点亮！","candle":"4","lantern":"1","state":"2|3|1|0|0|0|0|0|0|0"}
        // {"result":"-1","msg":"第一排南瓜已经点亮了!","candle":"5","lantern":"1","state":"2|1|1|0|0|0|0|0|0|0"}
        let cmd = format!("cmd=hallowmas&gb_id=4&cushaw_id={id}");
        let data: Query = match d.get(&cmd).await {
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

        if data.msg.contains("恭喜您获得10活力") {
            历练(d, 1).await;
        } else if data.msg.contains("恭喜您获得20活力") {
            历练(d, 2).await;
        }
    }
}

/// 找 state 中第一个值为 "1" 的位置索引，找不到返回 None
fn first_lightable_index(state: &str) -> Option<usize> {
    state.split('|').position(|v| v == "1")
}

async fn 历练(d: &DaLeDou, max_count: u8) {
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

async fn 兑换奖励(d: &DaLeDou) {
    // 只在星期四兑换
    if Local::now().weekday() != chrono::Weekday::Thu {
        return;
    }

    let Some(data) = query(d).await else {
        return;
    };

    let lantern: u8 = match data.lantern.parse() {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("解析 lantern 字段失败：{e}"));
            return;
        }
    };

    for id in exchange_plan(lantern) {
        兑换(d, id).await;
    }
}

/// 按优先级 B（40 个）→ A（20 个）计算兑换礼包 id 列表
fn exchange_plan(lantern: u8) -> &'static [&'static str] {
    match lantern {
        60..=u8::MAX => &["6", "5"],
        40..=59 => &["6"],
        20..=39 => &["5"],
        _ => &[],
    }
}

async fn 兑换(d: &DaLeDou, id: &str) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 兑换
    let cmd = format!("cmd=hallowmas&gb_id={id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_lightable_index() {
        assert_eq!(first_lightable_index("1|0|0|0"), Some(0));
        assert_eq!(first_lightable_index("0|0|1|0"), Some(2));
        assert_eq!(first_lightable_index("2|3|1|0|0|0|0|0|0|0"), Some(2));
        assert_eq!(first_lightable_index("0|0|0|0"), None);
        assert_eq!(first_lightable_index(""), None);
    }

    #[test]
    fn test_exchange_plan() {
        assert_eq!(exchange_plan(60), &["6", "5"][..]);
        assert_eq!(exchange_plan(40), &["6"][..]);
        assert_eq!(exchange_plan(39), &["5"][..]);
        assert_eq!(exchange_plan(20), &["5"][..]);
        assert!(exchange_plan(19).is_empty());
    }

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
