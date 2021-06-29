# `::fix_hidden_lifetime_bug`

[![Repository](https://img.shields.io/badge/repository-GitHub-brightgreen.svg)](
https://github.com/danielhenrymantilla/fix_hidden_lifetime_bug.rs)
[![Latest version](https://img.shields.io/crates/v/fix-hidden-lifetime-bug.svg)](
https://crates.io/crates/fix-hidden-lifetime-bug)
[![Documentation](https://docs.rs/fix-hidden-lifetime-bug/badge.svg)](
https://docs.rs/fix-hidden-lifetime-bug)
[![MSRV](https://img.shields.io/badge/MSRV-1.39.0-white)](
https://gist.github.com/danielhenrymantilla/8e5b721b3929084562f8f65668920c33)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](
https://github.com/rust-secure-code/safety-dance/)
[![License](https://img.shields.io/crates/l/fix-hidden-lifetime-bug.svg)](
https://github.com/danielhenrymantilla/fix_hidden_lifetime_bug.rs/blob/master/LICENSE-ZLIB)
[![CI](https://github.com/danielhenrymantilla/fix_hidden_lifetime_bug.rs/workflows/CI/badge.svg)](
https://github.com/danielhenrymantilla/fix_hidden_lifetime_bug.rs/actions)

### Are you getting one of the following errors (E700)?

  - ```rust,ignore
    error[E0700]: hidden type for `impl Trait` captures lifetime that does not appear in bounds
      --> examples/main.rs:13:40
       |
    13 | fn foo<'a, 'b>(it: &'a mut &'b ()) -> impl 'a + Sized {
       |                                       ^^^^^^^^^^^^^^^
       |
    note: hidden type `&'a mut &'b ()` captures the lifetime `'b` as defined on the function body at 13:12
      --> examples/main.rs:13:12
       |
    13 | fn foo<'a, 'b>(it: &'a mut &'b ()) -> impl 'a + Sized {
       |            ^^
    ```

    <details><summary>Problematic code</summary>

    ```rust,compile_fail
    fn foo<'a, 'b>(it: &'a mut &'b ()) -> impl 'a + Sized {
        it
    }
    ```

    </details>

    <br/>

  - ```rust,ignore
    error[E0700]: hidden type for `impl Trait` captures lifetime that does not appear in bounds
     --> examples/main.rs:8:45
      |
    8 | async fn bar<'a> (_: &(), _: Box<dyn Send>) {
      |                                             ^
      |
    note: hidden type `impl Future` captures lifetime smaller than the function body
     --> examples/main.rs:8:45
      |
    8 | async fn bar<'a> (_: &(), _: Box<dyn Send>) {
      |                                             ^
    ```

    <details><summary>Problematic code</summary>

    ```rust,compile_fail
    async fn bar<'a> (_: &(), _: Box<dyn Send>) {
        /* … */
    }
    ```

    </details>

    <br/>

  - ```rust,ignore
    error[E0700]: hidden type for `impl Trait` captures lifetime that does not appear in bounds
     --> examples/main.rs:4:57
      |
    4 | async fn baz<'a>(a: &'static (), b: &(), c: &()) {
      |                                                  ^
      |
    note: hidden type `impl Future` captures lifetime smaller than the function body
     --> examples/main.rs:4:57
      |
    4 | async fn baz<'a>(a: &'static (), b: &(), c: &()) {
      |                                                  ^
    ```

    <details><summary>Problematic code</summary>

    ```rust,compile_fail
    async fn baz<'a>(a: &'static (), b: &(), c: &()) {
        /* … */
    }
    ```

    </details>

    <br/>

Then you can add the attribute provided by this crate to automagically generate
an equivalent signature that soothes such a grumpy compiler 🙃

  - See [the lifetime bug `async` issue], as well as [this other comment](
    https://github.com/rust-lang/rust/issues/34511#issuecomment-373423999) for
    more info.

    The fix is thus to perform the unsugaring from an `async fn` to an `fn`
    yielding a `Future`, and then just adding the necessary `+ Captures<'_>`
    bounds.

  - See also [this post](
    https://users.rust-lang.org/t/lifetimes-in-smol-executor/59157/8?u=yandros)
    where I explain the issue more in detail.

[the lifetime bug `async` issue]: https://github.com/rust-lang/rust/issues/63033

### Usage

 1. `cargo add fix_hidden_lifetime_bug`, or add the following to your `Cargo.toml` file:

    ```toml
    [dependencies]
    fix-hidden-lifetime-bug = "x.y.z"
    ```

      - where you can find the version using `cargo search fix_hidden_lifetime_bug`

 1. Add the following to your `lib.rs` file:

    ```rust,ignore
    #[macro_use]
    extern crate fix_hidden_lifetime_bug;
    ```

 1. Slap a `#[fix_hidden_lifetime_bug]` on the problematic function:

    ```rust,ignore
    #[fix_hidden_lifetime_bug] // <-- Add this!
    fn foo<'a, 'b>(it: &'a mut &'b ()) -> impl 'a + Sized {
        it
    }
    ```

    ```rust,ignore
    #[fix_hidden_lifetime_bug] // <-- Add this!
    async fn baz<'a>(fst: &'static str, snd: &str, thrd: &str) {
        /* … */
    }
    ```

### Extra features

  - #### Full support for methods

    <details>

    In the case of methods, the `Self` type may be hiding lifetime parameters on
    its own, in which case a macro annotation on the method alone may not have
    enough syntactical information to generate the fix:

    ```rust,compile_fail
    use ::fix_hidden_lifetime_bug::fix_hidden_lifetime_bug;

    struct Invariant<'lt> (
        fn(&()) -> &mut &'lt (),
    );

    impl Invariant<'_> {
        #[fix_hidden_lifetime_bug]
        fn quux(&self) -> impl '_ + Sized { self }
    }
    ```

    In that case, the fix is to also decorate the whole `impl` block with
    the attribute:

    ```rust
    use ::fix_hidden_lifetime_bug::fix_hidden_lifetime_bug;

    struct Invariant<'lt> (
        fn(&()) -> &mut &'lt (),
    );

    #[fix_hidden_lifetime_bug]
    impl Invariant<'_> {
        #[fix_hidden_lifetime_bug]
        fn quux(&self) -> impl '_ + Sized { self }
    }
    ```

    ___

    </details>

  - #### Displaying the expansions

    By enabling the `"showme"` Cargo feature:

    ```toml
    [dependencies]
    fix-hidden-lifetime-bug.version = "x.y.z"
    fix-hidden-lifetime-bug.features = ["showme"]
    ```

    you can then feed a `showme` parameter to specific
    `#[fix_hidden_lifetime_bug]` annotations, as follows:

    ```rust,ignore
    #[fix_hidden_lifetime_bug(showme)]
    ```

    <details><summary>Example</summary>

    ```rust,ignore
    use ::fix_hidden_lifetime_bug::fix_hidden_lifetime_bug;

    #[fix_hidden_lifetime_bug(showme)]
    async fn baz<'a>(a: &'static (), b: &'_ (), c: &'_ ()) {
        println!("Hello, World!");
    }
    ```

    outputs:

    ```rust
    fn baz<'a, '_0, '_1, '__async_fut>(
        a: &'static (),
        b: &'_0 (),
        c: &'_1 (),
    ) -> impl '__async_fut
          + ::fix_hidden_lifetime_bug::core::future::Future<Output = ()>
          + ::fix_hidden_lifetime_bug::Captures<'a>
          + ::fix_hidden_lifetime_bug::Captures<'_0>
          + ::fix_hidden_lifetime_bug::Captures<'_1>
    where
        &'static (): '__async_fut,
        &'_0 (): '__async_fut,
        &'_1 (): '__async_fut,
    {
        async move {
            "Mention the input vars so that they get captured by the Future";
            let (a, b, c) = (a, b, c);
            println!("Hello, World!");
        }
    }
    ```

    </details>

    <br/>

    This will make the attribute display the result of its expansion (and
    its expansion only! Hence yielding output that is way more readable than
    that from `cargo expand` or other such tools), basically showing you how to
    manually fix a given function signature if you so wish (_e.g._, to avoid
    depending on proc-macro processing _every_ time the annotated function is
    compiled, or to make the life easier for IDEs).

    Should you fix the signature, you may then be interested in:

  - ### Opting out of the magic proc-macro attribute

    If you don't want to have to recompile each time the proc-macro able to fix
    function signatures for you (_e.g._, you rather want it to [show you how to
    fix the signature] so that you can do it through exclusive usage of
    `+ Captures<'…>` additions), so as not to have to pay the proc-macro
    compilation time each time you compile from scratch, then you can opt out of
    it by disabling the `default-features` of the crate: this will disable the
    `proc-macros` features, which is the one that brings it to the table.

    That way, you can still use this then very lightweight crate just for its
    `Captures<'…>` (and maybe `MentionsTy<…>`) definitions, and the
    documentation that goes with it!

    ```toml
    [dependencies]
    …
    fix-hidden-lifetime-bug.version = "x.y.z"
    fix-hidden-lifetime-bug.default-features = false
    ```

[show you how to fix the signature]: #displaying-the-expansions
