use tokio::sync::{mpsc, oneshot};

use crate::{NodeError, NodeRequest, NodeResponse, NodeState};

pub struct NodeCommand {
    pub request: NodeRequest,
    pub response: oneshot::Sender<NodeResponse>,
}

/// Apply one request to node state and return a response.
pub fn handle_request(state: &mut NodeState, request: NodeRequest) -> NodeResponse {
    // Steps:
    // 1. Return `Pong` for `Ping`.
    // 2. Return current height or tip for height/tip requests.
    // 3. Look up blocks for `GetBlock`.
    // 4. Add peers for `AddPeer`.
    // 5. Validate and append blocks for `SubmitBlock`.
    // 6. Never panic for malformed state; return `Rejected` or `Error`.
    //todo!()
    match request {
        NodeRequest::Ping => NodeResponse::Pong,
        NodeRequest::Height => NodeResponse::Height(state.height()),
        NodeRequest::GetTip => match state.tip_hash() {
            Some(hash) => NodeResponse::Tip(hash.to_string()),
            None => NodeResponse::Error("No tip available".to_string()),
        },
        NodeRequest::GetBlock(hash) => match state.chain.iter().find(|block| block.hash == hash) {
            Some(block) => NodeResponse::Block(block.clone()),
            None => NodeResponse::NotFound,
        },
        NodeRequest::SubmitBlock(block) => {
            match state.chain.last() {
                Some(tip) if block.previous_hash == tip.hash && block.height == tip.height + 1 => {
                    state.chain.push(block.clone());
                    NodeResponse::Accepted(block.hash)
                }
                _ => NodeResponse::Rejected("Invalid block".to_string()),
            }
        }
        NodeRequest::AddPeer(address) => {
            let count = state.add_peer(&address);
            NodeResponse::PeerAdded(count)
        }
        NodeRequest::GetPeers => {
            let peers = state.peer_addresses();
            NodeResponse::Peers(peers)
        }
    }
}

/// Run a state manager task that serializes access to `NodeState`.
pub async fn run_state_manager(mut state: NodeState, mut receiver: mpsc::Receiver<NodeCommand>) {
    // Steps:
    // 1. Receive `NodeCommand` values until the channel closes.
    // 2. Handle each request with `handle_request`.
    // 3. Send the response through the command's oneshot sender.
    // 4. Ignore send failures because the caller may have timed out or dropped.
    //todo!()
    while let Some(command) = receiver.recv().await {
        let response = handle_request(&mut state, command.request);
        let _ = command.response.send(response);
    }
}

/// Spawn a state manager and return a bounded command sender.
pub fn spawn_state_manager(state: NodeState, capacity: usize) -> mpsc::Sender<NodeCommand> {
    // Steps:
    // 1. Create a bounded channel with the requested capacity.
    // 2. Spawn `run_state_manager(state, receiver)` on Tokio.
    // 3. Return the sender.
    let (sender, receiver) = mpsc::channel(capacity);
    tokio::spawn(run_state_manager(state, receiver));
    sender
}

/// Send one request to the state manager and wait for its response.
pub async fn send_request(
    sender: &mpsc::Sender<NodeCommand>,
    request: NodeRequest,
) -> Result<NodeResponse, NodeError> {
    // Steps:
    // 1. Create a oneshot response channel.
    // 2. Send `NodeCommand { request, response }` through `sender`.
    // 3. Map closed mpsc or oneshot channels to `NodeError::ChannelClosed`.
    // 4. Return the node response.
    let (response_sender, response_receiver) = oneshot::channel();
    let command = NodeCommand { request, response: response_sender };
    sender.send(command).await.map_err(|_| NodeError::ChannelClosed)?;
    response_receiver.await.map_err(|_| NodeError::ChannelClosed)
}
