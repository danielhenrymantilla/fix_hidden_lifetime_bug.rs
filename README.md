# `::fix_hidden_lifetime_bug`

[![Repository](https://img.shields.io/badge/repository-GitHub-brightgreen.svg)](
https://github.com/danielhenrymantilla/fix_hidden_lifetime_bug.rs)
[![Latest version](https://img.shields.io/crates/v/fix_hidden_lifetime_bug.svg)](
https://crates.io/crates/fix_hidden_lifetime_bug)
[![Documentation](https://docs.rs/fix_hidden_lifetime_bug/badge.svg)](
https://docs.rs/fix_hidden_lifetime_bug)
[![MSRV](https://img.shields.io/badge/MSRV-1.39.0-white)](
https://gist.github.com/danielhenrymantilla/8e5b721b3929084562f8f65668920c33)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](
https://github.com/rust-secure-code/safety-dance/)
[![License](https://img.shields.io/crates/l/fix_hidden_lifetime_bug.svg)](
https://github.com/danielhenrymantilla/fix_hidden_lifetime_bug.rs/blob/master/LICENSE-ZLIB)
[![CI](https://github.com/danielhenrymantilla/fix_hidden_lifetime_bug.rs/workflows/CI/badge.svg)](
https://github.com/danielhenrymantilla/fix_hidden_lifetime_bug.rs/actions)

### Are you getting one of the two following errors (E700)?

  - ```rust,ignore
    error[E0700]: hidden type for `impl Trait` captures lifetime that does not appear in bounds
      --> examples/main.rs:13:40
       |
    13 | fn foo<'a, 'b> (it: &'a mut &'b ()) -> impl 'a + Sized {
       |                                        ^^^^^^^^^^^^^^^
       |
    note: hidden type `&'a mut &'b ()` captures the lifetime `'b` as defined on the function body at 13:12
      --> examples/main.rs:13:12
       |
    13 | fn foo<'a, 'b> (it: &'a mut &'b ()) -> impl 'a + Sized {
       |            ^^
    ```

    <details><summary>Problematic code</summary>

    ```rust,compile_fail
    fn foo<'a, 'b> (it: &'a mut &'b ()) -> impl 'a + Sized {
        it
    }
    ```

    </details>

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

  - ```rust,ignore
    error[E0700]: hidden type for `impl Trait` captures lifetime that does not appear in bounds
     --> examples/main.rs:4:57
      |
    4 | async fn baz<'a> (a: &'static (), b: &'_ (), c: &'_ ()) {
      |                                                         ^
      |
    note: hidden type `impl Future` captures lifetime smaller than the function body
     --> examples/main.rs:4:57
      |
    4 | async fn baz<'a> (a: &'static (), b: &'_ (), c: &'_ ()) {
      |                                                         ^
    ```

    <details><summary>Problematic code</summary>

    ```rust,compile_fail
    async fn baz<'a> (a: &'static (), b: &'_ (), c: &'_ ()) {
        /* … */
    }
    ```

    </details>

Then you can can the attribute provided by this crate to automagically generate
an equivalent signature that soothes this grumpy compiler

### Usage

 1. `cargo add fix_hidden_lifetime_bug`, or add the following to your `Cargo.toml` file:

    ```toml
    [dependencies]
    fix_hidden_lifetime_bug = "x.y.z"
    ```

      - where you can find the version using `cargo search fix_hidden_lifetime_bug`

 1. Add the following to your `lib.rs` file:

    ```rust,ignore
    #[macro_use]
    extern crate fix_hidden_lifetime_bug;
    ```

 1. Slap a `#[fix_hidden_lifetime_bug]` on the problematic function:

    ```rust
    # use ::fix_hidden_lifetime_bug::*;

    #[fix_hidden_lifetime_bug] // <-- Add this!
    fn foo<'a, 'b>(it: &'a mut &'b ()) -> impl 'a + Sized {
        it
    }
    ```

    ```rust
    # use ::fix_hidden_lifetime_bug::*;

    #[fix_hidden_lifetime_bug] // <-- Add this!
    async fn baz<'a>(fst: &'static str, snd: &str, thrd: &str) {
        /* … */
    }
    ```
