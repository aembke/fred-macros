Procedural macros that conditionally change or remove `Send` bounds.

## Examples

### Item (function) Modification

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

This will expand to:

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

### Trait Modification

```rust
extern crate rm_send_macros;

use rm_send_macros::rm_send_if;
use std::future::Future;

trait T1 {}
trait T2 {}

/// Test trait documentation.
#[rm_send_if(feature = "glommio")]
pub trait T3: Clone + Send + Sync {
  /// Test fn documentation
  fn bar<A, B>(&self, _a: A, _b: B) -> impl Future<Output=()> + Send
  where
    A: T1 + Send,
    B: T2 + Send + Sync,
  {
    async move { () }
  }
}
```

This will expand to:

```rust
// ... 

#[cfg(feature = "glommio")]
pub trait T3: Clone {
  /// Test fn documentation
  fn bar<A, B>(&self, _a: A, _b: B) -> impl Future<Output=()>
  where
    A: T1,
    B: T2,
  {
    async move { () }
  }
}

#[cfg(not(feature = "glommio"))]
pub trait T3: Clone + Send + Sync {
  /// Test fn documentation
  fn bar<A, B>(&self, _a: A, _b: B) -> impl Future<Output=()> + Send
  where
    A: T1 + Send,
    B: T2 + Send + Sync,
  {
    async move { () }
  }
}
```