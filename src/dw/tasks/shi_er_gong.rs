//! 十二宫自动选择最高场景请猴王扫荡

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "十二宫";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        left_challenge_times: String,
        max_scene_id: String,
    }

    let data: Query = match d.get("cmd=zodiacdungeon&op=query").await {
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

    if data.left_challenge_times == "1" {
        请猴王扫荡(d, &data.max_scene_id).await;
    }
}

async fn 请猴王扫荡(d: &DaLeDou, max_scene_id: &str) {
    #[derive(Deserialize)]
    struct Autofight {
        result: String,
        msg: String,
        #[serde(default)]
        records: Vec<String>,
    }

    // 请猴王扫荡
    let cmd = format!("cmd=zodiacdungeon&op=autofight&scene_id={}", max_scene_id);
    let data: Autofight = match d.get(&cmd).await {
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

    for m in data.records.iter().filter(|s| !s.is_empty()) {
        d.log(TASK, m);
    }
}
