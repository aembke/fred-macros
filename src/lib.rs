#![warn(clippy::large_types_passed_by_value)]
#![warn(clippy::large_stack_frames)]
#![warn(clippy::large_futures)]
#![cfg_attr(docsrs, deny(rustdoc::broken_intra_doc_links))]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![doc = include_str!("../README.md")]

#[allow(unused_extern_crates)]
extern crate proc_macro;

use proc_macro::TokenStream;

#[cfg(feature = "enabled")]
mod inner {
  use proc_macro2::TokenStream;
  use quote::ToTokens;
  use syn::{
    GenericParam,
    ImplItem,
    Item,
    ItemFn,
    ItemImpl,
    ItemTrait,
    ReturnType,
    Signature,
    TraitItem,
    Type,
    TypeParamBound,
    WherePredicate,
  };

  fn filter_send_bounds(
    bounds: impl IntoIterator<Item = TypeParamBound>,
  ) -> impl IntoIterator<Item = TypeParamBound> {
    bounds.into_iter().filter(|bound| {
      // TODO handle other path kinds
      if let TypeParamBound::Trait(bound) = bound {
        !(bound.path.is_ident("Send") || bound.path.is_ident("Sync"))
      } else {
        true
      }
    })
  }

  fn rm_from_fn_sig(sig: &mut Signature) {
    // remove Send from generic params
    for param in sig.generics.params.iter_mut() {
      if let GenericParam::Type(ty) = param {
        ty.bounds = filter_send_bounds(ty.bounds.clone()).into_iter().collect();
      }
    }
    // remove Send from where clause predicates
    if let Some(ref mut where_clause) = sig.generics.where_clause {
      for predicate in where_clause.predicates.iter_mut() {
        if let WherePredicate::Type(predicate) = predicate {
          predicate.bounds = filter_send_bounds(predicate.bounds.clone()).into_iter().collect();
        }
      }
    }
    // remove Send from any `impl Trait` return types
    if let ReturnType::Type(_, ref mut ty) = sig.output {
      if let Type::ImplTrait(ty) = ty.as_mut() {
        ty.bounds = filter_send_bounds(ty.bounds.clone()).into_iter().collect();
      }
    }
  }

  fn rm_from_item_fn(mut ast: ItemFn) -> TokenStream {
    rm_from_fn_sig(&mut ast.sig);
    ast.into_token_stream()
  }

  fn rm_from_impl(mut ast: ItemImpl) -> TokenStream {
    for item in ast.items.iter_mut() {
      if let ImplItem::Fn(item) = item {
        rm_from_fn_sig(&mut item.sig);
      }
    }
    ast.into_token_stream()
  }

  fn rm_from_item_trait(mut ast: ItemTrait) -> TokenStream {
    ast.supertraits = filter_send_bounds(ast.supertraits.clone()).into_iter().collect();

    for item in ast.items.iter_mut() {
      if let TraitItem::Fn(item) = item {
        rm_from_fn_sig(&mut item.sig);
      }
    }

    ast.into_token_stream()
  }

  pub(crate) fn rm_send(input: TokenStream) -> TokenStream {
    match syn::parse2::<Item>(input.clone()) {
      Ok(Item::Fn(item)) => rm_from_item_fn(item),
      Ok(Item::Trait(item)) => rm_from_item_trait(item),
      Ok(Item::Impl(item)) => rm_from_impl(item),
      _ => input,
    }
  }
}

