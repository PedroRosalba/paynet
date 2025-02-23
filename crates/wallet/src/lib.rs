mod db;
mod db;
use anyhow::Result;
use node::{
    MeltRequest, MeltResponse, MintQuoteRequest, MintQuoteResponse, MintQuoteState, MintRequest,
    QuoteStateRequest, SwapRequest, SwapResponse,
};
use node::{MintResponse, NodeClient};
use nuts::nut00::{BlindedMessage, Proof};
use rusqlite::Connection;
use tonic::transport::Channel;

pub use db::create_tables;

pub fn convert_inputs(inputs: &[Proof]) -> Vec<node::Proof> {
    inputs
        .iter()
        .map(|p| node::Proof {
            amount: p.amount.into(),
            keyset_id: p.keyset_id.to_bytes().to_vec(),
            secret: p.secret.to_string(),
            unblind_signature: p.c.to_bytes().to_vec(),
        })
        .collect()
}

pub fn convert_outputs(outputs: &[BlindedMessage]) -> Vec<node::BlindedMessage> {
    outputs
        .iter()
        .map(|o| node::BlindedMessage {
            amount: o.amount.into(),
            keyset_id: o.keyset_id.to_bytes().to_vec(),
            blinded_secret: o.blinded_secret.to_bytes().to_vec(),
        })
        .collect()
}

pub async fn create_mint_quote(
    db_conn: &mut Connection,
    node_client: &mut NodeClient<Channel>,
    node_client: &mut NodeClient<Channel>,
    method: String,
    amount: u64,
    unit: String,
) -> Result<MintQuoteResponse> {
    let response = node_client
        .mint_quote(MintQuoteRequest {
            method: method.clone(),
            amount,
            unit: unit.clone(),
            description: None,
        })
        .await?
        .into_inner();

    db::store_mint_quote(db_conn, method, amount, unit, &response)?;

    Ok(response)
}

pub async fn get_mint_quote_state(
    db_conn: &mut Connection,
    node_client: &mut NodeClient<Channel>,
    method: String,
    quote_id: String,
) -> Result<MintQuoteState> {
    let response = node_client
        .mint_quote_state(QuoteStateRequest {
            method,
            quote: quote_id,
        })
        .await?
        .into_inner();

    db::set_mint_quote_state(db_conn, response.quote, response.state)?;

    Ok(MintQuoteState::try_from(response.state)?)
}

pub async fn mint(
    node_client: &mut NodeClient<Channel>,
    method: String,
    quote: String,
    outputs: &[BlindedMessage],
) -> Result<MintResponse> {
    let req = MintRequest {
        method,
        quote,
        outputs: convert_outputs(outputs),
    };

    let resp = node_client.mint(req).await?;

    Ok(resp.into_inner())
}

pub async fn create_melt(
    node_client: &mut NodeClient<Channel>,
    method: String,
    unit: String,
    request: String,
    inputs: &[Proof],
) -> Result<MeltResponse> {
    let req = MeltRequest {
        method,
        unit,
        request,
        inputs: convert_inputs(inputs),
    };
    let resp = node_client.melt(req).await?;

    Ok(resp.into_inner())
}

pub async fn swap(
    node_client: &mut NodeClient<Channel>,
    inputs: &[Proof],
    outputs: &[BlindedMessage],
) -> Result<SwapResponse> {
    let req = SwapRequest {
        inputs: convert_inputs(inputs),
        outputs: convert_outputs(outputs),
    };

    let resp = node_client.swap(req).await?;

    Ok(resp.into_inner())
}
