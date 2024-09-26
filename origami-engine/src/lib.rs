pub use origami_macros::{anon, comp};

#[cfg(feature = "html_escape")]
pub use html_escape::encode_text_to_string;

pub struct Origami(pub String);

#[cfg(feature = "axum")]
use ::axum::response::{Html, IntoResponse, Response};
#[cfg(feature = "axum")]
impl IntoResponse for Origami {
    fn into_response(self) -> Response {
        Html(self.0).into_response()
    }
}
