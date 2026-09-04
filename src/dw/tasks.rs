mod bai_pai_ji_tan;
mod bang_pai_shang_hui;
mod bei_bao;
mod biao_xing_tian_xia;
mod cai_dan_shuang;
mod da_xia_hui_gui;
mod deng_lu_shang_dian;
mod deng_lu_you_li;
mod dian_feng_zhi_zhan;
mod dou_jing_tan_mi;
mod dui_huan_ma;
mod fei_sheng;
mod fen_xiang;
mod hao_jie_bao_xiang;
mod hao_li_ti_sheng;
mod hao_xia_chu_shi;
mod hua_juan_mi_zong;
mod hua_shan_lun_jian;
mod huan_jing;
mod hui_wu;
mod hui_zhang_zhan_ling;
mod huo_yue_li_bao;
mod ji_yun_pai;
mod jiang_hu_chang_meng;
mod jie_bai;
mod jin_ri_huo_yue_du;
mod jing_ji_chang;
mod kuang_dong;
mod le_dou;
mod le_dou_cai_dan;
mod le_dou_huang_li;
mod le_dou_yi_zhan;
mod le_dou_you_ji;
mod li_lian;
mod lian_sai;
mod ling_qu_tu_di;
mod long_huang_zhi_jing;
mod lve_duo;
mod mei_ri_bao_xiang;
mod mei_ri_jiang_li;
mod men_pai;
mod men_pai_yao_qing_sai;
mod meng_xiang_zhi_lv;
mod mi_ji_feng_yin;
mod qi_hun_fu_mo;
mod qiang_di_pan;
mod quan_min_luan_dou;
mod qun_xia;
mod qun_xiong_zhu_lu;
mod ren_wu;
mod ren_wu_pai_qian;
mod shen_mo_zhuan_pan;
mod shen_yuan_mi_bao;
mod shen_yuan_zhi_chao;
mod shi_er_gong;
mod shi_jie_shu;
mod shi_kong_yi_ji;
mod ti_guan;
mod wa_wa_ji;
mod wen_ding_tian_xia;
mod wo_de_bang_pai;
mod wu_lin;
mod wu_lin_meng_zhu;
mod xia_ke_dao;
mod xia_lv;
mod xia_shi_ke_zhan;
mod xian_wu_xiu_zhen;
mod xie_shen_mi_bao;
mod xing_yun_jin_dan;
mod xing_yun_zhuan_pan;
mod xu_yuan;
mod yuan_wu_deng_gao;
mod yuan_zheng;
mod zhi_ye_tiao_zhan;
mod zhou_zhou_li_bao;

use crate::dw::daledou::DaLeDou;

#[derive(Clone, Debug)]
pub enum Task {
    分享,
    乐斗,
    武林,
    结拜,
    侠侣,
    群侠,
    矿洞,
    掠夺,
    踢馆,
    许愿,
    历练,
    幻境,
    门派,
    会武,
    背包,
    竞技场,
    十二宫,
    抢地盘,
    侠客岛,
    世界树,
    每日奖励,
    每日宝箱,
    邪神秘宝,
    华山论剑,
    巅峰之战,
    镖行天下,
    群雄逐鹿,
    画卷迷踪,
    任务,
    帮派祭坛,
    梦想之旅,
    问鼎天下,
    帮派商会,
    武林盟主,
    全民乱斗,
    侠士客栈,
    江湖长梦,
    深渊之潮,
    时空遗迹,
    龙凰之境,
    我的帮派,
    门派邀请赛,
    帮派远征军,
    飞升大作战,
    今日活跃度,
    帮派黄金联赛,
    任务派遣中心,
    领取徒弟经验,
    仙武修真,
    乐斗黄历,
    器魂附魔,
    兑换码,
    激运牌,
    猜单双,
    娃娃机,
    乐斗驿站,
    神魔转盘,
    登录有礼,
    徽章战令,
    职业挑战,
    斗境探秘,
    深渊秘宝,
    活跃礼包,
    乐斗游记,
    浩劫宝箱,
    周周礼包,
    好礼提升,
    幸运金蛋,
    元武登高,
    乐斗菜单,
    幸运转盘,
    大侠回归,
    登录商店,
    豪侠出世,
    秘籍封印,
}

