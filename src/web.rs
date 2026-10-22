use crate::state::AppState;
use crate::storage::SharedFile;
use axum::{
    body::Body,
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Router,
};
use maud::{html, Markup, DOCTYPE};

mod qr;
mod response;
pub use qr::render_qr_svg;
pub use response::with_security_headers;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(home))
        .route("/share/{token}", get(room_page))
        .route("/style.css", get(stylesheet))
        .route("/qr/{token}", get(qr_code))
        .route("/upload/{token}", post(upload_files))
        .route("/files/{token}/{file_id}", get(download_file))
        .layer(DefaultBodyLimit::disable())
        .with_state(state)
}

async fn home(State(state): State<AppState>) -> Response {
    let location = format!("/share/{}", state.token.as_str());
    with_security_headers(Redirect::temporary(&location).into_response())
}

async fn room_page(Path(candidate): Path<String>, State(state): State<AppState>) -> Response {
    if !state.is_room_token(&candidate) {
        return with_security_headers(StatusCode::NOT_FOUND.into_response());
    }

    let files = state.store.list().await;
    let share_url = state.share_url().unwrap_or_default();
    let markup = render_room_page(
        state.token.as_str(),
        &share_url,
        &format!("/qr/{}", state.token.as_str()),
        &files,
    );
    html_response(markup.into_string())
}

async fn stylesheet() -> Response {
    let mut response = Response::new(Body::from(include_str!("../public/style.css")));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/css; charset=utf-8"),
    );
    with_security_headers(response)
}

async fn qr_code(Path(candidate): Path<String>, State(state): State<AppState>) -> Response {
    if !state.is_room_token(&candidate) {
        return with_security_headers(StatusCode::NOT_FOUND.into_response());
    }
    let Some(share_url) = state.share_url() else {
        return with_security_headers(StatusCode::SERVICE_UNAVAILABLE.into_response());
    };
    match render_qr_svg(&share_url) {
        Ok(svg) => {
            let mut response = Response::new(Body::from(svg));
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("image/svg+xml; charset=utf-8"),
            );
            with_security_headers(response)
        }
        Err(_) => with_security_headers(StatusCode::INTERNAL_SERVER_ERROR.into_response()),
    }
}

