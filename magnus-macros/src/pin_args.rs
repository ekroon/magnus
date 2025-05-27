use proc_macro::TokenStream;

/// Attribute macro for stack pinning function arguments.
///
/// This is currently a no-op and only exists as an opt-in marker for future
/// expansion.
pub fn expand(_attrs: TokenStream, item: TokenStream) -> TokenStream {
    item
}
