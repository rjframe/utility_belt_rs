# Utility types and functions for Rust

This is a collection of code I use in many/most of my Rust projects.


## Table of Contents

* [License](#license)
* [Running Tests](#running-tests)
* [Using](#using)
* [Source Overview](#source-overview)
* [Contributing](#contributing)
    * [Code and Patch Styles](#code-and-patch-styles)
* [Contact](#contact)


## License

All source code is licensed under the terms of the
[MPL 2.0 license](LICENSE.txt).


## Running Tests

Simply run `cargo test`, with whatever feature set you wish to use.
Or with [cargo-hack](https://crates.io/crates/cargo-hack):
`cargo hack --each-feature test`.


## Using

Unless I get requests to do so, I do not intend to list this on the crates.io
registry. Add to your `Cargo.toml`:

```toml
[dependencies]
utility_belt = {git = "https://git.sr.ht/~rjframe/utility_belt_rs", tag = "v0.1.0"}
```

Note: crates.io will not let you publish a package that includes a git
dependency. If you intend to use this library and publish your package, let me
know and I'll list this on the registry.

The `Cargo.toml` provides a complete list of dependencies for any given feature.

Alternatively, most files are either wholly or mostly-independent, and can
simply be copied into your project source, with minimal dependency updates to
your `Cargo.toml` and module paths (useful if all you want is, e.g., the `Uniq`
iterator). The MPL applies at the file boundary, so you can safely copy these
files as-is without worrying about messing up the licensing for your own code.


## Source Overview

Build and run the documentation (`cargo doc --all-features --no-deps --open`)
for full documentation. This section is a brief review of the code:

* **config.rs**: Provides an INI parser and configuration object. Can optionally
  make itself a global object.
* **iter**: A collection of useful iterators.
* **secure_string.rs**: A string type that prevents clones and wipes its memory
  when dropped.


## Contributing

The official code repository is hosted at
[https://git.sr.ht/~rjframe/utility_belt_rs](https://git.sr.ht/~rjframe/utility_belt_rs);
please send any bug reports, patches, or other communications to
[https://lists.sr.ht/~rjframe/public](https://lists.sr.ht/~rjframe/public).


### Code and Patch Styles

#### Commit Messages

The first line of a commit message should summarize the purpose of the commit.
It should be a full sentence but end without a period. The subject line must be
no more than 72 columns, preferably no more than 50.

If the commit addresses a specific feature or module, prefix the commit message
with that name (e.g., "iter: Some message here").

Write the subject in imperative style (like you're telling someone what to do);
use "Add xyz" instead of "Added xyz", "Fix" instead of "Fixed", etc.

The body of the message must have a hard 72-column line limit.

#### Code Style

* Use four spaces for indentation.
* Use a hard 80 column line width.
* Write tests whenever practical; exercise error conditions and edge cases as
  well as the happy path.
* Document all public declarations. Also document non-trivial private
  declarations.
* Follow the typical Rust naming conventions.
* If an import is used in one or very few places in a module, prefer a local
  import to a global one (import inside the function rather than the top of the
  file).
* Place braces on the same line as the preceding code, unless doing so would
  break the 80-column rule.
* In general, try to conform to the style of the code in which you're working.


## Contact

- Email: code@ryanjframe.com
- Website: [www.ryanjframe.com](https://www.ryanjframe.com)
- diaspora*: rjframe@diasp.org
