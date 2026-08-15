//! 每日宝箱
//!
//! 打开

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "每日宝箱";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        #[serde(default)]
        au_num: String, // 金质宝箱
        #[serde(default)]
        ag_num: String, // 银质宝箱
        #[serde(default)]
        cu_num: String, // 铜质宝箱
    }

    let data: Query = match d.get("cmd=dailychest").await {
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

    let au_count: u64 = data.au_num.parse().unwrap_or(0) / 30;
    let ag_count: u64 = data.ag_num.parse().unwrap_or(0) / 14;
    let cu_count: u64 = data.cu_num.parse().unwrap_or(0) / 24;

    for (count, t) in [(au_count, 2), (ag_count, 1), (cu_count, 0)] {
        打开(d, count, t).await;
    }
}

async fn 打开(d: &DaLeDou, count: u64, t: u8) {
    #[derive(Deserialize)]
    struct Response {
        result: String,
        msg: String,
    }

    for _ in 0..count {
        // 打开
        let cmd = format!("cmd=dailychest&op=open&type={t}");
        let data: Response = match d.get(&cmd).await {
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
    }
}
