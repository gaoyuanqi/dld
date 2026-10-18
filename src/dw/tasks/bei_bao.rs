//! 背包任务
//!
//! 使用锦囊类物品、开启名称以「宝箱」结尾的物品、配置的物品

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "背包";

pub async fn run(d: &DaLeDou) {
    锦囊(d).await;
    宝箱(d).await;
    使用配置(d).await;
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

async fn 宝箱(d: &DaLeDou) {
    let Some(data) = query(d).await else {
        return;
    };

    for item in &data.bag {
        if item.name.ends_with("宝箱") {
            使用(d, item).await;
        }
    }
}

async fn 使用配置(d: &DaLeDou) {
    let Some(data) = query(d).await else {
        return;
    };

    for item in 配置物品(&data.bag, &d.config().背包.使用) {
        使用(d, item).await;
    }
}

/// 从背包中筛选出名称在配置列表里的物品，保持背包顺序
fn 配置物品<'a>(bag: &'a [Bag], names: &[String]) -> Vec<&'a Bag> {
    bag.iter()
        .filter(|item| names.iter().any(|n| n == &item.name))
        .collect()
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

        if data.result != "0" {
            d.log(TASK, &format!("{} => {}", bag.name, data.msg));
            return;
        }
        d.log(TASK, &data.msg);
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

#[cfg(test)]
mod tests {
    use super::*;

    // 构造测试用背包物品
    fn bag_item(id: &str, name: &str) -> Bag {
        Bag {
            id: id.to_string(),
            name: name.to_string(),
            num: "1".to_string(),
            storage: "1".to_string(),
        }
    }

    // 命中配置名的物品被筛出，按背包顺序
    #[test]
    fn test_配置物品_matches() {
        let bag = vec![
            bag_item("1", "包子"),
            bag_item("2", "宝剑"),
            bag_item("3", "肉包子"),
        ];
        let names = vec!["宝剑".to_string(), "包子".to_string()];
        let result = 配置物品(&bag, &names);
        let matched: Vec<_> = result.iter().map(|b| b.name.as_str()).collect();
        assert_eq!(matched, ["包子", "宝剑"]);
    }

    // 配置名不在背包中时被忽略；空配置返回空
    #[test]
    fn test_配置物品_no_match_and_empty() {
        let bag = vec![bag_item("1", "包子")];
        assert!(配置物品(&bag, &["不存在".to_string()]).is_empty());
        assert!(配置物品(&bag, &[]).is_empty());
    }
}
