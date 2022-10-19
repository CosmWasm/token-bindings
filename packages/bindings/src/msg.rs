use cosmwasm_schema::cw_serde;
use cosmwasm_std::{CosmosMsg, CustomMsg, Uint128};

/// A top-level Custom message for the token factory.
/// It is embedded like this to easily allow adding other variants that are custom
/// to your chain, or other "standardized" extensions along side it.
#[cw_serde]
pub enum TokenFactoryMsg {
    Token(TokenMsg),
}

/// Special messages to be supported by any chain that supports token_factory
#[cw_serde]
pub enum TokenMsg {
    /// CreateDenom creates a new factory denom, of denomination:
    /// factory/{creating contract bech32 address}/{Subdenom}
    /// Subdenom can be of length at most 44 characters, in [0-9a-zA-Z./]
    /// Empty subdenoms are valid.
    /// The (creating contract address, subdenom) pair must be unique.
    /// The created denom's admin is the creating contract address,
    /// but this admin can be changed using the UpdateAdmin binding.
    CreateDenom { subdenom: String },
    /// ChangeAdmin changes the admin for a factory denom.
    /// Can only be called by the current contract admin.
    /// If the NewAdminAddress is empty, the denom will have no admin.
    ChangeAdmin {
        denom: String,
        new_admin_address: String,
    },
    /// Contracts can mint native tokens for an existing factory denom
    /// that they are the admin of.
    MintTokens {
        denom: String,
        amount: Uint128,
        mint_to_address: String,
    },
    /// Contracts can burn native tokens for an existing factory denom
    /// that they are the admin of.
    /// Currently, the burn from address must be the admin contract.
    BurnTokens {
        denom: String,
        amount: Uint128,
        burn_from_address: String,
    },
    // TODO: consider more meta-data extensions
}

impl TokenMsg {
    pub fn mint_contract_tokens(denom: String, amount: Uint128, mint_to_address: String) -> Self {
        TokenMsg::MintTokens {
            denom,
            amount,
            mint_to_address,
        }
    }

    pub fn burn_contract_tokens(
        denom: String,
        amount: Uint128,
        _burn_from_address: String,
    ) -> Self {
        TokenMsg::BurnTokens {
            denom,
            amount,
            burn_from_address: "".to_string(), // burn_from_address is currently disabled.
        }
    }
}

impl From<TokenMsg> for CosmosMsg<TokenFactoryMsg> {
    fn from(msg: TokenMsg) -> CosmosMsg<TokenFactoryMsg> {
        CosmosMsg::Custom(TokenFactoryMsg::Token(msg))
    }
}

impl CustomMsg for TokenFactoryMsg {}
