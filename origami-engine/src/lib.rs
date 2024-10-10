//! # Origami Engine
//! A Rust templating engine that allows for rendering HTML elements and building reusable components.
//!
//! ## Props Passing
//!
//! You can pass props to components to customize their behavior and appearance. Below is an example of a homepage and an about page, both utilizing a button component with various attributes.
//! ```rust
//! use origami_engine::comp;
//!
//! // Define a button component that takes props
//! comp! {
//!     button_component(attr, label) =>
//!     button class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600" @attr; {
//!         @label;
//!     }
//! }
//!
//! // Define the homepage component
//! comp! {
//!     home =>
//!     div {
//!         h1 { "Welcome to the Homepage!" }
//!         // Use the button_component with props
//!         call button_component { attr { onclick="alert('clicked')" }, label { "Click Me" } }
//!     }
//! }
//!
//! let html = home!();
//!
//! assert_eq!(
//!     html.0,
//!     r#"<div><h1>Welcome to the Homepage!</h1><button class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600" onclick="alert('clicked')">Click Me</button></div>"#
//! );
//!
//! // Define the about page component
//! comp! {
//!     about =>
//!     div {
//!         h1 { "About Us" }
//!         p { "We are committed to delivering quality service." }
//!         // Use the button_component with props
//!         call button_component { attr { onclick="alert('clicked learn more')" }, label { "Learn More" } }
//!     }
//! }
//!
//! let html = about!();
//! assert_eq!(
//!     html.0,
//!     r#"<div><h1>About Us</h1><p>We are committed to delivering quality service.</p><button class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600" onclick="alert('clicked learn more')">Learn More</button></div>"#
//! );
//! ```
//!
//! ## Layout
//!
//! You can create a layout structure that includes a navigation bar, a body for dynamic content, and a footer. Below is an example demonstrating this layout.
//!
//! ```rust
//! use origami_engine::comp;
//!
//! // Define a layout component with a navigation bar, body, and footer
//! comp! {
//!     layout_component(content) =>
//!     // Navigation bar
//!     nav {
//!         ul {
//!             li { a { "Home" } }
//!             li { a { "About" } }
//!             li { a { "Contact" } }
//!         }
//!     }
//!     // Body placeholder for dynamic content
//!     main {
//!         @content;
//!     }
//!     // Footer
//!     footer {
//!         p { "© 2024 Your Company" }
//!     }
//! }
//!
//! // Define the homepage component using the layout
//! comp! {
//!     home =>
//!     call layout_component {
//!         content {
//!             h1 { "Welcome to the Homepage!" }
//!             p { "This is the main content of the homepage." }
//!         }
//!     }
//! }
//!
//! let html = home!(cap => 250); // It is recommended to provide `cap`, i.e., the maximum length of html
//!                              // to avoid unnecessary reallocations of strings
//! assert_eq!(
//!     html.0,
//!     r#"<nav><ul><li><a>Home</a></li><li><a>About</a></li><li><a>Contact</a></li></ul></nav><main><h1>Welcome to the Homepage!</h1><p>This is the main content of the homepage.</p></main><footer><p>© 2024 Your Company</p></footer>"#
//! );
//!
//! // Define the about page component using the layout
//! comp! {
//!     about =>
//!     call layout_component {
//!         content {
//!             h1 { "About Us" }
//!             p { "We are committed to delivering quality service." }
//!         }
//!     }
//! }
//!
//! let html = about!(cap => 250); // It is recommended to provide `cap`, i.e., the maximum length of html
//!                              // to avoid unnecessary reallocations of strings
//! assert_eq!(
//!     html.0,
//!     r#"<nav><ul><li><a>Home</a></li><li><a>About</a></li><li><a>Contact</a></li></ul></nav><main><h1>About Us</h1><p>We are committed to delivering quality service.</p></main><footer><p>© 2024 Your Company</p></footer>"#
//! );
//! ```
//! ## Escape and Noescape
//!
//! You can use `escape` and `noescape` to control HTML escaping behavior in the template (`html_escape` is feature is required):
//!
//! ```rust
//! #[cfg(feature = "html_escape")]
//! {
//!     use origami_engine::comp;
//!
//!     comp! {
//!         foo =>
//!         div noescape {
//!             div { "<div>Unsafe HTML</div>" } // Inherited, this will not be escaped
//!             div escape {
//!                 "<div>Safe HTML</div>" // This will be escaped
//!             }
//!         }
//!     }
//!
//!     let html = foo!();
//!     assert_eq!(html.0, "<div><div><div>Unsafe HTML</div></div><div>&lt;div&gt;Safe HTML&lt;/div&gt;</div></div>");
//! }
//! ```
//!
//! You can also use `noescape` or `escape` with conditional rendering:
//!
//! ```rust
//! #[cfg(feature = "html_escape")]
//! {
//!     use origami_engine::comp;
//!
//!     let text = "bar";
//!
//!     comp! {
//!         foo =>
//!         div noescape {
//!             if text == "foo"; noescape {
//!                 "<div>Unsafe HTML</div>"
//!             } else if text == "bar"; escape {
//!                 "<div>Safe HTML</div>"
//!             } else noescape {
//!                 "<div>Default HTML</div>"
//!             }
//!         }
//!     }
//!
//!     let html = foo!();
//!     assert_eq!(html.0, "<div>&lt;div&gt;Safe HTML&lt;/div&gt;</div>");
//! }
//! ```
//!
//! Or with expressions:
//!
//! ```rust
//! #[cfg(feature = "html_escape")]
//! {
//!     use origami_engine::comp;
//!
//!     let text = "<div>foo</div>";
//!     comp! {
//!         foo =>
//!         div { @text;! }
//!     }
//!
//!     let html = foo!();
//!     assert_eq!(html.0, "<div><div>foo</div></div>");
//! }
//! ```
//!
//! Or with literals:
//!
//! ```rust
//! #[cfg(feature = "html_escape")]
//! {
//!     use origami_engine::comp;
//!
//!     comp! {
//!         foo =>
//!         div {
//!             "<div>foo</div>"!
//!         }
//!     }
//!
//!     let html = foo!();
//!     assert_eq!(html.0, "<div><div>foo</div></div>");
//! }
//! ```
//!
//! Or match expressions:
//! ```rust
//! #[cfg(feature = "html_escape")]
//! {
//!     use origami_engine::comp;
//!
//!     let text = "foo";
//!     comp! {
//!         foo =>
//!         div {
//!             match text; noescape {
//!                 "foo" => {
//!                     "<div>foo</div>" // Inherited, this will not be escaped
//!                 },
//!                 _ => escape {
//!                     "<div>foo</div>" // This will be escaped
//!                 }
//!             }
//!         }
//!     }
//!
//!     let html = foo!();
//!     assert_eq!(html.0, "<div><div>foo</div></div>");
//! }
//! ```

pub use origami_macros::anon;
pub use origami_macros::comp;

#[derive(Debug, Clone)]
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
