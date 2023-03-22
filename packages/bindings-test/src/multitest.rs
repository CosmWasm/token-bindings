use anyhow::{bail, Result as AnyResult};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use std::cmp::max;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use thiserror::Error;

use cosmwasm_std::testing::{MockApi, MockStorage};
use cosmwasm_std::{
    coins, to_binary, Addr, Api, Binary, BlockInfo, CustomQuery, Empty, Querier, QuerierResult,
    StdError, Storage,
};
use cw_multi_test::{
    App, AppResponse, BankKeeper, BankSudo, BasicAppBuilder, CosmosRouter, Module, WasmKeeper,
};
use cw_storage_plus::Map;

use token_bindings::{
    AdminResponse, CreateDenomResponse, DenomsByCreatorResponse, FullDenomResponse, Metadata,
    MetadataResponse, TokenFactoryMsg, TokenFactoryQuery, TokenMsg, TokenQuery,
};

use crate::error::ContractError;

pub struct TokenFactoryModule {}

/// How many seconds per block
/// (when we increment block.height, use this multiplier for block.time)
pub const BLOCK_TIME: u64 = 5;

// map denom to metadata
const METADATA: Map<&str, Metadata> = Map::new("metadata");

// map denom to admin
const ADMIN: Map<&str, Addr> = Map::new("admin");

// map creator to denoms
const DENOMS_BY_CREATOR: Map<&Addr, Vec<String>> = Map::new("denom");

impl TokenFactoryModule {
    fn build_denom(&self, creator: &Addr, subdenom: &str) -> Result<String, ContractError> {
        // Minimum validation checks on the full denom.
        // https://github.com/cosmos/cosmos-sdk/blob/2646b474c7beb0c93d4fafd395ef345f41afc251/types/coin.go#L706-L711
        // https://github.com/cosmos/cosmos-sdk/blob/2646b474c7beb0c93d4fafd395ef345f41afc251/types/coin.go#L677
        let full_denom = format!("factory/{}/{}", creator, subdenom);
        if full_denom.len() < 3
            || full_denom.len() > 128
            || creator.as_str().contains('/')
            || subdenom.len() > 44
            || creator.as_str().len() > 75
        {
            return Err(ContractError::InvalidFullDenom { full_denom });
        }
        Ok(full_denom)
    }
}

impl Module for TokenFactoryModule {
    type ExecT = TokenFactoryMsg;
    type QueryT = TokenFactoryQuery;
    type SudoT = Empty;

