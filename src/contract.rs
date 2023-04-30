use crate::error::ContractError;
use crate::msg::{CurrentInfoResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{BetInfo, CurrentBetDetail, Info};
use crate::state::{BETINFO, CLAIMED, CURRENTBET, DOWN, INFO, OWNER, UP};
use std::default;
use std::time::Duration;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Response, StdError, StdResult, SubMsg, Timestamp, Uint128, Uint64,
};
use cw_storage_plus::{Item, Map};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let owner = _info.sender;
    let info: Info = Info { id: 0, status: 0 };
    let current_bet: CurrentBetDetail = CurrentBetDetail::default();
    // let bet_info: Vec<BetInfo> = Vec::new();
    // let up: Vec<Uint128> = Vec::new();
    // let down: Vec<Uint128> = Vec::new();

    OWNER.save(deps.storage, &owner)?;
    INFO.save(deps.storage, &info)?;
    CURRENTBET.save(deps.storage, &current_bet)?;
    // BETINFO.save(deps.storage, &bet_info)?;
    // UP.save(deps.storage, &up)?;
    // DOWN.save(deps.storage, &down)?;
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Start { price } => execute_start_bet(deps, _env, _info, price),
        ExecuteMsg::End { price } => execute_end_bet(deps, _env, _info, price),
        ExecuteMsg::UpBet {} => execute_up_bet(deps, _env, _info),
        ExecuteMsg::DownBet {} => execute_down_bet(deps, _env, _info),
        ExecuteMsg::ClaimReward { bet_id } => execute_claim(deps, _env, _info, bet_id),
    }
}

pub fn execute_start_bet(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    price: Uint128,
) -> Result<Response, ContractError> {
    let owner = OWNER.load(deps.storage)?;
    let info = INFO.load(deps.storage)?;
    if owner != _info.sender {
        return Err(ContractError::Unauthorized {});
    }
    if info.status != 0 {
        return Err(ContractError::NotPendingID {});
    }

    let info: Info = Info {
        id: info.id,
        status: 1,
    };

    let bet_detail = CurrentBetDetail {
        // upBettor: Vec::new(),
        // downBetter: Vec::new(),
        totalUp: Uint128::from(0u128),
        totalDown: Uint128::from(0u128),
        startTime: Uint64::from(_env.block.time.seconds()),
        endTime: Uint64::from(_env.block.time.seconds() + 300),
        startPrice: price,
    };

    CURRENTBET.save(deps.storage, &bet_detail)?;
    INFO.save(deps.storage, &info);

    Ok(Response::new()
        .add_attribute("action", "execute_start_bet")
        .add_attribute("start_price", price))
}

pub fn execute_end_bet(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    price: Uint128,
) -> Result<Response, ContractError> {
    let owner = OWNER.load(deps.storage)?;
    let info = INFO.load(deps.storage)?;
    let current_bet_detail = CURRENTBET.load(deps.storage)?;
    if owner != _info.sender {
        return Err(ContractError::Unauthorized {});
    }
    if info.status != 1 {
        return Err(ContractError::NotOpeningID {});
    }
    if Uint64::from(_env.block.time.seconds()) < current_bet_detail.endTime {
        return Err(ContractError::BetIDNotEnd {});
    }

    let temp_id = info.id.into();
    let prize = (current_bet_detail.totalUp + current_bet_detail.totalDown) * Uint128::from(99u128)
        / Uint128::from(100u128);
    let fee = current_bet_detail.totalUp + current_bet_detail.totalDown - prize;

    let bet_info = BetInfo {
        upBet: current_bet_detail.totalUp,
        downBet: current_bet_detail.totalDown,
        totalPrize: prize,
        startPrice: current_bet_detail.startPrice,
        endPrice: price,
        // decimals: current_bet_detail.decimals,
    };
    BETINFO.save(deps.storage, temp_id, &bet_info)?;

    let info: Info = Info {
        id: info.id + 1,
        status: 0,
    };
    INFO.save(deps.storage, &info)?;

    let current_bet: CurrentBetDetail = CurrentBetDetail::default();
    CURRENTBET.save(deps.storage, &current_bet)?;

    let get_fee_message = BankMsg::Send {
        to_address: owner.to_string(),
        amount: coins(fee.u128(), String::from("inj")),
    };

    Ok(Response::new()
        .add_message(get_fee_message)
        .add_attribute("action", "execute_end_bet")
        .add_attribute("end_price", price)
        .add_attribute("fee", fee))
}

