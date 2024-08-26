Procedural macros that conditionally change or remove `Send` bounds.

## Examples

```rust 
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
```

The above will expand to:

```rust
// ...

pub trait T3 {
  /// Documentation tests.
  #[cfg(feature = "glommio")]
  fn foo<A, B>(&self, _a: A, _b: B) -> impl Future<Output=()>
  where
    A: T1,
    B: T2,
  {
    async move { () }
  }

  /// Documentation tests.
  #[cfg(not(feature = "glommio"))]
  fn foo<A, B>(&self, _a: A, _b: B) -> impl Future<Output=()> + Send
  where
    A: T1 + Send,
    B: T2 + Send,
  {
    async move { () }
  }
} 
```