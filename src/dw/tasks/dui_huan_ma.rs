//! 兑换码
//!
//! 周四微信兑换

use chrono::{Datelike, Local, Weekday};
use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "兑换码";

pub async fn run(d: &DaLeDou) {
    if Local::now().weekday() != Weekday::Thu {
        return;
    }

    #[derive(Deserialize)]
    struct WeiXin {
        msg: String,
    }

    let code = &d.global_config().兑换码.code;
    let cmd = format!("cmd=weixin&sub=1&cdkey={}", code);
    let data: WeiXin = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}