pub fn execute_up_bet(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
) -> Result<Response, ContractError> {
    let sender = _info.sender;
    let info = INFO.load(deps.storage)?;
    let mut current_bet = CURRENTBET.load(deps.storage)?;

    if info.status != 1 {
        return Err(ContractError::CannotBet {});
    }
    if Uint64::from(_env.block.time.seconds()) > current_bet.endTime {
        return Err(ContractError::BetIDEnd {});
    }
    let denom = String::from("inj");
    let funds = _info
        .funds
        .iter()
        .find(|coin| coin.denom.eq(&denom))
        .unwrap();
    current_bet.totalUp += funds.amount;

    if funds.amount < Uint128::from(100000000000000000u128) {
        return Err(ContractError::SmallBet {});
    }

    CURRENTBET.save(deps.storage, &current_bet)?;

    if UP.may_load(deps.storage, (&sender, info.id))?.is_some() {
        UP.update(
            deps.storage,
            (&sender, info.id),
            |balance: Option<Uint128>| -> StdResult<_> {
                Ok(balance.unwrap_or_default() + funds.amount)
            },
        )?;
    } else {
        UP.save(deps.storage, (&sender, info.id), &funds.amount)?;
    }
    Ok(Response::new()
        .add_attribute("action", "up_bet")
        .add_attribute("value", funds.amount))
}

pub fn execute_down_bet(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
) -> Result<Response, ContractError> {
    let sender = _info.sender;
    let info = INFO.load(deps.storage)?;
    let mut current_bet = CURRENTBET.load(deps.storage)?;

    if info.status != 1 {
        return Err(ContractError::CannotBet {});
    }

    if Uint64::from(_env.block.time.seconds()) > current_bet.endTime {
        return Err(ContractError::BetIDEnd {});
    }

    let denom = String::from("inj");
    let funds = _info
        .funds
        .iter()
        .find(|coin| coin.denom.eq(&denom))
        .unwrap();
    current_bet.totalDown += funds.amount;

    if funds.amount < Uint128::from(100000000000000000u128) {
        return Err(ContractError::SmallBet {});
    }

    CURRENTBET.save(deps.storage, &current_bet)?;

    if DOWN.may_load(deps.storage, (&sender, info.id))?.is_some() {
        DOWN.update(
            deps.storage,
            (&sender, info.id),
            |balance: Option<Uint128>| -> StdResult<_> {
                Ok(balance.unwrap_or_default() + funds.amount)
            },
        )?;
    } else {
        DOWN.save(deps.storage, (&sender, info.id), &funds.amount)?;
    }
    Ok(Response::new()
        .add_attribute("action", "down_bet")
        .add_attribute("value", funds.amount))
}

