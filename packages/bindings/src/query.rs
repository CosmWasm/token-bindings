use crate::types::{Metadata, Params};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{CustomQuery, QueryRequest};

#[cw_serde]
pub enum TokenFactoryQuery {
    // Note: embded enums don't work with QueryResponses currently
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
    /// Returns the metadata set for this denom, if present. May return None.
    /// This will also return metadata for native tokens created outside
    /// of the token factory (like staking tokens)
    #[returns(MetadataResponse)]
    Metadata { denom: String },
    /// Returns info on admin of the denom, only if created/managed via token factory.
    /// Errors if denom doesn't exist or was created by another module.
    #[returns(AdminResponse)]
    Admin { denom: String },
    /// List all denoms that were created by the given creator.
    /// This does not imply all tokens currently managed by the creator.
    /// (Admin may have changed)
    #[returns(DenomsByCreatorResponse)]
    DenomsByCreator { creator: String },
    /// Returns configuration params for TokenFactory modules
    #[returns(ParamsResponse)]
    Params {},
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

#[cw_serde]
pub struct MetadataResponse {
    /// Empty if this was never set for the given denom
    pub metadata: Option<Metadata>,
}

#[cw_serde]
pub struct AdminResponse {
    pub admin: String,
}

#[cw_serde]
pub struct DenomsByCreatorResponse {
    pub denoms: Vec<String>,
}

#[cw_serde]
pub struct ParamsResponse {
    pub params: Params,
}
