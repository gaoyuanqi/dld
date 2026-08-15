//! 背包任务
//!
//! 使用锦囊类物品，开启名称以「宝箱」「食盒」结尾的物品

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "背包";

pub async fn run(d: &DaLeDou) {
    锦囊(d).await;
    箱盒(d).await;
}

async fn 锦囊(d: &DaLeDou) {
    let Some(data) = query(d).await else {
        return;
    };

    for item in &data.bag {
        // 分类 5 为锦囊类物品
        if item.storage == "5" {
            使用(d, item).await;
        }
    }
}

async fn 箱盒(d: &DaLeDou) {
    let Some(data) = query(d).await else {
        return;
    };

    for item in &data.bag {
        if item.name.ends_with("宝箱") || item.name.ends_with("食盒") {
            使用(d, item).await;
        }
    }
}

async fn 使用(d: &DaLeDou, bag: &Bag) {
    #[derive(Deserialize)]
    struct Response {
        result: String,
        msg: String,
    }

    let num: u32 = match bag.num.parse() {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("解析 {} num 字段失败：{e}", bag.name));
            return;
        }
    };

    for _ in 0..num {
        let cmd = format!("cmd=use&selfuin={}&id={}", d.qq(), bag.id);
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

#[derive(Deserialize)]
struct Query {
    result: String,
    msg: String,
    #[serde(default)]
    bag: Vec<Bag>,
}

#[derive(Deserialize)]
struct Bag {
    id: String,
    name: String,
    num: String,
    storage: String, // 物品分类（"5" 为锦囊）
}

async fn query(d: &DaLeDou) -> Option<Query> {
    let cmd = format!("cmd=view&kind=0&sub=2&type=4&selfuin={}", d.qq());
    let data: Query = match d.get(&cmd).await {
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
