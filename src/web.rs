use maud::{html, Markup, DOCTYPE};
use crate::storage::SharedFile;

mod qr;
mod response;
pub use qr::render_qr_svg;
pub use response::with_security_headers;

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
                            div.drop-zone {
                                span.drop-icon aria-hidden="true" { span {} }
                                p.drop-title { "Select files to share" }
                                p.drop-detail { "Files appear for everyone in this room." }
                                button.choose-button type="button" disabled { "Choose files" }
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
