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

    #[error("Bet id not open yet")]
    CannotBet {},
}
