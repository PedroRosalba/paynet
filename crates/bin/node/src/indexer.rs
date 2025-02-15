use futures::TryStreamExt;
use invoice_payment_indexer::ApibaraIndexerService;
use starknet_types_core::felt::Felt;

use crate::errors::{InitializationError, ServiceError};

pub async fn spawn_indexer_task(
    apibara_token: String,
    strk_token_address: Felt,
    recipient_address: Felt,
) -> Result<ApibaraIndexerService, InitializationError> {
    let conn = rusqlite::Connection::open_in_memory().map_err(InitializationError::OpenSqlite)?;

    let service = invoice_payment_indexer::ApibaraIndexerService::init(
        conn,
        apibara_token,
        vec![(recipient_address, strk_token_address)],
    )
    .await
    .map_err(InitializationError::InitIndexer)?;

    Ok(service)
}

pub async fn listen_to_indexer(
    mut indexer_service: ApibaraIndexerService,
) -> Result<(), ServiceError> {
    while indexer_service
        .try_next()
        .await
        .map_err(ServiceError::Indexer)?
        .is_some()
    {
        // Do nothing more
        // the indexer is already writing the events in db
    }

    Ok(())
}