impl Task {
    /// 返回全部任务列表（内部使用）
    pub fn all() -> &'static [Task] {
        &[
            Task::分享,
            Task::乐斗,
            Task::武林,
            Task::结拜,
            Task::侠侣,
            Task::群侠,
            Task::矿洞,
            Task::掠夺,
            Task::踢馆,
            Task::许愿,
            Task::历练,
            Task::幻境,
            Task::门派,
            Task::会武,
            Task::背包,
            Task::竞技场,
            Task::十二宫,
            Task::抢地盘,
            Task::侠客岛,
            Task::世界树,
            Task::每日奖励,
            Task::每日宝箱,
            Task::邪神秘宝,
            Task::华山论剑,
            Task::巅峰之战,
            Task::镖行天下,
            Task::群雄逐鹿,
            Task::画卷迷踪,
            Task::任务,
            Task::帮派祭坛,
            Task::梦想之旅,
            Task::问鼎天下,
            Task::帮派商会,
            Task::武林盟主,
            Task::全民乱斗,
            Task::侠士客栈,
            Task::江湖长梦,
            Task::深渊之潮,
            Task::时空遗迹,
            Task::龙凰之境,
            Task::我的帮派,
            Task::门派邀请赛,
            Task::帮派远征军,
            Task::飞升大作战,
            Task::今日活跃度,
            Task::帮派黄金联赛,
            Task::任务派遣中心,
            Task::领取徒弟经验,
            Task::仙武修真,
            Task::乐斗黄历,
            Task::器魂附魔,
            Task::兑换码,
            Task::激运牌,
            Task::猜单双,
            Task::娃娃机,
            Task::乐斗驿站,
            Task::神魔转盘,
            Task::登录有礼,
            Task::徽章战令,
            Task::职业挑战,
            Task::斗境探秘,
            Task::深渊秘宝,
            Task::活跃礼包,
            Task::乐斗游记,
            Task::浩劫宝箱,
            Task::周周礼包,
            Task::好礼提升,
            Task::幸运金蛋,
            Task::元武登高,
            Task::乐斗菜单,
            Task::幸运转盘,
            Task::大侠回归,
            Task::登录商店,
            Task::豪侠出世,
            Task::秘籍封印,
        ]
    }
}

