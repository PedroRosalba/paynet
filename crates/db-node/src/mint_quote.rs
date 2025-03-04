use nuts::{
    Amount,
    nut04::{MintQuoteResponse, MintQuoteState},
    traits::Unit,
};
use sqlx::{PgConnection, types::time::OffsetDateTime};
use uuid::Uuid;

use crate::Error;

pub async fn insert_new<U: Unit>(
    conn: &mut PgConnection,
    quote_id: Uuid,
    unit: U,
    amount: Amount,
    request: &str,
    expiry: u64,
) -> Result<(), Error> {
    let expiry: i64 = expiry
        .try_into()
        .map_err(|_| Error::RuntimeToDbConversion)?;
    let expiry =
        OffsetDateTime::from_unix_timestamp(expiry).map_err(|_| Error::RuntimeToDbConversion)?;

    sqlx::query!(
        r#"INSERT INTO mint_quote (id, unit, amount, request, expiry, state) VALUES ($1, $2, $3, $4, $5, 'UNPAID')"#,
        quote_id,
        &unit.to_string(),
        amount.into_i64_repr(),
        request,
        expiry,
    ).execute(conn).await?;

    Ok(())
}

pub async fn build_response_from_db(
    conn: &mut PgConnection,
    quote_id: Uuid,
) -> Result<MintQuoteResponse<Uuid>, Error> {
    let record = sqlx::query!(
        r#"SELECT request, state as "state: MintQuoteState", expiry FROM mint_quote where id = $1"#,
        quote_id
    )
    .fetch_one(conn)
    .await?;

    let expiry = record
        .expiry
        .unix_timestamp()
        .try_into()
        .map_err(|_| Error::DbToRuntimeConversion)?;

    Ok(MintQuoteResponse {
        quote: quote_id,
        request: record.request,
        state: record.state,
        expiry,
    })
}

pub async fn get_amount_and_state(
    conn: &mut PgConnection,
    quote_id: Uuid,
) -> Result<(Amount, MintQuoteState), Error> {
    let record = sqlx::query!(
        r#"SELECT amount, state as "state: MintQuoteState" FROM mint_quote where id = $1"#,
        quote_id
    )
    .fetch_one(conn)
    .await?;

    let amount = Amount::from_i64_repr(record.amount);

    Ok((amount, record.state))
}

pub async fn set_state(
    conn: &mut PgConnection,
    quote_id: Uuid,
    state: MintQuoteState,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            UPDATE mint_quote
            SET state = $2
            WHERE id = $1
        "#,
        quote_id,
        state as MintQuoteState
    )
    .execute(conn)
    .await?;

    Ok(())
}
