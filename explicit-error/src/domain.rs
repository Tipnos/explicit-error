/// This trait must be implemented for type that converts to [Error](crate::Error)
/// Example of such implementation can be found in crates `explicit-error-http` or `explicit-error-exit` for `DomainError`.
pub trait Domain
where
    Self: std::error::Error + 'static + std::fmt::Debug + Into<crate::error::Error<Self>>,
{
    fn with_context(self, context: impl std::fmt::Display) -> Self;

    fn into_source(self) -> Option<Box<dyn std::error::Error>>;
}