pub fn execute_claim(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    bet_id: u64,
) -> Result<Response, ContractError> {
    let sender = _info.sender;
    let mut reward = Uint128::from(0u128);
    let mut claim_msg = BankMsg::Send {
        to_address: sender.to_string(),
        amount: coins(reward.u128(), String::from("inj")),
    };

    if CLAIMED.may_load(deps.storage, (&sender, bet_id))?.is_some() {
        let claimed = CLAIMED.load(deps.storage, (&sender, bet_id))?;
        if claimed {
            return Err(ContractError::Claimed {});
        }
    }

    if BETINFO.may_load(deps.storage, bet_id)?.is_some() {
        let bet_info = BETINFO.load(deps.storage, bet_id)?;
        if bet_info.startPrice <= bet_info.endPrice {
            if UP.may_load(deps.storage, (&sender, bet_id))?.is_some() {
                reward = bet_info.totalPrize * Uint128::from(1000000u128) / bet_info.upBet
                    * UP.load(deps.storage, (&sender, bet_id))?
                    / Uint128::from(1000000u128);

                claim_msg = BankMsg::Send {
                    to_address: sender.to_string(),
                    amount: coins(reward.u128(), String::from("inj")),
                };
            }
        } else if bet_info.startPrice > bet_info.endPrice {
            if DOWN.may_load(deps.storage, (&sender, bet_id))?.is_some() {
                reward = bet_info.totalPrize * Uint128::from(1000000u128) / bet_info.downBet
                    * DOWN.load(deps.storage, (&sender, bet_id))?
                    / Uint128::from(1000000u128);

                claim_msg = BankMsg::Send {
                    to_address: sender.to_string(),
                    amount: coins(reward.u128(), String::from("inj")),
                };
            }
        } else {
        }

        CLAIMED.save(deps.storage, (&sender, bet_id), &true)?;
    }

    Ok(Response::new()
        .add_message(claim_msg)
        .add_attribute("action", "claim")
        .add_attribute("value", reward)
        .add_attribute("addr", sender)
        .add_attribute("bet_id", bet_id.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::CurrentInfo { addr } => query_current_info(deps, addr),
        QueryMsg::BetInfo { bet_id } => query_bet_info(deps, bet_id),
        QueryMsg::UserReward { addr, bet_id } => query_user_reward(deps, addr, bet_id),
    }
}

pub fn query_current_info(deps: Deps, addr: Addr) -> StdResult<Binary> {
    let info = INFO.load(deps.storage)?;
    let current_bet = CURRENTBET.load(deps.storage)?;
    let mut upPosition = Uint128::from(0u128);
    let mut downPosition = Uint128::from(0u128);
    if UP.may_load(deps.storage, (&addr, info.id))?.is_some() {
        upPosition = UP.load(deps.storage, (&addr, info.id))?;
    }
    if DOWN.may_load(deps.storage, (&addr, info.id))?.is_some() {
        downPosition = DOWN.load(deps.storage, (&addr, info.id))?;
    }
    let resp = CurrentInfoResponse {
        id: info.id,
        status: info.status,
        totalUp: current_bet.totalUp,
        totalDown: current_bet.totalDown,
        startTime: current_bet.startTime,
        endTime: current_bet.endTime,
        startPrice: current_bet.startPrice,
        upPosition: upPosition,
        downPosition: downPosition,
    };

    to_binary(&resp)
}

pub fn query_bet_info(deps: Deps, bet_id: u64) -> StdResult<Binary> {
    let mut resp = BetInfo::default();
    if BETINFO.may_load(deps.storage, bet_id)?.is_some() {
        resp = BETINFO.load(deps.storage, bet_id)?;
    }

    to_binary(&resp)
}

pub fn query_user_reward(deps: Deps, addr: Addr, bet_id: u64) -> StdResult<Binary> {
    let mut resp = Uint128::from(0u128);
    if CLAIMED.may_load(deps.storage, (&addr, bet_id))?.is_some() {
        let claimed = CLAIMED.load(deps.storage, (&addr, bet_id))?;
        if claimed {
            return to_binary(&resp);
        }
    }

    if BETINFO.may_load(deps.storage, bet_id)?.is_some() {
        let bet_info = BETINFO.load(deps.storage, bet_id)?;
        if bet_info.startPrice <= bet_info.endPrice {
            if UP.may_load(deps.storage, (&addr, bet_id))?.is_some() {
                resp = bet_info.totalPrize * Uint128::from(1000000u128) / bet_info.upBet
                    * UP.load(deps.storage, (&addr, bet_id))?
                    / Uint128::from(1000000u128);
            }
        } else if bet_info.startPrice > bet_info.endPrice {
            if DOWN.may_load(deps.storage, (&addr, bet_id))?.is_some() {
                resp = bet_info.totalPrize * Uint128::from(1000000u128) / bet_info.downBet
                    * DOWN.load(deps.storage, (&addr, bet_id))?
                    / Uint128::from(1000000u128);
            }
        }
    }
    to_binary(&resp)
}