/// Conditionally removes `Send` and `Sync` bounds in generic arguments, where clause predicates, and
/// `impl Trait` return types.
///
///
/// ## Item (function) modification:
///
/// ```rust
/// use fred_macros::rm_send_if;
/// use std::future::Future;
///
/// trait T1 {}
/// trait T2 {}
///
/// #[rm_send_if(feature = "glommio")]
/// pub async fn foo<R, A, B>(a: A, b: B) -> impl Future<Output = R> + Send
/// where
///   A: T1 + Send,
///   B: T2 + Send,
/// {
///   unimplemented!()
/// }
///
/// // will be modified to the following
///
/// #[cfg(feature = "glommio")]
/// pub async fn foo<R, A, B>(a: A, b: B) -> impl Future<Output = R>
/// where
///   A: T1,
///   B: T2,
/// {
///   unimplemented!()
/// }
///
/// #[cfg(not(feature = "glommio"))]
/// pub async fn foo<R, A, B>(a: A, b: B) -> impl Future<Output = R> + Send
/// where
///   A: T1 + Send,
///   B: T2 + Send,
/// {
///   unimplemented!()
/// }
/// ```
///
/// ### Trait modification
///
/// ```rust
/// use fred_macros::rm_send_if;
/// use std::future::Future;
///
/// trait T1 {}
/// trait T2 {}
///
/// /// Test trait documentation.
/// #[rm_send_if(feature = "glommio")]
/// pub trait T3: Clone + Send + Sync {
///   /// Test fn documentation
///   fn bar<A, B>(&self, _a: A, _b: B) -> impl Future<Output = ()> + Send
///   where
///     A: T1 + Send,
///     B: T2 + Send + Sync,
///   {
///     async move { () }
///   }
/// }
///
/// // will be modified to the following
///
/// #[cfg(feature = "glommio")]
/// pub trait T3: Clone {
///   /// Test fn documentation
///   fn bar<A, B>(&self, _a: A, _b: B) -> impl Future<Output = ()>
///   where
///     A: T1,
///     B: T2,
///   {
///     async move { () }
///   }
/// }
///
/// #[cfg(not(feature = "glommio"))]
/// pub trait T3: Clone + Send + Sync {
///   /// Test fn documentation
///   fn bar<A, B>(&self, _a: A, _b: B) -> impl Future<Output = ()> + Send
///   where
///     A: T1 + Send,
///     B: T2 + Send + Sync,
///   {
///     async move { () }
///   }
/// }
/// ```
#[proc_macro_attribute]
#[cfg(not(feature = "enabled"))]
pub fn rm_send_if(_: TokenStream, input: TokenStream) -> TokenStream {
  input
}

#[cfg(feature = "enabled")]
fn wrap_cfg_attr(args: TokenStream, modified: TokenStream, input: TokenStream) -> TokenStream {
  format!("#[cfg({args})]\n{modified}\n#[cfg(not({args}))]\n{input}")
    .parse()
    .unwrap()
}

/// Conditionally removes `Send` and `Sync` bounds in generic arguments, where clause predicates, and
/// `impl Trait` return types.
///
///
/// ## Item (function) modification:
///
/// ```rust
/// use fred_macros::rm_send_if;
/// use std::future::Future;
///
/// trait T1 {}
/// trait T2 {}
///
/// #[rm_send_if(feature = "glommio")]
/// pub async fn foo<R, A, B>(a: A, b: B) -> impl Future<Output = R> + Send
/// where
///   A: T1 + Send,
///   B: T2 + Send,
/// {
///   unimplemented!()
/// }
///
/// // will be modified to the following
///
/// #[cfg(feature = "glommio")]
/// pub async fn foo<R, A, B>(a: A, b: B) -> impl Future<Output = R>
/// where
///   A: T1,
///   B: T2,
/// {
///   unimplemented!()
/// }
///
/// #[cfg(not(feature = "glommio"))]
/// pub async fn foo<R, A, B>(a: A, b: B) -> impl Future<Output = R> + Send
/// where
///   A: T1 + Send,
///   B: T2 + Send,
/// {
///   unimplemented!()
/// }
/// ```
///
/// ### Trait modification
///
/// ```rust
/// use fred_macros::rm_send_if;
/// use std::future::Future;
///
/// trait T1 {}
/// trait T2 {}
///
/// /// Test trait documentation.
/// #[rm_send_if(feature = "glommio")]
/// pub trait T3: Clone + Send + Sync {
///   /// Test fn documentation
///   fn bar<A, B>(&self, _a: A, _b: B) -> impl Future<Output = ()> + Send
///   where
///     A: T1 + Send,
///     B: T2 + Send + Sync,
///   {
///     async move { () }
///   }
/// }
///
/// // will be modified to the following
///
/// #[cfg(feature = "glommio")]
/// pub trait T3: Clone {
///   /// Test fn documentation
///   fn bar<A, B>(&self, _a: A, _b: B) -> impl Future<Output = ()>
///   where
///     A: T1,
///     B: T2,
///   {
///     async move { () }
///   }
/// }
///
/// #[cfg(not(feature = "glommio"))]
/// pub trait T3: Clone + Send + Sync {
///   /// Test fn documentation
///   fn bar<A, B>(&self, _a: A, _b: B) -> impl Future<Output = ()> + Send
///   where
///     A: T1 + Send,
///     B: T2 + Send + Sync,
///   {
///     async move { () }
///   }
/// }
/// ```
#[proc_macro_attribute]
#[cfg(feature = "enabled")]
pub fn rm_send_if(args: TokenStream, input: TokenStream) -> TokenStream {
  let modified: TokenStream = inner::rm_send(input.clone().into()).into();
  wrap_cfg_attr(args, modified, input)
}

