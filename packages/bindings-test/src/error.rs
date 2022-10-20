use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Invalid full denom '{full_denom}'")]
    InvalidFullDenom { full_denom: String },

    #[error("Not admin of token, cannot perfrom action")]
    NotTokenAdmin,

    #[error("Token denom already exists, cannot create again")]
    TokenExists,

    #[error("Token denom was never created")]
    TokenDoesntExist,
}
