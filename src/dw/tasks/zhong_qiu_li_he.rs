//! 中秋礼盒
//!
//! 领取任务奖励

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "中秋礼盒";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default)]
        package_list: Vec<Package>,
    }

    #[derive(Deserialize)]
    struct Package {
        task_id: String,
        level_num: String,       // 总等级数
        award_level_num: String, // 已领等级数
        finish_num: String,      // 当前进度
        limit_num: String,       // 进度上限
    }

    let data: Query = match d.get("cmd=midautumngiftbag&sub=0").await {
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

    for item in &data.package_list {
        // 已领取完该系列任务所有奖励
        if item.level_num == item.award_level_num {
            continue;
        }

        let finish_num: u32 = match item.finish_num.parse() {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("解析 finish_num 字段失败：{e}"));
                return;
            }
        };
        let limit_num: u32 = match item.limit_num.parse() {
            Ok(v) => v,
            Err(e) => {
                d.log(TASK, &format!("解析 limit_num 字段失败：{e}"));
                return;
            }
        };

        if finish_num >= limit_num {
            领取(d, &item.task_id).await;
        }
    }
}

async fn 领取(d: &DaLeDou, id: &str) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 领取
    let cmd = format!("cmd=midautumngiftbag&sub=1&id={id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
