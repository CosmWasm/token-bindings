mod metadata;
mod msg;
mod querier;
mod query;

pub use msg::{TokenFactoryMsg, TokenMsg};
pub use querier::TokenQuerier;
pub use query::{FullDenomResponse, TokenFactoryQuery, TokenQuery};

// This is a signal, such that any contract that imports these helpers will only run on
// blockchains that support token_factory feature
#[no_mangle]
extern "C" fn requires_token_factory() {}
