//! # Origami Engine
//! A Rust templating engine that allows for rendering HTML elements and building reusable components.
//!
//! ## Overview
//! - `anon!`: Used for generating HTML content, handling conditionals, loops, and expressions.
//! - `comp!`: Simplifies creating reusable components by allowing the definition of props that can be replaced dynamically.

/// # `anon!` Macro
///
/// The `anon!` macro is responsible for rendering HTML elements into a `String`. It supports conditionals, loops, and any expression that returns `&str`.
///
/// ### Basic Usage
/// ```rust
/// use origami_macros::anon;
///
/// let mut s = String::new();
/// anon!(s, div {
///     "Hello, World!"
/// });
/// assert_eq!(s, "<div>Hello, World!</div>");
/// ```
///
/// ### Expressions
/// You can include any expression that returns `&str` directly in the HTML:
/// ```rust
/// use origami_macros::anon;
///
/// let foo = "some_string";
/// let mut s = String::new();
/// anon!(s, div {
///     *foo;
/// });
/// assert_eq!(s, "<div>some_string</div>");
///
/// let mut s = String::new();
/// let bar = "dynamic_string";
/// anon!(s, div {
///     *format!("Hello, {}", bar).as_str(); // Using format! as an expression
/// });
/// assert_eq!(s, "<div>Hello, dynamic_string</div>");
/// ```
///
/// ### Match Expressions
/// `anon!` supports match expressions for dynamic content:
/// ```rust
/// use origami_macros::anon;
///
/// let value = "bar";
/// let mut s = String::new();
/// anon!(s, div {
///     match value; {
///         "bar" => { "Bar Component" },
///         _ => { "Default Component" },
///     }
/// });
/// assert_eq!(s, "<div>Bar Component</div>");
/// ```
///
/// ### Conditionals
/// `anon!` supports conditionals directly:
/// ```rust
/// use origami_macros::anon;
///
/// let condition = true;
/// let mut s = String::new();
/// anon!(s, div {
///     if condition; {
///         "Condition met!"
///     }
/// });
/// assert_eq!(s, "<div>Condition met!</div>");
/// ```
///
/// ### Loops
/// You can iterate over collections with `anon!`:
/// ```rust
/// use origami_macros::anon;
///
/// struct Points {
///     x: i32,
///     y: i32,
/// }
/// let points = [
///     Points { x: 1, y: 2 },
///     Points { x: 3, y: 4 },
/// ];
/// let mut s = String::new();
/// anon!(s, div {
///     for point in points.iter(); {
///         div {
///             *point.x.to_string().as_str();
///             ", "
///             *point.y.to_string().as_str();
///         }
///     }
/// });
/// assert_eq!(s, "<div><div>1, 2</div><div>3, 4</div></div>");
/// ```
pub use origami_macros::anon;

/// # `comp!` Macro
///
/// The `comp!` macro generates a reusable component by defining its structure and allowing the replacement of props. This facilitates the creation of dynamic components that can be rendered with different values.
///
/// ### Example
/// ```rust
/// use origami_macros::{comp};
///
/// comp! {
///     greeting_component =>
///     div {
///         "Hello, "
///         $name
///     }
///     
/// }
///
/// let html = greeting_component!(name { "World" });
/// assert_eq!(html.0, "<div>Hello, World</div>");
/// ```
pub use origami_macros::comp;

pub struct Origami(pub String);

#[cfg(feature = "html_escape")]
#[doc(no_inline)]
pub use html_escape::encode_text_to_string;

#[cfg(feature = "minify_html")]
#[doc(no_inline)]
pub use minify_html::*;

#[cfg(feature = "axum")]
use ::axum::response::{Html, IntoResponse, Response};
#[cfg(feature = "axum")]
impl IntoResponse for Origami {
    fn into_response(self) -> Response {
        Html(self.0).into_response()
    }
}
