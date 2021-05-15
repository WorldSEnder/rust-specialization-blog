## Specialization, considered harmful - a story in four paragraphs

Specialization is a proposed feature that allows multiple potential implementations for a trait-type combination to exist, of which one most-specific is selected. This makes it possible to write "blanket impls". This post focuses on this angle and tries to argue that such blanket impls, if part of a public interface, are harmful.

### The case of a crate providing a trait

Consider a crate offering a trait `ShowDetails` with the following interface:

```rust
// In some crate called 'trait_host'
pub trait ShowDetails {
    fn fmt_details(&self, f: &mut Formatter) -> fmt::Result;
}
```

The [`specialization`] feature can be leveraged to write an implementation of the above for any type that implements, say `Display`:

```rust
#![feature(min_specialization)]
impl<T> ShowDetails for T where T: Display {
    default fn fmt_details(&self, f: &mut Formatter) -> fmt::Result {
        <Self as Display>::fmt(self, f)
        // ------------------^
        // delegate to the impl of Display to implement `fmt_details`.
    }
}
```

That was very useful, now every type that implements `Display` gets an implementation of `ShowDetails`! So far, so good.

### The case of a crate with functionality

Consider then a crate with a `struct FizzBuzzer`, containing - for this post irrelevant - awesome features. Among other things there is an implementation for `Display` that prints out debug information. Still, the author is unaware of this new `trait_host` crate and as such does not know about `ShowDetails`. She doesn't even know that `FizzBuzzer` implements `ShowDetails`, but it actually does because of the default impl! Still awesome?!

```rust
// In some crate called 'trait_impl'
pub struct FizzBuzzer;

impl Display for FizzBuzzer {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_fmt(format_args!("Useful display, much wow! 🐕"))
    }
}
```

### The case of a crate using the above two crates

Seeing both `trait_host` and `trait_impl`, a new idea is born and a third crate `trait_consumer` emerges. This app is carries around a `FizzBuzzer` in some context and does other useful stuff. Diligently checking the documentation, and discovering both

```rust
// in docs for trait_host
impl<T> ShowDetails for T
where
    T: Display,
// in docs for trait_impl
impl Display for FizzBuzzer
```

the plan is formed to use `<FizzBuzzer as ShowDetails>::fmt_details` to write some sweet business logic - which for simplicity is just printing to the console, but pretend it's more involved. `trait_consumer` is *blissfully unaware* of `#![feature(min_specialization)]`. It was never mentioned anywhere, and didn't have to be turned on to consume the specialized implementation.

### Nailing the coffin shut

In the last act, the maintainer of `trait_impl` learns about `trait_host` and decides to write an `impl ShowDetails for FizzBuzzer`. In stable rust, this would be considered a minor change. Even though it's a *breaking* change due to potentially inducing ambiguity at downstream callsites, all damage should be caught be easy to fix errors. With `specialization` though, the behaviour observed in `trait_consumer` *silently* changes from one implementation to another and **`trait_consumer` suffers the collateral damage**.[^1] Does that mean that specialized impls are major changes?

## Take-away message

The message is not that `#![feature(min_specialization)]` is inherently broken, but rather that correct usage requires utmost care. I want to argue that most exposed blanket implementations, using the feature, are bad style. Either the blanket impl is already correct for all cases and no specialization is needed for more concrete types (e.g. `T: Into<U>`) anyway, or it is not good enough and the more concrete impl will run into the problem outlined above.

When the specialization is restricted to only the crate in which it is defined, there is a usage where one can implement a generic trait differently for specific arguments that is hard to achieve without `specialization`. In the [`Extend`] trait impl for `Vec`, a trick behind the scences can efficiently extend a vector by a slice. It delegates to different impls of a hidden `SpecExtend` trait, which is *specialized* on the type of iterator that is passed to `extend`.[^2] The opinion of this post applies only if the specialization extends over crate boundaries.

### An alternative, for the future

Instead of specialization, the `trait_host` crate, knowing that basic impls of `ShowDetails` can be derived from `Display`, could offer the following, using the [`ref_cast`] crate:

```rust
// In 'trait_host'
// No need for !#[feature(min_specialization)]

// Transparent according to the newtype idiom.
// This struct has the same ABI (byte representation) as the wrapped `T`
#[derive(RefCast)]
//       ^------ some secret sauce, that will be made clear in a moment
#[repr(transparent)]
pub struct ShowDetailsFromDisplay<T>(T);

impl<T> ShowDetails for ShowDetailsFromDisplay<T> where T: Display {
    fn fmt_details(&self, f: &mut Formatter) -> fmt::Result {
        // Replaces the default impl from the start, almost identical
        <T as Display>::fmt(&self.0, f)
    }
}
```

So far, the `trait_consumer` can not leverage `FizzBuzzer: ShowDetails`, so let's have a look at how to write this, using the new `ShowDetailsFromDisplay`, e.g. with help from the [`delegate`] crate.

```rust
impl ShowDetails for FizzBuzzer {
    delegate! {
        to ShowDetailsFromDisplay::ref_cast(self) {
            //                     ^------- secret sauce from RefCast
            fn fmt_details(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result;
        }
    }
}
```

This delegate impl is written mechanically. The secret sauce is `ShowDetailsFromDisplay::ref_cast(&T) -> &ShowDetailsFromDisplay<T>` which let's us delegate the implementation to the one of `ShowDetailsFromDisplay<FizzBuzzer>`. Pretending that `T` and `ShowDetailsFromDisplay<T>` are exactly equal is not always correct, but it is if (but not only if) they are used as arguments to reference types, contained in a `Box`, `Cell` or most standard wrappers, precisely because of the `#[repr(transparent)]` annotation.

A lot of what has been said so far, derives from and heavily uses the equivalent problem in Haskell. `specialization` corrseponds to `{-# OVERLAPPING #-}` instances and the proposed alternative is inspired by the [`DerivingVia`] extension. GHC, the go-to Haskell compiler, *internally* implements the equivalent functionality of `ref_cast` and `delegate` - with some extra power, that has to be saved for another post or an RFC[^3]. If I had to propose a syntax, it would look something like

```rust
// Not possible today
impl ShowDetails for FizzBuzzer
        as ShowDetailsFromDisplay<FizzBuzzer>;
    //  ^--- Internally:
    //    - check that FizzBuzzer and ShowDetailsFromDisplay<FizzBuzzer> have the same representation
    //    - copy all associated types and associated constants
    //    - try to cast all the implementation methods from one type to the other
    //    - if that is not possible, error diagnostic. Should be possible here.
```

[^1]: To experience this, have a look at the [example repo] simulating this scenario.

[^2]: The gory details of this are in https://github.com/rust-lang/rust/blob/50f2bf6a5751751ea27a8fd5577d5bdd37236669/library/alloc/src/vec/spec_extend.rs

[^3]: In the meantime, you can read Haskell blog posts about this, such as [this one](https://samtay.github.io/posts/deriving-via-use-case), or [this presentation](https://andres-loeh.de/deriving-via-haskellx.pdf)

[`specialization`]: https://github.com/rust-lang/rust/issues/31844
[`Extend`]: https://doc.rust-lang.org/std/iter/trait.Extend.html
[`delegate`]: https://docs.rs/crate/delegate/
[`ref_cast`]: https://docs.rs/crate/ref_cast/
[`DerivingVia`]: https://ghc.gitlab.haskell.org/ghc/doc/users_guide/exts/deriving_via.html
[example repo]: https://github.com/WorldSEnder/rust-specialization-blog
