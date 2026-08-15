//! 领取徒弟经验
//!
//! 领取

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "领取徒弟经验";

pub async fn run(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Query {
        result: String,
        msg: String,
        tudi: Vec<TuDi>,
        baseinfo: BaseInfo,
    }

    #[derive(Deserialize)]
    struct TuDi {
        flag: String, // 是否真实徒弟
    }

    #[derive(Deserialize)]
    struct BaseInfo {
        expflag: String, // 是否可领取
    }

    // 玩家资料
    let cmd = format!("cmd=visit&puin={}&kind=1", d.qq());
    let data: Query = match d.get(&cmd).await {
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

    // 可领取
    if data.baseinfo.expflag == "1" {
        for item in &data.tudi {
            // 真实徒弟，非系统推荐徒弟
            if item.flag == "0" {
                领取(d).await;
                return;
            }
        }
    }
}

async fn 领取(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 领取
    let data: Response = match d.get("cmd=getexp").await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
