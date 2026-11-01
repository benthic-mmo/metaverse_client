mod mock_server;

use std::{path::PathBuf, sync::Arc};

use benthic_protocol::session::rate_limiter::RateLimiter;
use metaverse_messages::udp::object::util::PBRMaterial;
use metaverse_objects::download_asset::download_material;

use mock_server::mock_asset_server;
use mock_server::{MATERIAL_1, MATERIAL_2};

#[actix_rt::test]
async fn test_download_mesh() {
    let endpoint = mock_asset_server().await;
    let client = awc::Client::default();
    let rate_limiter = Arc::new(RateLimiter::new(20, 20));

    let generated_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/generated");

    tokio::fs::create_dir_all(&generated_dir).await.unwrap();

    let material1_path = generated_dir.join("material1.json");

    download_material(
        MATERIAL_1,
        &endpoint,
        &material1_path,
        client.clone(),
        rate_limiter.clone(),
    )
    .await
    .unwrap();

    let material1_bytes = tokio::fs::read(&material1_path).await.unwrap();
    let material1_data = serde_llsd_benthic::auto_from_bytes(&material1_bytes).unwrap();
    let material1 = PBRMaterial::from_llsd(material1_data).unwrap();

    println!("{material1:#?}");

    let material2_path = generated_dir.join("material2.json");

    download_material(MATERIAL_2, &endpoint, &material2_path, client, rate_limiter)
        .await
        .unwrap();

    let material2_bytes = tokio::fs::read(&material2_path).await.unwrap();
    let material2_data = serde_llsd_benthic::auto_from_bytes(&material2_bytes).unwrap();
    let material2 = PBRMaterial::from_llsd(material2_data).unwrap();

    println!("{material2:#?}");

    assert!(!material1_bytes.is_empty());
    assert!(!material2_bytes.is_empty());
}