    // Builds a mock rust implementation of the expected osmosis functionality for testing
    fn execute<ExecC, QueryC>(
        &self,
        api: &dyn Api,
        storage: &mut dyn Storage,
        router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        block: &BlockInfo,
        sender: Addr,
        msg: TokenFactoryMsg,
    ) -> AnyResult<AppResponse>
    where
        ExecC: Debug + Clone + PartialEq + JsonSchema + DeserializeOwned + 'static,
        QueryC: CustomQuery + DeserializeOwned + 'static,
    {
        let TokenFactoryMsg::Token(msg) = msg;
        match msg {
            TokenMsg::CreateDenom { subdenom, metadata } => {
                let new_token_denom = self.build_denom(&sender, &subdenom)?;

                // errors if the denom was already created
                if ADMIN.may_load(storage, &new_token_denom)?.is_some() {
                    return Err(ContractError::TokenExists.into());
                }
                ADMIN.save(storage, &new_token_denom, &sender)?;

                // TODO: charge the creation fee (once params is supported)

                let mut denoms = DENOMS_BY_CREATOR
                    .may_load(storage, &sender)?
                    .unwrap_or_default();
                denoms.push(new_token_denom.clone());
                DENOMS_BY_CREATOR.save(storage, &sender, &denoms)?;

                // set metadata if provided
                if let Some(md) = metadata {
                    METADATA.save(storage, &new_token_denom, &md)?;
                }

                let data = Some(CreateDenomResponse { new_token_denom }.encode()?);
                Ok(AppResponse {
                    data,
                    events: vec![],
                })
            }
            TokenMsg::MintTokens {
                denom,
                amount,
                mint_to_address,
            } => {
                // ensure we are admin of this denom (and it exists)
                let admin = ADMIN
                    .may_load(storage, &denom)?
                    .ok_or(ContractError::TokenDoesntExist)?;
                if admin != sender {
                    return Err(ContractError::NotTokenAdmin.into());
                }
                let mint = BankSudo::Mint {
                    to_address: mint_to_address,
                    amount: coins(amount.u128(), &denom),
                };
                router.sudo(api, storage, block, mint.into())?;
                Ok(AppResponse::default())
            }
            TokenMsg::BurnTokens {
                denom: _,
                amount: _,
                burn_from_address: _,
            } => todo!(),
            TokenMsg::ForceTransfer {
                denom: _,
                amount: _,
                from_address: _,
                to_address: _,
            } => todo!(),
            TokenMsg::ChangeAdmin {
                denom,
                new_admin_address,
            } => {
                // ensure we are admin of this denom (and it exists)
                let admin = ADMIN
                    .may_load(storage, &denom)?
                    .ok_or(ContractError::TokenDoesntExist)?;
                if admin != sender {
                    return Err(ContractError::NotTokenAdmin.into());
                }
                // and new admin is valid
                let new_admin = api.addr_validate(&new_admin_address)?;
                ADMIN.save(storage, &denom, &new_admin)?;
                Ok(AppResponse::default())
            }
            TokenMsg::SetMetadata { denom, metadata } => {
                // ensure we are admin of this denom (and it exists)
                let admin = ADMIN
                    .may_load(storage, &denom)?
                    .ok_or(ContractError::TokenDoesntExist)?;
                if admin != sender {
                    return Err(ContractError::NotTokenAdmin.into());
                }
                // FIXME: add validation of metadata
                METADATA.save(storage, &denom, &metadata)?;
                Ok(AppResponse::default())
            }
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
        storage: &dyn Storage,
        _querier: &dyn Querier,
        _block: &BlockInfo,
        request: TokenFactoryQuery,
    ) -> anyhow::Result<Binary> {
        let TokenFactoryQuery::Token(query) = request;
        match query {
            TokenQuery::FullDenom {
                creator_addr,
                subdenom,
            } => {
                let contract = api.addr_validate(&creator_addr)?;
                let denom = self.build_denom(&contract, &subdenom)?;
                let res = FullDenomResponse { denom };
                Ok(to_binary(&res)?)
            }
            TokenQuery::Metadata { denom } => {
                let metadata = METADATA.may_load(storage, &denom)?;
                Ok(to_binary(&MetadataResponse { metadata })?)
            }
            TokenQuery::Admin { denom } => {
                let admin = ADMIN.load(storage, &denom)?.to_string();
                Ok(to_binary(&AdminResponse { admin })?)
            }
            TokenQuery::DenomsByCreator { creator } => {
                let creator = api.addr_validate(&creator)?;
                let denoms = DENOMS_BY_CREATOR
                    .may_load(storage, &creator)?
                    .unwrap_or_default();
                Ok(to_binary(&DenomsByCreatorResponse { denoms })?)
            }
            TokenQuery::Params {} => todo!(),
        }
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum TokenFactoryError {
    #[error("{0}")]
    Std(#[from] StdError),

    /// Remove this to let the compiler find all TODOs
    #[error("Not yet implemented (TODO)")]
    Unimplemented,
}

pub type TokenFactoryAppWrapped = App<
    BankKeeper,
    MockApi,
    MockStorage,
    TokenFactoryModule,
    WasmKeeper<TokenFactoryMsg, TokenFactoryQuery>,
>;

pub struct TokenFactoryApp(TokenFactoryAppWrapped);

impl Deref for TokenFactoryApp {
    type Target = TokenFactoryAppWrapped;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TokenFactoryApp {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Querier for TokenFactoryApp {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        self.0.raw_query(bin_request)
    }
}

impl Default for TokenFactoryApp {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFactoryApp {
    pub fn new() -> Self {
        Self(
            BasicAppBuilder::<TokenFactoryMsg, TokenFactoryQuery>::new_custom()
                .with_custom(TokenFactoryModule {})
                .build(|_router, _, _storage| {
                    // router.custom.set_owner(storage, &owner).unwrap();
                }),
        )
    }

    pub fn block_info(&self) -> BlockInfo {
        self.0.block_info()
    }

    /// This advances BlockInfo by given number of blocks.
    /// It does not do any callbacks, but keeps the ratio of seconds/block
    pub fn advance_blocks(&mut self, blocks: u64) {
        self.update_block(|block| {
            block.time = block.time.plus_seconds(BLOCK_TIME * blocks);
            block.height += blocks;
        });
    }

    /// This advances BlockInfo by given number of seconds.
    /// It does not do any callbacks, but keeps the ratio of seconds/block
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
    use cosmwasm_std::{Coin, Uint128};
    use cw_multi_test::Executor;

    #[test]
    fn mint_token() {
        let contract = Addr::unchecked("govner");
        let rcpt = Addr::unchecked("townies");
        let subdenom = "fundz";

        let mut app = TokenFactoryApp::new();

        // no tokens
        let start = app.wrap().query_all_balances(rcpt.as_str()).unwrap();
        assert_eq!(start, vec![]);

        // let's find the mapping
        let FullDenomResponse { denom } = app
            .wrap()
            .query(
                &TokenQuery::FullDenom {
                    creator_addr: contract.to_string(),
                    subdenom: subdenom.to_string(),
                }
                .into(),
            )
            .unwrap();
        assert_ne!(denom, subdenom);
        assert!(denom.len() > 10);

        // prepare to mint
        let amount = Uint128::new(1234567);
        let msg = TokenMsg::MintTokens {
            denom: denom.to_string(),
            amount,
            mint_to_address: rcpt.to_string(),
        };

        // fails to mint token before creating it
        let err = app
            .execute(contract.clone(), msg.clone().into())
            .unwrap_err();
        assert_eq!(
            err.downcast::<ContractError>().unwrap(),
            ContractError::TokenDoesntExist
        );

        // create the token now
        let create = TokenMsg::CreateDenom {
            subdenom: subdenom.to_string(),
            metadata: Some(Metadata {
                description: Some("Awesome token, get it now!".to_string()),
                denom_units: vec![],
                base: None,
                display: Some("FUNDZ".to_string()),
                name: Some("Fundz pays".to_string()),
                symbol: Some("FUNDZ".to_string()),
            }),
        };
        app.execute(contract.clone(), create.into()).unwrap();

        // now we can mint
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
