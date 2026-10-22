//! 重阳太白诗会
//!
//! 领取重阳礼包、口令提交、领取赋诗奖赏礼包
//!
//! 虽然赋诗活动时间是每天18点到20点，但实测20点之后一段时间内也可以领取

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "重阳太白诗会";

pub async fn run(d: &DaLeDou) {
    let Some(data) = query(d).await else {
        return;
    };

    if data.day_gift == "1" {
        领取(d, "1").await;
    }

    if data.upload_poetry == "1" {
        输入今日口令(d, &data.show_poetry).await;
    }

    // 重新查询，刷新领奖状态
    let Some(data) = query(d).await else {
        return;
    };

    if data.poetry_gift == "1" {
        领取(d, "2").await;
    }
}

#[derive(Deserialize)]
struct Query {
    result: String,
    msg: String,
    show_poetry: String,   // 诗仙口令提示
    day_gift: String,      // 是否可领重阳礼包
    poetry_gift: String,   // 是否可以领取赋诗奖赏礼包
    upload_poetry: String, // 是否可以提交口令
}

async fn query(d: &DaLeDou) -> Option<Query> {
    // {"show_poetry":"孤帆远影碧空尽的下一句","act_time":"10月16日6点后至10月23日6点","active":"10","day_gift":"0","poetry_gift":"0","upload_poetry":"0","result":"0","msg":""}
    let data: Query = match d.get("cmd=newAct&subtype=157&op=0").await {
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

async fn 输入今日口令(d: &DaLeDou, show_poetry: &str) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    let Some((direction, line)) = 解析提示(show_poetry) else {
        d.log(TASK, &format!("{show_poetry} => 口令提示无法解析"));
        return;
    };
    let Some(poetry) = 查找答案(line, direction) else {
        d.log(TASK, &format!("{show_poetry} => 诗句库未收录"));
        return;
    };

    // 确认口令
    let cmd = format!("cmd=newAct&subtype=157&op=3&poetry={poetry}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &data.msg);
}

/// 诗句库：("上句", "下句")
///
/// 新增诗句追加在同长度组的末尾（五言在前、七言在后）
/// 全部诗对双向可查、同长度成组由测试锁定
const 诗句: &[(&str, &str)] = &[
    ("长风几万里", "吹度玉门关"),
    ("长安一片月", "万户捣衣声"),
    ("床前明月光", "疑是地上霜"),
    ("又疑瑶台镜", "飞在青云端"),
    ("危楼高百尺", "手可摘星辰"),
    ("孤帆远影碧空尽", "唯见长江天际流"),
    ("天生我材必有用", "千金散尽还复来"),
    ("飞流直下三千尺", "疑是银河落九天"),
    ("桃花潭水深千尺", "不及汪伦送我情"),
    ("杨花落尽子规啼", "闻道龙标过五溪"),
    ("大鹏一日同风起", "扶摇直上九万里"),
    ("两岸青山相对出", "孤帆一片日边来"),
    ("云想衣裳花想容", "春风拂槛露华浓"),
    ("长风万里送秋雁", "对此可以酣高楼"),
    ("金樽清酒斗十千", "玉盘珍羞直万钱"),
    ("凤凰台上凤凰游", "凤去台空江自流"),
];

/// 口令提示方向
#[derive(Debug, PartialEq, Eq)]
enum Direction {
    /// 上一句
    Prev,
    /// 下一句
    Next,
}

/// 从口令提示中解析方向和被提示的诗句
///
/// 形如「孤帆远影碧空尽的下一句」，其他形态返回 None
fn 解析提示(show_poetry: &str) -> Option<(Direction, &str)> {
    if let Some(line) = show_poetry.strip_suffix("的下一句") {
        return Some((Direction::Next, line));
    }
    if let Some(line) = show_poetry.strip_suffix("的上一句") {
        return Some((Direction::Prev, line));
    }
    None
}

/// 在诗句库中查找答案：按方向返回诗对的另一句
fn 查找答案(line: &str, direction: Direction) -> Option<&'static str> {
    诗句.iter().find_map(|&(first, second)| match direction {
        Direction::Next if first == line => Some(second),
        Direction::Prev if second == line => Some(first),
        _ => None,
    })
}

async fn 领取(d: &DaLeDou, op: &str) {
    #[derive(Deserialize)]
    struct Response {
        msg: String,
    }

    // 领取
    let cmd = format!("cmd=newAct&subtype=157&op={op}");
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

    // 解析「XX的下一句」
    #[test]
    fn test_解析提示_下一句() {
        assert_eq!(
            解析提示("孤帆远影碧空尽的下一句"),
            Some((Direction::Next, "孤帆远影碧空尽"))
        );
    }

    // 解析「XX的上一句」
    #[test]
    fn test_解析提示_上一句() {
        assert_eq!(
            解析提示("吹度玉门关的上一句"),
            Some((Direction::Prev, "吹度玉门关"))
        );
    }

    // 无「的下一句/的上一句」后缀
    #[test]
    fn test_解析提示_无后缀() {
        assert_eq!(解析提示("孤帆远影碧空尽"), None);
    }

    // 全部诗对双向都能查到另一句
    #[test]
    fn test_查找答案_全部诗对双向() {
        for &(first, second) in 诗句 {
            assert_eq!(查找答案(first, Direction::Next), Some(second));
            assert_eq!(查找答案(second, Direction::Prev), Some(first));
        }
    }

    // 诗句库未收录返回 None
    #[test]
    fn test_查找答案_未收录() {
        assert_eq!(查找答案("不存在的诗句", Direction::Next), None);
    }

    // 追加约定：五言在前、七言在后，同长度连续成组
    #[test]
    fn test_诗句_同长度成组() {
        let mut max_len = 0;
        for &(first, second) in 诗句 {
            let len = first.chars().count();
            assert!(
                len >= max_len,
                "「{first}」长度 {len} 应排在长度 {max_len} 组之后"
            );
            assert_eq!(
                len,
                second.chars().count(),
                "「{first}，{second}」上下句长度不一致"
            );
            max_len = len;
        }
    }
}
