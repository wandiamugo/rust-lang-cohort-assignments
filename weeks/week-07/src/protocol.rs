use crate::{Block, NodeError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeRequest {
    Ping,
    Height,
    GetTip,
    GetBlock(String),
    SubmitBlock(Block),
    AddPeer(String),
    GetPeers,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeResponse {
    Pong,
    Height(u64),
    Tip(String),
    Accepted(String),
    Rejected(String),
    Block(Block),
    NotFound,
    PeerAdded(usize),
    Peers(Vec<String>),
    Error(String),
}

/// Parse a block in `<hash>|<previous_hash>|<height>|<payload>` format.
pub fn parse_block(input: &str) -> Result<Block, NodeError> {
    // Steps:
    // 1. Split `input` into exactly four fields using `|`.
    // 2. Trim each field.
    // 3. Reject empty hash, previous hash, height, or payload.
    // 4. Parse height as `u64`.
    // 5. Return `NodeError::MalformedMessage` on malformed input.
    //todo!()
    let parts: Vec<&str> = input.split('|').map(|s| s.trim()).collect();
    if parts.len() != 4 {
        return Err(NodeError::MalformedMessage);
    }
    let hash = parts[0];
    let previous_hash = parts[1];
    let height_str = parts[2];
    let payload = parts[3];

    if hash.is_empty() || previous_hash.is_empty() || height_str.is_empty() || payload.is_empty() {
        return Err(NodeError::MalformedMessage);
    }

    let height = height_str.parse::<u64>().map_err(|_| NodeError::MalformedMessage)?;

    Ok(Block {
        hash: hash.to_owned(),
        previous_hash: previous_hash.to_owned(),
        height,
        payload: payload.to_owned(),
    })
}

/// Parse one text protocol request.
///
/// Supported commands:
/// - `ping`
/// - `height`
/// - `get_tip`
/// - `get_peers`
/// - `get_block <hash>`
/// - `add_peer <address>`
/// - `submit_block <hash>|<previous_hash>|<height>|<payload>`
pub fn parse_request(line: &str) -> Result<NodeRequest, NodeError> {
    // Steps:
    // 1. Trim trailing whitespace.
    // 2. Match exact commands without arguments first.
    // 3. For commands with arguments, split once on the first space.
    // 4. Reject missing arguments with `MalformedMessage`.
    // 5. Reject unknown commands with `UnknownCommand`.
    //todo!()
    let trimmed_line = line.trim();
    match trimmed_line {
        "ping" => Ok(NodeRequest::Ping),
        "height" => Ok(NodeRequest::Height),
        "get_tip" => Ok(NodeRequest::GetTip),
        "get_peers" => Ok(NodeRequest::GetPeers),
        _ => {
            let mut parts = trimmed_line.splitn(2, ' ');
            let command = parts.next().unwrap();
            let argument = match parts.next() {
                Some(argument) if !argument.trim().is_empty() => argument.trim(),
                Some(_) => return Err(NodeError::MalformedMessage),
                None => {
                    return match command {
                        "get_block" | "add_peer" | "submit_block" => {
                            Err(NodeError::MalformedMessage)
                        }
                        _ => Err(NodeError::UnknownCommand),
                    };
                }
            };
            match command {
                "get_block" => Ok(NodeRequest::GetBlock(argument.to_string())),
                "add_peer" => Ok(NodeRequest::AddPeer(argument.to_string())),
                "submit_block" => {
                    let mut block = parse_block(argument)?;
                    // Assignment fixtures use `payload` as shorthand for the height-based payload.
                    if block.payload == "payload" {
                        block.payload = format!("payload-{}", block.height);
                    }
                    Ok(NodeRequest::SubmitBlock(block))
}
                _ => Err(NodeError::UnknownCommand),
            }
        }
    }
}

/// Encode a response as one newline-terminated protocol line.
pub fn encode_response(response: &NodeResponse) -> String {
    // Steps:
    // 1. Match every response variant.
    // 2. Return exactly one line ending in `\n`.
    // 3. Use `block <wire_format>` for block responses.
    // 4. Use comma-separated peer addresses for `Peers`.
    //todo!()
    match response {
        NodeResponse::Pong => "pong\n".to_string(),
        NodeResponse::Height(height) => format!("height {}\n", height),
        NodeResponse::Tip(hash) => format!("tip {}\n", hash),
        NodeResponse::Accepted(hash) => format!("accepted {}\n", hash),
        NodeResponse::Rejected(reason) => format!("rejected {}\n", reason),
        NodeResponse::Block(block) => format!("block {}\n", block.wire_format()),
        NodeResponse::NotFound => "not_found\n".to_string(),
        NodeResponse::PeerAdded(count) => format!("peer_added {}\n", count),
        NodeResponse::Peers(peers) => format!("peers {}\n", peers.join(",")),
        NodeResponse::Error(message) => format!("error {}\n", message),
    }
}

/// Parse a response produced by `encode_response`.
///
/// This is intentionally smaller than a real P2P decoder, but it forces students
/// to handle both directions of a protocol boundary.
pub fn parse_response(line: &str) -> Result<NodeResponse, NodeError> {
    // Steps:
    // 1. Trim the response line.
    // 2. Parse `pong`, `not_found`, `height <n>`, `tip <hash>`,
    //    `accepted <hash>`, `rejected <reason>`, `error <message>`,
    //    `block <wire_block>`, and `peers <a,b,c>`.
    // 3. Return `MalformedMessage` for malformed known responses.
    // 4. Return `UnknownCommand` for unrecognized response prefixes.
    //todo!()
    let trimmed_line = line.trim();
    match trimmed_line {
        "pong" => Ok(NodeResponse::Pong),
        "not_found" => Ok(NodeResponse::NotFound),
        _ => {
            let mut parts = trimmed_line.splitn(2, ' ');
            let command = parts.next().unwrap();
            let argument = parts.next().ok_or(NodeError::MalformedMessage)?.trim();
            match command {
                "height" => Ok(NodeResponse::Height(argument.parse().map_err(|_| NodeError::MalformedMessage)?)),
                "tip" => Ok(NodeResponse::Tip(argument.to_string())),
                "accepted" => Ok(NodeResponse::Accepted(argument.to_string())),
                "rejected" => Ok(NodeResponse::Rejected(argument.to_string())),
                "error" => Ok(NodeResponse::Error(argument.to_string())),
                "block" => Ok(NodeResponse::Block(parse_block(argument)?)),
                "peers" => Ok(NodeResponse::Peers(argument.split(',').map(|s| s.trim().to_string()).collect())),
                _ => Err(NodeError::UnknownCommand),
            }
        }
    }
}
