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
  use syn::{parse::Parser, GenericParam, Item, ItemFn, ReturnType, Type, TypeParamBound, WherePredicate};

  fn filter_send_bounds(
    bounds: impl IntoIterator<Item = TypeParamBound>,
  ) -> impl IntoIterator<Item = TypeParamBound> {
    bounds.into_iter().filter(|bound| {
      // TODO handle other path kinds
      if let TypeParamBound::Trait(bound) = bound {
        !bound.path.is_ident("Send")
      } else {
        true
      }
    })
  }

  pub(crate) fn rm_send(input: TokenStream) -> TokenStream {
    let mut ast: ItemFn = match syn::parse2::<Item>(input.clone()) {
      Ok(Item::Fn(item)) => item,
      _ => return input,
    };

    // remove Send from generic params
    for param in ast.sig.generics.params.iter_mut() {
      if let GenericParam::Type(ty) = param {
        ty.bounds = filter_send_bounds(ty.bounds.clone()).into_iter().collect();
      }
    }
    // remove Send from where clause predicates
    if let Some(ref mut where_clause) = ast.sig.generics.where_clause {
      for predicate in where_clause.predicates.iter_mut() {
        if let WherePredicate::Type(predicate) = predicate {
          predicate.bounds = filter_send_bounds(predicate.bounds.clone()).into_iter().collect();
        }
      }
    }
    // remove Send from any `impl Trait` return types
    if let ReturnType::Type(_, ref mut ty) = ast.sig.output {
      if let Type::ImplTrait(ty) = ty.as_mut() {
        ty.bounds = filter_send_bounds(ty.bounds.clone()).into_iter().collect();
      }
    }

    ast.into_token_stream()
  }
}

/// Conditionally removes `+Send` constraints on trait bounds in generic arguments, where clause predicates, and
/// `impl Trait` return types.
///
/// ```rust no_compile no_run
/// use rm_send_macros::rm_send_if;
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
/// #[cfg(not(feature = "glommio))]
/// pub async fn foo<R, A, B>(a: A, b: B) -> impl Future<Output = R> + Send
/// where
///   A: T1 + Send,
///   B: T2 + Send,
/// {
///   unimplemented!()
/// }
/// ```
#[proc_macro_attribute]
#[cfg(not(feature = "enabled"))]
pub fn rm_send_if(_: TokenStream, input: TokenStream) -> TokenStream {
  input
}

#[cfg(feature = "enabled")]
fn wrap_cfg_if(args: TokenStream, modified: TokenStream, input: TokenStream) -> TokenStream {
  format!("#[cfg({args})]\n{modified}\n#[cfg(not({args}))]\n{input}")
    .parse()
    .unwrap()
}

/// Conditionally removes `+Send` constraints on trait bounds in generic arguments, where clause predicates, and
/// `impl Trait` return types.
///
/// ```rust no_compile no_run
/// use rm_send_macros::rm_send_if;
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
/// #[cfg(not(feature = "glommio))]
/// pub async fn foo<R, A, B>(a: A, b: B) -> impl Future<Output = R> + Send
/// where
///   A: T1 + Send,
///   B: T2 + Send,
/// {
///   unimplemented!()
/// }
/// ```
#[proc_macro_attribute]
#[cfg(feature = "enabled")]
pub fn rm_send_if(args: TokenStream, input: TokenStream) -> TokenStream {
  let modified: TokenStream = inner::rm_send(input.clone().into()).into();
  wrap_cfg_if(args, modified, input)
}

#[cfg(test)]
#[cfg(feature = "enabled")]
mod tests {
  use crate::inner::rm_send;
  use quote::quote;

  #[test]
  fn should_remove_basic_send() {
    let input = quote! {
      pub async fn foo<R, A, B: Baz>(a: B, b: B) -> impl Future<Output = R> + Send
        where R: Send,
              A: Foo + Send,
              B: Bar + Send
      {
        async move { Ok(()) }
      }
    };
    let modified = rm_send(input);
    assert!(!modified.to_string().contains("Send"));
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
}
