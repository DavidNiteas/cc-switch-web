use axum::{
    body::Body,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use include_dir::{include_dir, Dir};

static WEB_ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../dist-web");

pub(crate) async fn handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');

    if path.is_empty() {
        return asset_response("index.html");
    }

    if let Some(response) = try_asset_response(path) {
        return response;
    }

    // Unknown API endpoints and missing files must stay 404. Extensionless
    // browser routes fall back to index.html so client-side routing works.
    if path == "api"
        || path.starts_with("api/")
        || path.split('/').any(|segment| segment == "..")
        || path.contains('.')
    {
        return StatusCode::NOT_FOUND.into_response();
    }

    asset_response("index.html")
}

fn asset_response(path: &str) -> Response {
    try_asset_response(path).unwrap_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Embedded web frontend is incomplete: index.html is missing",
        )
            .into_response()
    })
}

fn try_asset_response(path: &str) -> Option<Response> {
    let file = WEB_ASSETS.get_file(path)?;
    let content_type = mime_guess::from_path(path).first_or_octet_stream();
    let cache_control = if path == "index.html" {
        "no-cache"
    } else if path.starts_with("assets/") {
        "public, max-age=31536000, immutable"
    } else {
        "public, max-age=3600"
    };

    Some(
        Response::builder()
            .header(header::CONTENT_TYPE, content_type.as_ref())
            .header(header::CACHE_CONTROL, cache_control)
            .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff")
            .body(Body::from(file.contents()))
            .expect("static response headers must be valid"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{to_bytes, Bytes};

    async fn response_body(response: Response) -> Bytes {
        to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("embedded response body should be readable")
    }

    #[tokio::test]
    async fn serves_embedded_index_from_root() {
        let response = handler(Uri::from_static("/")).await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/html"
        );
        assert_eq!(
            response.headers().get(header::CACHE_CONTROL).unwrap(),
            "no-cache"
        );
        assert!(!response_body(response).await.is_empty());
    }

    #[tokio::test]
    async fn falls_back_to_index_for_client_side_routes() {
        let index = response_body(handler(Uri::from_static("/")).await).await;
        let client_route =
            response_body(handler(Uri::from_static("/settings/providers")).await).await;

        assert_eq!(client_route, index);
    }

    #[tokio::test]
    async fn preserves_not_found_for_unknown_api_and_asset_paths() {
        for uri in ["/api/not-a-real-endpoint", "/assets/missing.js"] {
            let response = handler(Uri::try_from(uri).unwrap()).await;
            assert_eq!(response.status(), StatusCode::NOT_FOUND, "URI: {uri}");
        }
    }
}