#[cfg(test)]
#[cfg(feature = "enabled")]
mod tests {
  use crate::inner::rm_send;
  use quote::quote;

  #[test]
  fn should_remove_basic_send_and_sync() {
    let input = quote! {
      pub async fn foo<R, A, B: Baz>(a: B, b: B) -> impl Future<Output = R> + Send
        where R: Send,
              A: Foo + Send,
              B: Bar + Send + Sync
      {
        async move { Ok(()) }
      }
    };
    let modified = rm_send(input);
    assert!(!modified.to_string().contains("Send"));
    assert!(!modified.to_string().contains("Sync"));
  }

  #[test]
  fn should_remove_into_send() {
    let input = quote! {
      pub async fn foo<R, A, B>(a: B, b: B) -> impl Future<Output = R> + Send
        where R: Send,
              A: Into<Foo> + Send,
              B: Into<Bar> + Send
      {
        async move { Ok(()) }
      }
    };
    let modified = rm_send(input);
    assert!(!modified.to_string().contains("Send"));
  }

  #[test]
  fn should_remove_try_into_send() {
    let input = quote! {
      pub async fn foo<R, A, B>(a: B, b: B) -> impl Future<Output = Result<R>> + Send
        where R: Send,
              A: TryInto<Foo> + Send,
              A::Error: Into<Error> + Send,
              B: TryInto<Bar> + Send,
              B::Error: Into<Error> + Send
      {
        async move { Ok(()) }
      }
    };
    let modified = rm_send(input);
    assert!(!modified.to_string().contains("Send"));
  }

  #[test]
  fn should_remove_fn_args_send() {
    let input = quote! {
      pub async fn foo<R, A, B, F>(a: B, b: B) -> impl Future<Output = Result<R>> + Send
        where R: Send,
              A: TryInto<Foo> + Send,
              A::Error: Into<Error> + Send,
              B: TryInto<Bar> + Send,
              B::Error: Into<Error> + Send,
              F: Fn(()) -> Result<(), ()> + Send + 'static
      {
        async move { Ok(()) }
      }
    };
    let modified = rm_send(input);
    assert!(!modified.to_string().contains("Send"));
  }

  #[test]
  fn should_remove_trait_impl() {
    let input = quote! {
      impl T4 for Foo {
        fn bar<A, B>(&self, _a: A, _b: B) -> impl Future<Output = ()> + Send
        where
          A: T1 + Send,
          B: T2 + Send + Sync,
        {
          async move { () }
        }
      }
    };

    let modified = rm_send(input);
    assert!(!modified.to_string().contains("Send"));
  }
}