async fn upload_files(
    Path(candidate): Path<String>,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Response {
    if !state.is_room_token(&candidate) {
        return with_security_headers(StatusCode::NOT_FOUND.into_response());
    }

    let mut uploaded = 0usize;
    loop {
        let field = match multipart.next_field().await {
            Ok(Some(field)) => field,
            Ok(None) => break,
            Err(_) => return with_security_headers(StatusCode::BAD_REQUEST.into_response()),
        };
        if field.file_name().is_none() {
            continue;
        }
        let mut field = field;
        match state.store.save_upload(&mut field).await {
            Ok(_) => uploaded += 1,
            Err(_) => return with_security_headers(StatusCode::PAYLOAD_TOO_LARGE.into_response()),
        }
    }

    let location = format!("/share/{}?uploaded={uploaded}", state.token.as_str());
    let mut response = Redirect::to(&location).into_response();
    *response.status_mut() = StatusCode::SEE_OTHER;
    with_security_headers(response)
}

async fn download_file(
    Path((candidate, file_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Response {
    if !state.is_room_token(&candidate) {
        return with_security_headers(StatusCode::NOT_FOUND.into_response());
    }
    let Ok(file_id) = uuid::Uuid::parse_str(&file_id) else {
        return with_security_headers(StatusCode::NOT_FOUND.into_response());
    };
    let Some(shared_file) = state.store.get(file_id).await else {
        return with_security_headers(StatusCode::NOT_FOUND.into_response());
    };
    let Ok(file) = tokio::fs::File::open(&shared_file.path).await else {
        return with_security_headers(StatusCode::NOT_FOUND.into_response());
    };

    let stream = tokio_util::io::ReaderStream::new(file);
    let mut response = Response::new(Body::from_stream(stream));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(shared_file.content_type()),
    );
    if let Ok(value) = HeaderValue::from_str(&shared_file.content_disposition()) {
        response
            .headers_mut()
            .insert(header::CONTENT_DISPOSITION, value);
    }
    response
        .headers_mut()
        .insert(header::CONTENT_LENGTH, HeaderValue::from(shared_file.size));
    with_security_headers(response)
}

fn html_response(body: String) -> Response {
    let mut response = Response::new(Body::from(body));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    with_security_headers(response)
}

pub fn render_room_page(
    room_token: &str,
    share_url: &str,
    qr_url: &str,
    files: &[SharedFile],
) -> Markup {
    let total_bytes = files.iter().map(|file| file.size).sum::<u64>();
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                meta name="theme-color" content="#171b17";
                title { "Handoff · Local transfer room" }
                link rel="stylesheet" href="/style.css";
            }
            body {
                main.shell {
                    header.window-bar {
                        div.window-title {
                            span.connection-light aria-hidden="true" {}
                            span { "Handoff" }
                            span.bar-separator aria-hidden="true" { "/" }
                            span.window-subtitle { "local transfer room" }
                        }
                        span.session-state { span.state-mark aria-hidden="true" {} "Room open" }
                    }
                    div.workspace {
                        aside.connect-panel aria-labelledby="connect-title" {
                            div.section-heading {
                                span.section-index { "01" }
                                h2 id="connect-title" { "Connect a device" }
                            }
                            p.connect-intro { "Open this room on another device on the same Wi-Fi." }
                            div.qr-frame {
                                img id="qr-code" src=(qr_url) alt="QR code for this transfer room";
                            }
                            label.field-label for="room-link" { "ROOM LINK" }
                            div.link-control {
                                input id="room-link" type="text" readonly value=(share_url) aria-label="Transfer room link";
                            }
                            p.network-note role="status" { "Use this link on the same Wi-Fi." }
                            div.route-diagram aria-label="Files move directly between devices on this network" {
                                div.route-endpoint { span.device-square aria-hidden="true" {} span { "This device" } }
                                span.route-line aria-hidden="true" {}
                                div.route-endpoint { span.device-circle aria-hidden="true" {} span { "Nearby device" } }
                            }
                            p.privacy-note { "The room link is the key. Share it only with devices you trust." }
                        }
                        section.transfer-panel aria-labelledby="transfer-title" {
                            div.transfer-heading {
                                div {
                                    div.section-heading {
                                        span.section-index { "02" }
                                        h2 id="transfer-title" { "Shared files" }
                                    }
                                    p.file-count {
                                        (files.len())
                                        @if files.len() == 1 { " file in this room" } @else { " files in this room" }
                                    }
                                }
                                div.size-readout {
                                    span.readout-label { "IN ROOM" }
                                    span { (format_size(total_bytes)) }
                                }
                            }
                            form.drop-zone method="post" enctype="multipart/form-data" action=(format!("/upload/{room_token}")) {
                                span.drop-icon aria-hidden="true" { span {} }
                                p.drop-title { "Select files to share" }
                                p.drop-detail { "Files appear for everyone in this room." }
                                input type="file" name="files" multiple required aria-label="Select files to share";
                                button.choose-button type="submit" { "Share selected files" }
                            }
                            div.queue-heading { span { "ROOM CONTENTS" } span { "512 MB per file" } }
                            div.file-list {
                                @if files.is_empty() {
                                    p.empty-state { "Nothing here yet. Add a file to begin." }
                                } @else {
                                    @for file in files {
                                        article.file-row {
                                            span.file-marker aria-hidden="true" {}
                                            div.file-details {
                                                p.file-name title=(file.name) { (file.name) }
                                                p.file-meta { (format_size(file.size)) }
                                            }
                                            a.download-link href=(format!("/files/{room_token}/{}", file.id)) {
                                                "Download"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    footer.status-bar {
                        span { "Direct transfer over your local network" }
                        span { "Temporary room · closes with this app" }
                    }
                }
            }
        }
    }
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    let units = ["KB", "MB", "GB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < units.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.1} {}", units[unit])
}
