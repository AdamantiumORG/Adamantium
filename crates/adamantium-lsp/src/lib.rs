//! Protocol boundary for the future Adamantium language server.
//!
//! This crate owns LSP wire types and advertised capabilities. It deliberately
//! does not depend on the CLI or a native backend. Compiler diagnostics,
//! formatting, and symbol information will be connected through crate APIs.

use serde::{Deserialize, Serialize};

pub const SERVER_NAME: &str = "adamantium-lsp";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerCapabilities {
    pub text_document_sync: u8,
    pub document_formatting_provider: bool,
}

pub const fn capabilities() -> ServerCapabilities {
    ServerCapabilities {
        // LSP TextDocumentSyncKind.Full. Incremental synchronization can be
        // enabled after the compiler accepts a persistent source database.
        text_document_sync: 1,
        document_formatting_provider: true,
    }
}

pub fn initialize_result() -> serde_json::Value {
    serde_json::json!({
        "capabilities": capabilities(),
        "serverInfo": {
            "name": SERVER_NAME,
            "version": env!("CARGO_PKG_VERSION")
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_result_uses_stable_lsp_field_names() {
        let result = initialize_result();
        assert_eq!(result["serverInfo"]["name"], SERVER_NAME);
        assert_eq!(result["capabilities"]["textDocumentSync"], 1);
        assert_eq!(result["capabilities"]["documentFormattingProvider"], true);
    }

    #[test]
    fn positions_round_trip_through_json() {
        let position = Position {
            line: 12,
            character: 7,
        };
        let json = serde_json::to_string(&position).unwrap();
        assert_eq!(serde_json::from_str::<Position>(&json).unwrap(), position);
    }
}
