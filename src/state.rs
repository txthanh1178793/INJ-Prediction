use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp, Uint128, Uint64};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::ops::Add;

#[cw_serde]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Info {
    pub id: u64,
    pub status: u8,
}

#[cw_serde]
#[derive(Default)]
pub struct BetInfo {
    pub upBet: Uint128,
    pub downBet: Uint128,
    pub totalPrize: Uint128,
    pub startPrice: Uint128,
    pub endPrice: Uint128,
    // pub decimals: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct CurrentBetDetail {
    // pub upBettor: Vec<Addr>,
    // pub downBetter: Vec<Addr>,
    pub totalUp: Uint128,
    pub totalDown: Uint128,
    pub startTime: Uint64,
    pub endTime: Uint64,
    pub startPrice: Uint128,
}

pub struct BetKey {
    addr: Addr,
    id: u64,
}

pub const OWNER: Item<Addr> = Item::new("Owner");
pub const INFO: Item<Info> = Item::new("info");
pub const CURRENTBET: Item<CurrentBetDetail> = Item::new("currentBet");
pub const BETINFO: Map<u64, BetInfo> = Map::new("betinfo");
pub const UP: Map<(&Addr, u64), Uint128> = Map::new("up");
pub const DOWN: Map<(&Addr, u64), Uint128> = Map::new("down");
pub const CLAIMED: Map<(&Addr, u64), bool> = Map::new("isClaimed");
