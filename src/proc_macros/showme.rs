use super::*;

pub(in crate)
fn pretty_print_tokenstream (
    code: &'_ TokenStream2,
)
{
    fn try_format (input: &'_ str)
      -> Option<String>
    {Some({
        let mut child =
            ::std::process::Command::new("rustfmt")
                .args(&["--edition", "2018"])
                .stdin(::std::process::Stdio::piped())
                .stdout(::std::process::Stdio::piped())
                .stderr(::std::process::Stdio::piped())
                .spawn()
                .ok()?
        ;
        match child.stdin.take().unwrap() { ref mut stdin => {
            ::std::io::Write::write_all(stdin, input.as_bytes()).ok()?;
        }}
        let mut stdout = String::new();
        ::std::io::Read::read_to_string(
            &mut child.stdout.take().unwrap(),
            &mut stdout,
        ).ok()?;
        if child.wait().ok()?.success().not() { return None; }
        stdout
    })}
    let mut code = code.to_string();
    // Try to format the code, but don't sweat it if it fails.
    if let Some(formatted) = try_format(&code) {
        code = formatted;
    }
    // Now let's try to also colorize it:
    if  ::bat::PrettyPrinter::new()
            .input_from_bytes(code.as_ref())
            .language("rust")
            .true_color(false)
            .snip(true)
            .print()
            .is_err()
    {
        // Fallback to non-colorized output.
        println!("{}", code);
    }
}

pub(in crate)
const BANNER: &str = "\n\n\
👋\n\
Now that you have seen the trick, chances are you will stop depending on this \
crate altogether.\n\
This is fine, and even a good thing, but for the visibility of this very \
crate… 😬\n\
If you have found it useful, feel free to keep using it but with \
`default-features` disabled (at which point you should know the crate will \
have no impact on compile-time whatsoever), or to stop using it, but then ⭐️ \
the repository:\n  \
+-------------------------------------------------------------------+\n  \
| https://github.com/danielhenrymantilla/fix_hidden_lifetime_bug.rs |\n  \
+-------------------------------------------------------------------+\n\
Both things may enhance the visibility of this crate to:\n  \
    - let other users struggling with this bug find out \
    about it;\n  \
    - let the lang team realize that the current _status \
    quo_ is actually a bug: there is no harm in making the \
    returned `impl Trait` be overly dependent on \
    parameters!\n\
Thanks 🙂\n
";
