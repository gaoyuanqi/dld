//! 画卷迷踪
//!
//! 如果有免费次数则执行准备完成进入战斗，战败直接结束

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "画卷迷踪";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default)]
        free_times: String, // 免费次数
    }

    let data: Query = match d.get("cmd=scroll_dungeon").await {
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

    if data.free_times == "1" {
        准备完成进入战斗(d).await;
    }
}

async fn 准备完成进入战斗(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Response {
        result: String,
        msg: String,
    }

    for _ in 0..80 {
        // 准备完成进入战斗
        let data: Response = match d.get("cmd=scroll_dungeon&op=fight&buff=0").await {
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

        if data.msg.starts_with("弱爆了") {
            return;
        }
    }
}
