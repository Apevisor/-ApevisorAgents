use flutter_rust_bridge::frb;

#[frb(opaque)]
pub struct XybridSdkClient;

impl XybridSdkClient {
    #[frb(sync)]
    pub fn init_sdk_cache_dir(cache_dir: String) {
        xybrid_sdk::init_sdk_cache_dir(cache_dir);
    }

    #[frb(sync)]
    pub fn set_api_key(api_key: &str) {
        xybrid_sdk::set_api_key(api_key);
    }

    /// Check if a model is cached locally (extracted and ready to use).
    ///
    /// This is a pure filesystem check — no network access required.
    /// Returns `true` if the model has been downloaded and extracted
    /// at `~/.xybrid/cache/extracted/{model_id}/model_metadata.json`.
    #[frb(sync)]
    pub fn is_model_cached(model_id: &str) -> bool {
        if let Ok(client) = xybrid_sdk::RegistryClient::from_env() {
            return client.is_extracted(model_id);
        }
        false
    }
}
