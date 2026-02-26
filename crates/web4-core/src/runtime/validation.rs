use crate::error::{ErrorCode, RuntimeError, Web4Error};
use crate::model::ValidationTarget;
use crate::traits::Validator;
use jsonschema::validator_for;
use serde_json::json;

#[derive(Default)]
pub struct JsonSchemaValidator;

impl Validator for JsonSchemaValidator {
    fn validate(&self, target: &ValidationTarget) -> Result<(), Web4Error> {
        match target {
            ValidationTarget::NormalizedModel { .. } => Ok(()),
            ValidationTarget::JsonSchema { schema, payload } => {
                let compiled = validator_for(schema).map_err(|err| {
                    Web4Error::Runtime(RuntimeError::new(
                        ErrorCode::InvalidArgument,
                        format!("invalid schema: {err}"),
                        false,
                    ))
                })?;

                if compiled.is_valid(payload) {
                    Ok(())
                } else {
                    let details = compiled
                        .iter_errors(payload)
                        .map(|e| e.to_string())
                        .collect::<Vec<_>>();
                    Err(Web4Error::Runtime(
                        RuntimeError::new(
                            ErrorCode::InvalidArgument,
                            "schema validation failed",
                            false,
                        )
                        .with_details(json!({"errors": details})),
                    ))
                }
            }
        }
    }
}
