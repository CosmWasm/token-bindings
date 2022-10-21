use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use token_bindings::{
    AdminResponse, DenomsByCreatorResponse, FullDenomResponse, MetadataResponse, ParamsResponse,
    TokenFactoryMsg, TokenFactoryQuery,
};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(TokenFactoryMsg), &out_dir);
    export_schema(&schema_for!(TokenFactoryQuery), &out_dir);
    export_schema(&schema_for!(AdminResponse), &out_dir);
    export_schema(&schema_for!(DenomsByCreatorResponse), &out_dir);
    export_schema(&schema_for!(FullDenomResponse), &out_dir);
    export_schema(&schema_for!(MetadataResponse), &out_dir);
    export_schema(&schema_for!(ParamsResponse), &out_dir);
}
