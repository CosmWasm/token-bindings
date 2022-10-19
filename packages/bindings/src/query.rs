use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{CustomQuery, QueryRequest};

#[cw_serde]
pub enum TokenFactoryQuery {
    // TODO: test how this works with cw_serde
    Token(TokenQuery),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum TokenQuery {
    /// Given a subdenom created by the address `creator_addr` via `OsmosisMsg::CreateDenom`,
    /// returns the full denom as used by `BankMsg::Send`.
    /// You may call `FullDenom { creator_addr: env.contract.address, subdenom }` to find the denom issued
    /// by the current contract.
    #[returns(FullDenomResponse)]
    FullDenom {
        creator_addr: String,
        subdenom: String,
    },
    // TODO: more about metadata? owner?
}

impl CustomQuery for TokenFactoryQuery {}

impl From<TokenQuery> for QueryRequest<TokenFactoryQuery> {
    fn from(query: TokenQuery) -> Self {
        QueryRequest::Custom(TokenFactoryQuery::Token(query))
    }
}

#[cw_serde]
pub struct FullDenomResponse {
    pub denom: String,
}