/// 运行单个任务
pub async fn run_task(d: &DaLeDou, name: &Task) {
    match name {
        Task::分享 => fen_xiang::run(d).await,
        Task::乐斗 => le_dou::run(d).await,
        Task::武林 => wu_lin::run(d).await,
        Task::结拜 => jie_bai::run(d).await,
        Task::侠侣 => xia_lv::run(d).await,
        Task::群侠 => qun_xia::run(d).await,
        Task::矿洞 => kuang_dong::run(d).await,
        Task::掠夺 => lve_duo::run(d).await,
        Task::踢馆 => ti_guan::run(d).await,
        Task::许愿 => xu_yuan::run(d).await,
        Task::历练 => li_lian::run(d).await,
        Task::幻境 => huan_jing::run(d).await,
        Task::门派 => men_pai::run(d).await,
        Task::会武 => hui_wu::run(d).await,
        Task::背包 => bei_bao::run(d).await,
        Task::竞技场 => jing_ji_chang::run(d).await,
        Task::十二宫 => shi_er_gong::run(d).await,
        Task::抢地盘 => qiang_di_pan::run(d).await,
        Task::侠客岛 => xia_ke_dao::run(d).await,
        Task::世界树 => shi_jie_shu::run(d).await,
        Task::每日奖励 => mei_ri_jiang_li::run(d).await,
        Task::每日宝箱 => mei_ri_bao_xiang::run(d).await,
        Task::邪神秘宝 => xie_shen_mi_bao::run(d).await,
        Task::华山论剑 => hua_shan_lun_jian::run(d).await,
        Task::巅峰之战 => dian_feng_zhi_zhan::run(d).await,
        Task::镖行天下 => biao_xing_tian_xia::run(d).await,
        Task::群雄逐鹿 => qun_xiong_zhu_lu::run(d).await,
        Task::画卷迷踪 => hua_juan_mi_zong::run(d).await,
        Task::任务 => ren_wu::run(d).await,
        Task::帮派祭坛 => bai_pai_ji_tan::run(d).await,
        Task::梦想之旅 => meng_xiang_zhi_lv::run(d).await,
        Task::问鼎天下 => wen_ding_tian_xia::run(d).await,
        Task::帮派商会 => bang_pai_shang_hui::run(d).await,
        Task::武林盟主 => wu_lin_meng_zhu::run(d).await,
        Task::全民乱斗 => quan_min_luan_dou::run(d).await,
        Task::侠士客栈 => xia_shi_ke_zhan::run(d).await,
        Task::江湖长梦 => jiang_hu_chang_meng::run(d).await,
        Task::深渊之潮 => shen_yuan_zhi_chao::run(d).await,
        Task::时空遗迹 => shi_kong_yi_ji::run(d).await,
        Task::龙凰之境 => long_huang_zhi_jing::run(d).await,
        Task::我的帮派 => wo_de_bang_pai::run(d).await,
        Task::门派邀请赛 => men_pai_yao_qing_sai::run(d).await,
        Task::帮派远征军 => yuan_zheng::run(d).await,
        Task::飞升大作战 => fei_sheng::run(d).await,
        Task::今日活跃度 => jin_ri_huo_yue_du::run(d).await,
        Task::帮派黄金联赛 => lian_sai::run(d).await,
        Task::任务派遣中心 => ren_wu_pai_qian::run(d).await,
        Task::领取徒弟经验 => ling_qu_tu_di::run(d).await,
        Task::仙武修真 => xian_wu_xiu_zhen::run(d).await,
        Task::乐斗黄历 => le_dou_huang_li::run(d).await,
        Task::器魂附魔 => qi_hun_fu_mo::run(d).await,
        Task::兑换码 => dui_huan_ma::run(d).await,
        Task::激运牌 => ji_yun_pai::run(d).await,
        Task::猜单双 => cai_dan_shuang::run(d).await,
        Task::娃娃机 => wa_wa_ji::run(d).await,
        Task::乐斗驿站 => le_dou_yi_zhan::run(d).await,
        Task::神魔转盘 => shen_mo_zhuan_pan::run(d).await,
        Task::登录有礼 => deng_lu_you_li::run(d).await,
        Task::徽章战令 => hui_zhang_zhan_ling::run(d).await,
        Task::职业挑战 => zhi_ye_tiao_zhan::run(d).await,
        Task::斗境探秘 => dou_jing_tan_mi::run(d).await,
        Task::深渊秘宝 => shen_yuan_mi_bao::run(d).await,
        Task::活跃礼包 => huo_yue_li_bao::run(d).await,
        Task::乐斗游记 => le_dou_you_ji::run(d).await,
        Task::浩劫宝箱 => hao_jie_bao_xiang::run(d).await,
        Task::周周礼包 => zhou_zhou_li_bao::run(d).await,
        Task::好礼提升 => hao_li_ti_sheng::run(d).await,
        Task::幸运金蛋 => xing_yun_jin_dan::run(d).await,
        Task::元武登高 => yuan_wu_deng_gao::run(d).await,
        Task::乐斗菜单 => le_dou_cai_dan::run(d).await,
        Task::幸运转盘 => xing_yun_zhuan_pan::run(d).await,
        Task::大侠回归 => da_xia_hui_gui::run(d).await,
        Task::登录商店 => deng_lu_shang_dian::run(d).await,
        Task::豪侠出世 => hao_xia_chu_shi::run(d).await,
        Task::秘籍封印 => mi_ji_feng_yin::run(d).await,
    }
}
