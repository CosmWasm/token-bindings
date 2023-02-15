mod msg;
mod querier;
mod query;
mod types;

pub use msg::{CreateDenomResponse, TokenFactoryMsg, TokenMsg};
pub use querier::TokenQuerier;
pub use query::{
    AdminResponse, DenomsByCreatorResponse, FullDenomResponse, MetadataResponse, ParamsResponse,
    TokenFactoryQuery, TokenQuery,
};
pub use types::{DenomUnit, Metadata, Params};

// This is a signal, such that any contract that imports these helpers will only run on
// blockchains that support token_factory feature
#[no_mangle]
extern "C" fn requires_token_factory() {}
