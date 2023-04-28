use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Only start a pending ID")]
    NotPendingID {},

    #[error("Only close an opening ID")]
    NotOpeningID {},

    #[error("Only bet with more than 0.1 INJ")]
    SmallBet {},

    #[error("Bet ID not open yet")]
    CannotBet {},

    #[error("Bet ID doesn't end")]
    BetIDNotEnd {},

    #[error("Bet id ended")]
    BetIDEnd {},

    #[error("Reward Claimed")]
    Claimed {},    
}
