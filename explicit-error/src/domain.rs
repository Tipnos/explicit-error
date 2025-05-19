pub trait Domain
where
    Self: std::error::Error + 'static + std::fmt::Debug + Into<crate::Error<Self>>,
{
    fn with_context(self, context: impl std::fmt::Display) -> Self;

    fn into_source(self) -> Option<Box<dyn std::error::Error>>;
}
