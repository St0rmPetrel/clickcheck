use crate::analyzer;
use crate::client;
use crate::model;
use crate::output;
use tokio::sync::mpsc;

pub async fn handle_top_queries(
    client: client::Client,
    req: model::TopQueryRequest,
) -> Result<(), String> {
    let (tx, rx) = mpsc::channel(128);
    let analyzer_task = analyzer::top(rx, req.limit, req.sort_by);
    let stream_task = client.stream_query_logs(tx);

    let (stream_result, top_queries) = tokio::join!(stream_task, analyzer_task);

    stream_result.map_err(|e| format!("Stream error: {e}"))?;

    output::print_top(&top_queries, req.out);

    Ok(())
}
