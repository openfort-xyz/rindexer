use alloy::transports::{
    layers::{RateLimitRetryPolicy, RetryPolicy},
    RpcError::Transport,
    TransportError,
    TransportErrorKind::HttpError,
};

const ERPC_ERROR_CODE_UNAUTHORIZED: &str = "-32601";
const HTTP_STATUS_CODE_5XX_START: u16 = 500;
const HTTP_STATUS_CODE_5XX_END: u16 = 599;

/// [ErpcRetryPolicy] is mostly equivalent to [RateLimitRetryPolicy].
/// We've found some RPC providers don't return the expected 4XX when
/// sent a request they don't support, rather they return 5XX.
/// Rindexer sends `zks_L1ChainId` to every provider to see if they
/// support it, which caused an endless loop in the initialization.
/// This policy prevents retrying on 5XX errors that also contain
/// the error code added by ERPC for some unsupported operations..
#[derive(Debug, Clone)]
pub struct ErpcRetryPolicy {
    inner: RateLimitRetryPolicy,
}

impl ErpcRetryPolicy {
    pub fn default() -> Self {
        Self { inner: RateLimitRetryPolicy::default() }
    }
}

impl RetryPolicy for ErpcRetryPolicy {
    fn should_retry(&self, error: &TransportError) -> bool {
        if let Some(status) = get_status(error) {
            if is_server_error(status) && error.to_string().contains(ERPC_ERROR_CODE_UNAUTHORIZED) {
                return false;
            }
        }

        self.inner.should_retry(error)
    }

    fn backoff_hint(&self, error: &TransportError) -> Option<std::time::Duration> {
        self.inner.backoff_hint(error)
    }
}

#[inline]
fn is_server_error(status_code: u16) -> bool {
    HTTP_STATUS_CODE_5XX_START <= status_code && status_code <= HTTP_STATUS_CODE_5XX_END
}

fn get_status(error: &TransportError) -> Option<u16> {
    match error {
        Transport(transport_error) => match transport_error {
            HttpError(http_error) => Some(http_error.status),
            _ => None,
        },
        _ => None,
    }
}
