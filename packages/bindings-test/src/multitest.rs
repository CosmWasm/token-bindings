use anyhow::{bail, Result as AnyResult};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
// use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use thiserror::Error;

use cosmwasm_std::testing::{MockApi, MockStorage};
use cosmwasm_std::{
    to_binary, Addr, Api, Binary, BlockInfo, Coin, CustomQuery, Empty, Querier, QuerierResult,
    StdError, Storage,
};
use cw_multi_test::{
    App, AppResponse, BankKeeper, BankSudo, BasicAppBuilder, CosmosRouter, Module, WasmKeeper,
};
// use cw_storage_plus::{Item, Map};

use osmo_bindings::{FullDenomResponse, OsmosisMsg, OsmosisQuery};

pub struct OsmosisModule {}

/// How many seconds per block
/// (when we increment block.height, use this multiplier for block.time)
pub const BLOCK_TIME: u64 = 5;

impl OsmosisModule {
    fn build_denom(&self, contract: &Addr, subdenom: &str) -> String {
        // TODO: validation assertion on subdenom
        format!("cw/{}/{}", contract, subdenom)
    }
}

impl Module for OsmosisModule {
    type ExecT = OsmosisMsg;
    type QueryT = OsmosisQuery;
    type SudoT = Empty;

    fn execute<ExecC, QueryC>(
        &self,
        api: &dyn Api,
        storage: &mut dyn Storage,
        router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        block: &BlockInfo,
        sender: Addr,
        msg: OsmosisMsg,
    ) -> AnyResult<AppResponse>
    where
        ExecC: Debug + Clone + PartialEq + JsonSchema + DeserializeOwned + 'static,
        QueryC: CustomQuery + DeserializeOwned + 'static,
    {
        match msg {
            OsmosisMsg::MintTokens {
                subdenom,
                amount,
                recipient,
            } => {
                let denom = self.build_denom(&sender, &subdenom);
                let mint = BankSudo::Mint {
                    to_address: recipient,
                    amount: vec![Coin { denom, amount }],
                };
                router.sudo(api, storage, block, mint.into())
            }
            // TODO
            _ => unimplemented!(),
        }
    }

    fn sudo<ExecC, QueryC>(
        &self,
        _api: &dyn Api,
        _storage: &mut dyn Storage,
        _router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        _block: &BlockInfo,
        _msg: Self::SudoT,
    ) -> AnyResult<AppResponse>
    where
        ExecC: Debug + Clone + PartialEq + JsonSchema + DeserializeOwned + 'static,
        QueryC: CustomQuery + DeserializeOwned + 'static,
    {
        bail!("sudo not implemented for OsmosisModule")
    }

    fn query(
        &self,
        api: &dyn Api,
        _storage: &dyn Storage,
        _querier: &dyn Querier,
        _block: &BlockInfo,
        request: OsmosisQuery,
    ) -> anyhow::Result<Binary> {
        match request {
            OsmosisQuery::FullDenom { contract, subdenom } => {
                let contract = api.addr_validate(&contract)?;
                let denom = self.build_denom(&contract, &subdenom);
                let res = FullDenomResponse { denom };
                Ok(to_binary(&res)?)
            }
            // TODO
            _ => unimplemented!(),
        }
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum OsmosisError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),
}

pub type OsmosisAppWrapped =
    App<BankKeeper, MockApi, MockStorage, OsmosisModule, WasmKeeper<OsmosisMsg, OsmosisQuery>>;

pub struct OsmosisApp(OsmosisAppWrapped);

impl Deref for OsmosisApp {
    type Target = OsmosisAppWrapped;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for OsmosisApp {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Querier for OsmosisApp {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        self.0.raw_query(bin_request)
    }
}

impl Default for OsmosisApp {
    fn default() -> Self {
        Self::new()
    }
}

impl OsmosisApp {
    pub fn new() -> Self {
        Self(
            BasicAppBuilder::<OsmosisMsg, OsmosisQuery>::new_custom()
                .with_custom(OsmosisModule {})
                .build(|_router, _, _storage| {
                    // router.custom.set_owner(storage, &owner).unwrap();
                }),
        )
    }

    pub fn block_info(&self) -> BlockInfo {
        self.0.block_info()
    }

    /// This advances BlockInfo by given number of blocks.
    /// It does not do any callbacks, but keeps the ratio of seconds/blokc
    pub fn advance_blocks(&mut self, blocks: u64) {
        self.update_block(|block| {
            block.time = block.time.plus_seconds(BLOCK_TIME * blocks);
            block.height += blocks;
        });
    }

    /// This advances BlockInfo by given number of seconds.
    /// It does not do any callbacks, but keeps the ratio of seconds/blokc
    pub fn advance_seconds(&mut self, seconds: u64) {
        self.update_block(|block| {
            block.time = block.time.plus_seconds(seconds);
            block.height += max(1, seconds / BLOCK_TIME);
        });
    }

    /// Simple iterator when you don't care too much about the details and just want to
    /// simulate forward motion.
    pub fn next_block(&mut self) {
        self.advance_blocks(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::Uint128;
    use cw_multi_test::Executor;

    #[test]
    fn mint_token() {
        let contract = Addr::unchecked("govner");
        let rcpt = Addr::unchecked("townies");
        let subdenom = "fundz";

        let mut app = OsmosisApp::new();

        // no tokens
        let start = app.wrap().query_all_balances(rcpt.as_str()).unwrap();
        assert_eq!(start, vec![]);

        // let's find the mapping
        let FullDenomResponse { denom } = app
            .wrap()
            .query(
                &OsmosisQuery::FullDenom {
                    contract: contract.to_string(),
                    subdenom: subdenom.to_string(),
                }
                .into(),
            )
            .unwrap();
        assert_ne!(denom, subdenom);
        assert!(denom.len() > 10);

        // prepare to mint
        let amount = Uint128::new(1234567);
        let msg = OsmosisMsg::MintTokens {
            subdenom: subdenom.to_string(),
            amount,
            recipient: rcpt.to_string(),
        };

        // simulate contract calling
        app.execute(contract, msg.into()).unwrap();

        // we got tokens!
        let end = app.wrap().query_balance(rcpt.as_str(), &denom).unwrap();
        let expected = Coin { denom, amount };
        assert_eq!(end, expected);

        // but no minting of unprefixed version
        let empty = app.wrap().query_balance(rcpt.as_str(), subdenom).unwrap();
        assert_eq!(empty.amount, Uint128::zero());
    }
}