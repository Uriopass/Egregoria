use crate::{CompanyKind, MultiError, Prototypes};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("{0}: only factories can have trucks")]
    WrongTrucks(String),
    #[error("{0}: factories must have trucks")]
    ZeroTrucks(String),
    #[error("{0}.{1}: referenced prototype not found")]
    ReferencedProtoNotFound(String, &'static str),

    #[error("{0}.{1}: {2}")]
    InvalidField(String, &'static str, String),
}

pub(crate) fn validate(proto: &Prototypes) -> Result<(), MultiError<ValidationError>> {
    let mut errors = vec![];

    for comp in proto.companies.values() {
        if comp.n_trucks > 0 && comp.kind != CompanyKind::Factory {
            errors.push(ValidationError::WrongTrucks(comp.name.clone()));
        }

        if comp.n_trucks == 0 && comp.kind == CompanyKind::Factory {
            errors.push(ValidationError::ZeroTrucks(comp.name.clone()));
        }

        for item in &comp.recipe.consumption {
            if !proto.items.contains_key(&item.id) {
                errors.push(ValidationError::ReferencedProtoNotFound(
                    comp.name.clone(),
                    "consumption",
                ));
            }
        }

        for item in &comp.recipe.production {
            if !proto.items.contains_key(&item.id) {
                errors.push(ValidationError::ReferencedProtoNotFound(
                    comp.name.clone(),
                    "production",
                ));
            }
        }

        if comp.recipe.power_usage.0 < 0 {
            errors.push(ValidationError::InvalidField(
                comp.name.clone(),
                "power_usage",
                "must not be negative".to_string(),
            ));
        }

        if comp.recipe.power_generation.0 < 0 {
            errors.push(ValidationError::InvalidField(
                comp.name.clone(),
                "power_generation",
                "must not be negative".to_string(),
            ));
        }
    }

    if !errors.is_empty() {
        return Err(MultiError(errors));
    }
    Ok(())
}
