use std::sync::Arc;
use axum::{
    Json,
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::{AppState, models::UserDetails};

/// POST /user
/// Body: JSON UserDetails
/// Upserts the record in SQLite.
pub async fn upsert_user(
    State(state): State<Arc<AppState>>,
    Json(user): Json<UserDetails>,
) -> impl IntoResponse {
    tracing::info!("Received user record: {}", user.file_name);

    match state.db.upsert_user(&user) {
        Ok(_) => (StatusCode::OK, "User saved").into_response(),
        Err(e) => {
            tracing::error!("DB error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// POST /image
/// Multipart form with two fields:
///   - "metadata": JSON-encoded UserDetails
///   - "image":    raw PNG bytes
///
/// The image is only stored if consent == true.
pub async fn upload_image(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut metadata: Option<UserDetails> = None;
    let mut image_bytes: Option<(String, Vec<u8>)> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name() {
            Some("metadata") => {
                let text = match field.text().await {
                    Ok(t) => t,
                    Err(e) => return (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
                };
                match serde_json::from_str::<UserDetails>(&text) {
                    Ok(u) => metadata = Some(u),
                    Err(e) => return (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
                }
            }
            Some("image") => {
                let file_name = field
                    .file_name()
                    .unwrap_or("image.png")
                    .to_string();
                let bytes = match field.bytes().await {
                    Ok(b) => b.to_vec(),
                    Err(e) => return (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
                };
                image_bytes = Some((file_name, bytes));
            }
            _ => {}
        }
    }

    let user = match metadata {
        Some(u) => u,
        None => return (StatusCode::BAD_REQUEST, "Missing metadata field").into_response(),
    };

    let (_, bytes) = match image_bytes {
        Some(b) => b,
        None => return (StatusCode::BAD_REQUEST, "Missing image field").into_response(),
    };

    if !user.consent {
        tracing::info!("Skipping image upload — consent not given for {}", user.file_name);
        return (StatusCode::OK, "No consent — image not stored").into_response();
    }

    // Store to MinIO using the GUID filename as the object key
    let key = format!("{}.png", user.file_name);
    if let Err(e) = state.storage.put_image(&key, bytes).await {
        tracing::error!("MinIO error: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }

    // Upsert user record alongside the image
    if let Err(e) = state.db.upsert_user(&user) {
        tracing::error!("DB error after image upload: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }

    tracing::info!("Image + user record stored for {}", user.file_name);
    (StatusCode::OK, "Image and user record stored").into_response()
}
