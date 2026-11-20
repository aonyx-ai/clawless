//! Assertions that a type lacks an auto trait

/// Makes a type ambiguous when it is `Send`, so that a use of it compiles only for a type that
/// is not
pub(crate) trait AmbiguousIfSend<Marker> {
    /// Compiles only when the implementing type is not `Send`
    fn compiles_only_without_send() {}
}

impl<T: ?Sized> AmbiguousIfSend<()> for T {}

impl<T: ?Sized + Send> AmbiguousIfSend<u8> for T {}

/// Makes a type ambiguous when it is `Sync`, so that a use of it compiles only for a type that
/// is not
pub(crate) trait AmbiguousIfSync<Marker> {
    /// Compiles only when the implementing type is not `Sync`
    fn compiles_only_without_sync() {}
}

impl<T: ?Sized> AmbiguousIfSync<()> for T {}

impl<T: ?Sized + Sync> AmbiguousIfSync<u8> for T {}
