use std::env;
use tonic::transport::Channel;

use micro_traffic_sim::pb;
use micro_traffic_sim::pb::service_client::ServiceClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Server address (override with MT_SIM_ADDR, e.g. http://127.0.0.1:50051)
    let raw = env::var("MT_SIM_ADDR").unwrap_or_else(|_| "127.0.0.1:50051".to_string());
    // Ensure scheme for grpc endpoint
    let addr = if raw.starts_with("http://") || raw.starts_with("https://") {
        raw
    } else {
        format!("http://{raw}")
    };

    // Connect
    let channel = Channel::from_shared(addr.clone())?
        .connect()
        .await?;
    let mut client = ServiceClient::new(channel);

    // Create session (srid: 0 → Euclidean, 4326 → WGS84)
    let req = pb::SessionReq { srid: 0 };
    let resp = client.new_session(req).await?.into_inner();

    let sid = resp
        .id
        .as_ref()
        .map(|x| x.value.clone())
        .ok_or("server returned empty session id")?;

    println!("New session created:");
    println!("  code: {} text: {}", resp.code, resp.text);
    println!("  id:   {}", sid);

    // Verify with info_session
    let info = client
        .info_session(pb::UuiDv4 { value: sid.clone() })
        .await?
        .into_inner();

    println!("Info session:");
    println!("  code: {} text: {}", info.code, info.text);
    if let Some(data) = info.data {
        if let Some(id) = data.id {
            println!("  id:   {}", id.value);
        }
    }

    Ok(())
}