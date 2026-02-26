use crate::error::{ErrorCode, RuntimeError, Web4Error};
use crate::model::ConsentRequest;
use crate::traits::ConsentEngine;
use async_trait::async_trait;

#[derive(Default)]
pub struct DefaultConsentEngine;

#[async_trait]
impl ConsentEngine for DefaultConsentEngine {
    async fn check(&self, request: &ConsentRequest) -> Result<(), Web4Error> {
        let effects_high = matches!(
            request.effects.as_str(),
            "write" | "control" | "financial" | "unknown"
        );

        match request.mode.as_str() {
            "deny" => Err(Web4Error::Runtime(RuntimeError::new(
                ErrorCode::Forbidden,
                "service consent mode is deny",
                false,
            ))),
            "interactive" => {
                if request.interactive_approved {
                    Ok(())
                } else {
                    Err(Web4Error::Runtime(RuntimeError::new(
                        ErrorCode::ConsentRequired,
                        "interactive consent required",
                        false,
                    )))
                }
            }
            "capability" => {
                if request
                    .capability_token
                    .as_deref()
                    .unwrap_or_default()
                    .is_empty()
                {
                    Err(Web4Error::Runtime(RuntimeError::new(
                        ErrorCode::Unauthorized,
                        "capability token required",
                        false,
                    )))
                } else {
                    Ok(())
                }
            }
            "open" => {
                if effects_high {
                    Err(Web4Error::Runtime(RuntimeError::new(
                        ErrorCode::EffectsBlocked,
                        "effects level requires consent",
                        false,
                    )))
                } else {
                    Ok(())
                }
            }
            _ => Err(Web4Error::Runtime(RuntimeError::new(
                ErrorCode::InvalidArgument,
                "unsupported consent mode",
                false,
            ))),
        }
    }
}
