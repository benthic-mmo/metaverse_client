use crate::error::SimTimeError;
use awc::Client;
use metaverse_messages::http::environment_data::DayCycle;

/// Retrieve environment time from the server's capability url
pub async fn fetch_environment_time(
    capability_url: &String,
    client: Client,
) -> Result<(), SimTimeError> {
    let response = client
        .get(capability_url.to_string())
        .send()
        .await?
        .body()
        .await?;
    let _day_cycle = DayCycle::from_bytes(&response)?;
    Ok(())
}
