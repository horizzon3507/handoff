use maud::{html, Markup, DOCTYPE};

pub fn render_room_page(share_url: &str, qr_url: &str) -> Markup {
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
                                    p.file-count { "0 files in this room" }
                                }
                                div.size-readout {
                                    span.readout-label { "IN ROOM" }
                                    span { "0 B" }
                                }
                            }
                            div.drop-zone {
                                span.drop-icon aria-hidden="true" { span {} }
                                p.drop-title { "Select files to share" }
                                p.drop-detail { "Files appear for everyone in this room." }
                                button.choose-button type="button" disabled { "Choose files" }
                            }
                            div.queue-heading { span { "ROOM CONTENTS" } span { "512 MB per file" } }
                            div.file-list { p.empty-state { "Nothing here yet. Add a file to begin." } }
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
