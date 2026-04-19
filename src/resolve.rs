use crate::cache::{Cache, FileCache};
use crate::client::{ClientError, FetchResult, PageClient};

pub struct ResolveOutput {
    pub fetch_result: FetchResult,
    pub from_cache: bool,
}

pub async fn resolve_page(
    url: &str,
    client: &PageClient,
    no_cache: bool,
    refresh: bool,
) -> Result<ResolveOutput, ClientError> {
    let cache = FileCache::new(crate::cache::CacheConfig {
        enabled: !no_cache,
        refresh,
        ..Default::default()
    });
    cache.init().map_err(|e| ClientError::Request {
        url: url.to_string(),
        reason: e.to_string(),
    })?;

    let cache_key =
        cache
            .key_for_final_url(url)
            .map_err(|e| ClientError::Request {
                url: url.to_string(),
                reason: e.to_string(),
            })?;

    if !no_cache && !cache.should_refresh() {
        if let Some(cached) =
            cache.load(&cache_key).map_err(|e| ClientError::Request {
                url: url.to_string(),
                reason: e.to_string(),
            })?
        {
            return Ok(ResolveOutput {
                fetch_result: FetchResult {
                    input_url: cached.fetch.input_url,
                    final_url: cached.fetch.final_url,
                    status: cached.fetch.status,
                    headers: cached.headers,
                    body: cached.html,
                    duration_ms: 0,
                },
                from_cache: true,
            });
        }
    }

    let fetch_result = client.fetch(url).await?;

    if !no_cache {
        cache.store(fetch_result.to_cached_page()).map_err(|e| {
            ClientError::Request {
                url: url.to_string(),
                reason: e.to_string(),
            }
        })?;
    }

    Ok(ResolveOutput {
        fetch_result,
        from_cache: false,
    })
}
