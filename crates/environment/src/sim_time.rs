use metaverse_messages::http::environment_data::DayCycle;

use crate::error::SimTimeError;

pub async fn fetch_environment_time(capability_url: &Option<String>) -> Result<(), SimTimeError> {
    let url = capability_url
        .clone()
        .ok_or(SimTimeError::CapNotPresent {})?;
    let client = awc::Client::default();
    let response = client.get(url.to_string()).send().await?.body().await?;
    let _day_cycle = DayCycle::from_bytes(&response)?;
    Ok(())
}
