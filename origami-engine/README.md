# Origami Engine

Origami Engine is a templating engine focused on modularity, designed for efficient HTML generation with powerful macros.

## Features

- Focused on modularity for easy extensibility
- Support for expressions, conditionals, loops, and match expressions

## Basic Example

```rust
use origami_engine::comp;

comp! {
    greeting =>
    div {
        "Hello, World!"
    }
}

let html = greeting!();
assert_eq!(html.0, "<div>Hello, World!</div>");
```

# Documentation

For comprehensive documentation and usage instructions, please visit [docs.rs](https://docs.rs/origami-engine/latest/origami_engine/).
