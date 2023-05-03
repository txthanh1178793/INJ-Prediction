use crate::state::BetInfo;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Timestamp, Uint128, Uint64};
use cosmwasm_std::{Binary, Coin};
use std::time::Duration;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Start { price: Uint128 },
    End { price: Uint128 },
    UpBet {},
    DownBet {},
    ClaimReward { bet_id: u64 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(CurrentInfoResponse)]
    CurrentInfo { addr: Addr },

    #[returns(BetInfo)]
    BetInfo { bet_id: u64 },

    #[returns(Uint128)]
    UserReward { addr: Addr, bet_id: u64 },

    #[returns(Uint64)]
    TimeStampInfo {},
}

#[cw_serde]
pub struct CurrentInfoResponse {
    pub id: u64,
    pub status: u8,
    pub totalUp: Uint128,
    pub totalDown: Uint128,
    pub startTime: Uint64,
    pub endTime: Uint64,
    pub startPrice: Uint128,
    pub upPosition: Uint128,
    pub downPosition: Uint128,
}
