extern crate rm_send_macros;

use rm_send_macros::rm_send_if;
use std::future::Future;

pub trait T1 {}
pub trait T2 {}

pub trait T3 {
  /// Documentation tests.
  #[rm_send_if(feature = "glommio")]
  fn foo<A, B>(&self, _a: A, _b: B) -> impl Future<Output=()> + Send
  where
    A: T1 + Send,
    B: T2 + Send,
  {
    async move { () }
  }
}
