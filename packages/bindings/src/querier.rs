use cosmwasm_std::{QuerierWrapper, StdResult};

use crate::query::{FullDenomResponse, TokenFactoryQuery, TokenQuery};

/// This is a helper wrapper to easily use our custom queries
pub struct TokenQuerier<'a> {
    querier: &'a QuerierWrapper<'a, TokenFactoryQuery>,
}

impl<'a> TokenQuerier<'a> {
    pub fn new(querier: &'a QuerierWrapper<TokenFactoryQuery>) -> Self {
        TokenQuerier { querier }
    }

    pub fn full_denom(
        &self,
        creator_addr: String,
        subdenom: String,
    ) -> StdResult<FullDenomResponse> {
        let full_denom_query = TokenQuery::FullDenom {
            creator_addr,
            subdenom,
        };
        self.querier.query(&full_denom_query.into())
    }
}
