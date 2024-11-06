/// Implements AsHandle and Indexable traits for a given type
///
/// This macro generates implementations for:
/// - AsHandle trait: provides access to the handle field
/// - Indexable trait: provides access to the handle's index
///
/// Usage:
/// ```
/// impl_handle!(MyStruct);
/// // Or with generics:
/// impl_handle!(MyStruct<T>);
/// impl_handle!(MyStruct<'a, T: Clone>);
/// ```
#[macro_export]
macro_rules! impl_handle {
    ($name:ident $(<$($lt:tt$(:$clt:tt$(+$dlt:tt)*)?),+ >)?) => {
        impl $(<$($lt$(:$clt$(+$dlt)*)?),+>)? $crate::common::handle::AsHandle for $name $(<$($lt),+>)?
        {
            type Output = $crate::common::handle::Handle<$name $(<$($lt),+ >)?>;
            #[inline(always)]
            fn as_handle(&self) -> Self::Output {
                self.handle
            }
        }

        impl $(<$($lt$(:$clt$(+$dlt)*)?),+>)?  $crate::common::handle::Indexable for $name $(<$($lt),+>)? {
            #[inline(always)]
            fn index(&self) -> usize {
                self.handle.index()
            }
        }
    }
}
