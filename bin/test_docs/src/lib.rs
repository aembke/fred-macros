extern crate rm_send_macros;

use rm_send_macros::rm_send_if;
use std::future::Future;

pub trait T1 {}
pub trait T2 {}

pub trait T3 {
  /// Test fn documentation.
  #[rm_send_if(feature = "glommio")]
  fn foo<A, B>(&self, _a: A, _b: B) -> impl Future<Output = ()> + Send
  where
    A: T1 + Send,
    B: T2 + Send + Sync,
  {
    async move { () }
  }
}

/// Test trait documentation.
#[rm_send_if(feature = "glommio")]
pub trait T4: Clone + Send + Sync {
  /// Test fn documentation
  fn bar<A, B>(&self, _a: A, _b: B) -> impl Future<Output = ()> + Send
  where
    A: T1 + Send,
    B: T2 + Send + Sync,
  {
    async move { () }
  }
}
