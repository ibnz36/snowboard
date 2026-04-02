# **Contributing guidelines**

Since this project is fairly small, contributing isn't much of a headache. You'll only need to have [rust & cargo](https://www.rust-lang.org/es) installed and a working machine.

Pull requests and issues can be opened freely if needed using their respective templates.

## **Recommended:**

-   Small commits (avoid large commits that change too much things at once)
-   Use [gitmoji](https://gitmoji.dev/) or [cm](https://github.com/ibnz36/cm)
-   Not to change the version number yourself but rather specify the version number in the PR description and let the maintainer change it

## **Must:**

-   Run `cargo fmt`, `cargo check`, `cargo test` and `cargo clippy` before commiting
-   Use any extension or plugin for your editor that supports `.editorconfig`
-   Follow [the code of conduct](./CODE_OF_CONDUCT.md) (basic, common sense attitude in github)
-   Follow the semantic versioning guidelines (major.minor.patch) for the version number

## **Must not:**

-   Spam unnecessary issues or PRs (multiple issues or PRs for the same thing, etc)
-   Ignore failing tests or warnings (unless you have a valid reason & are going to fix it later either by correcting the error or using `#[allow(...)]`)
-   Commit unformatted code and refusing to format it
-   Break the code of conduct guidelines

**Note more rules may be applied to common sense.**
