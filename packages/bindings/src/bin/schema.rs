use cosmwasm_schema::write_api;

use osmo_bindings::{TokenFactoryMsg, TokenFactoryQuery};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: TokenFactoryMsg,
        query: TokenFactoryQuery,
    }
}
