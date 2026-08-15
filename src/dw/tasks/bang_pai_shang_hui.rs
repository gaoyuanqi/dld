//! 帮派商会
//!
//! 领取、交易、兑换

use serde::Deserialize;

use crate::dw::daledou::DaLeDou;

const TASK: &str = "帮派商会";

#[derive(Deserialize)]
struct Response {
    result: String,
    msg: String,
}

pub async fn run(d: &DaLeDou) {
    // 帮派宝库
    let data: Response = match d.get("cmd=fac_corp&op=0").await {
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

    帮派宝库(d).await;
    交易会所(d).await;
    兑换商店(d).await;
}

async fn 帮派宝库(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct BaoKu {
        result: String,
        msg: String,
        #[serde(rename = "giftInfo")]
        gift_info: Vec<GiftInfo>,
    }

    #[derive(Deserialize)]
    struct GiftInfo {
        #[serde(rename = "giftId")]
        gift_id: String,
        #[serde(default, rename = "type")]
        t: String,
    }

    for _ in 0..10 {
        // 帮派宝库
        let data: BaoKu = match d.get("cmd=fac_corp&op=0").await {
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

        if data.gift_info.is_empty() {
            return;
        }

        for item in &data.gift_info {
            if !领取(d, &item.gift_id, &item.t).await {
                return;
            }
        }
    }
}

async fn 领取(d: &DaLeDou, id: &str, t: &str) -> bool {
    // 领取
    let cmd = format!("cmd=fac_corp&op=3&giftId={id}&type={t}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return false;
        }
    };

    d.log(TASK, &data.msg);
    data.result == "0"
}

async fn 交易会所(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct HuiSuo {
        result: String,
        msg: String,
        #[serde(rename = "tradeInfo")]
        trade_info: Vec<TradeInfo>,
    }

    #[derive(Deserialize)]
    struct TradeInfo {
        tips: String,
        #[serde(rename = "goodsId")]
        goods_id: String,
        #[serde(rename = "isTraded")]
        is_traded: String, // 是否已交易
        #[serde(rename = "type")]
        t: String,
    }

    // 交易会所
    let data: HuiSuo = match d.get("cmd=fac_corp&op=1").await {
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

    for item in &data.trade_info {
        // 已交易
        if item.is_traded == "1" {
            continue;
        }

        if !d.config().帮派商会.交易会所.is_match(&item.tips) {
            continue;
        }
        交易(d, &item.t, &item.goods_id, &item.tips).await;
    }
}

async fn 交易(d: &DaLeDou, t: &str, id: &str, tips: &str) {
    // 交易
    let cmd = format!("cmd=fac_corp&op=4&type={t}&goods_id={id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    let item = tips
        .split('要')
        .nth(1)
        .and_then(|s| s.split('，').next())
        .unwrap_or(tips);
    d.log(TASK, &format!("{item} => {}", data.msg));
}

async fn 兑换商店(d: &DaLeDou) {
    #[derive(Deserialize)]
    struct DuiHuan {
        result: String,
        msg: String,
        #[serde(rename = "exchangeInfo")]
        exchange_info: Vec<ExchangeInfo>,
    }

    #[derive(Deserialize)]
    struct ExchangeInfo {
        #[serde(rename = "typeId")]
        type_id: String,
        #[serde(rename = "goodsName")]
        goods_name: String, // 物品名称
        #[serde(rename = "isExchanged")]
        is_exchanged: String, // 是否已兑换
    }

    // 兑换商店
    let data: DuiHuan = match d.get("cmd=fac_corp&op=2").await {
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

    let exchange = &d.config().帮派商会.兑换商店;
    for item in &data.exchange_info {
        // 已兑换
        if item.is_exchanged == "1" || !exchange.should_exchange(&item.goods_name) {
            continue;
        }

        兑换(d, &item.goods_name, &item.type_id).await;
    }
}

async fn 兑换(d: &DaLeDou, name: &str, id: &str) {
    // 兑换
    let cmd = format!("cmd=fac_corp&op=5&type_id={id}");
    let data: Response = match d.get(&cmd).await {
        Ok(v) => v,
        Err(e) => {
            d.log(TASK, &format!("{e}"));
            return;
        }
    };

    d.log(TASK, &format!("{name} => {}", data.msg));
}
