//! 掠夺
//!
//! 周二掠夺、领奖；其它时间报名、领取胜负奖励
//!
//! 循环掠夺逻辑：
//! 1. 初始目标战力 = 配置的 `目标战力`
//! 2. 每次查询所有粮仓，找到第一个防守者战力 ≤ 当前目标战力的粮仓，对其进行掠夺
//! 3. 掠夺成功后，**重新查询** 粮仓状态（不提升目标战力），继续按相同目标战力寻找下一个可掠夺目标
//! 4. 若当前所有粮仓的防守者战力均高于当前目标战力：
//!    - `战力增量` > 0：目标战力 += `战力增量`，重复上述过程（上限 99999）
//!    - `战力增量` = 0：直接停止

use std::time::Duration;

use serde::Deserialize;
use tokio::time;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "掠夺";

#[derive(Deserialize)]
struct Response {
    result: String,
    msg: String,
}

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default)]
        status: String,
        #[serde(default)]
        time: String, // 战斗状态
        #[serde(default)]
        signup: String, // 是否可报名
        #[serde(default)]
        gift: String, // 是否可领取胜负奖励
    }

    let data: Query = match d.get("cmd=forage_war").await {
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

    // 战斗期
    if data.time == "2" {
        if !data.status.ends_with("未报名") {
            掠夺(d).await;
            领奖(d).await;
        }
        return;
    };

    // 可报名
    if data.signup == "1" {
        报名(d).await;
    };

    // 可领取
    if data.gift == "1" {
        领取胜负奖励(d).await;
    }
}

async fn 掠夺(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct ForageWarState {
        blood: String,            // 血量
        resurgence_times: String, // 剩余复活次数
        granary_info: Vec<GranaryInfo>,
    }

    #[derive(Deserialize)]
    struct GranaryInfo {
        id: String,                 // 粮仓id
        situation: String,          // 是否已占领该粮仓
        defend_list: Vec<Defender>, // 防守列表
    }

    #[derive(Deserialize)]
    struct Defender {
        power: String, // 战力
    }

    let mut target_power = d.config().掠夺.目标战力;
    let power_increment = d.config().掠夺.战力增量;
    let max_power: u32 = 99999;
    'outer: loop {
        let data: ForageWarState = match d.get("cmd=forage_war&subtype=3").await {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("{e}"));
                return;
            }
        };

        // 复活次数且血量为0
        if data.resurgence_times == "0" && data.blood.starts_with("0/") {
            return;
        }

        for item in &data.granary_info {
            // 已占领或者防守列表为空
            if item.situation == "1" || item.defend_list.is_empty() {
                continue;
            }

            // 该粮仓第一个成员
            let first = &item.defend_list[0];
            let first_power: f64 = match first.power.parse() {
                Ok(v) => v,
                Err(e) => {
                    d.log(TASK, &format!("解析 power 字段失败：{e}"));
                    return;
                }
            };

            if first_power < 0.0 || !first_power.is_finite() {
                d.log(TASK, &format!("power 数值异常：{first_power}"));
                return;
            }

            // 高于目标战力
            if first_power as u32 > target_power {
                continue;
            }

            if !执行掠夺(d, &item.id).await {
                return;
            }

            // 回到loop开头重新获取数据
            continue 'outer;
        }

        // 战力增量为 0 表示不递增，找不到就停止
        if power_increment == 0 {
            return;
        }

        // 所有粮仓都高于目标战力时递增
        target_power += power_increment;
        if target_power > max_power {
            return;
        }
        d.log(TASK, &format!("目标战力 => {}", target_power));
    }
}

async fn 执行掠夺(d: &DaLeDou, id: &str) -> bool {
    // 掠夺
    let cmd = format!("cmd=forage_war&subtype=4&gra_id={}", id);
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
    };

    挑战记录(d).await;
    time::sleep(Duration::from_millis(1500)).await;

    true
}

async fn 挑战记录(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct QueryRecord {
        result: String,
        msg: String,
        #[serde(rename = "feedDes")]
        feed: Vec<FeedDes>,
    }

    #[derive(Deserialize)]
    struct FeedDes {
        #[serde(rename = "feedDes")]
        des: String,
    }

    // 挑战记录
    let data: QueryRecord = match d.get("cmd=forage_war&subtype=7").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    if data.result != "0" {
        d.log(TASK, &data.msg);
        return;
    };

    if let Some(feed) = data.feed.first() {
        d.log(TASK, &feed.des);
    }
}

async fn 领奖(d: &DaLeDou) {
    // 领奖
    let data: Response = match d.get("cmd=forage_war&subtype=5").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 报名(d: &DaLeDou) {
    // 报名
    let data: Response = match d.get("cmd=forage_war&subtype=1").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

async fn 领取胜负奖励(d: &DaLeDou) {
    // 领取胜负奖励
    let data: Response = match d.get("cmd=forage_war&subtype=6").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
