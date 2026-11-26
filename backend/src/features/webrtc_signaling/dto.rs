use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalScope {
    Room,
    Swarm,
    Direct,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum SignalMessage {
    PeerAnnounce {
        scope: SignalScope,
        room_id: Option<i64>,
        resource_id: Option<String>,
        peer_id: String,
        available_resources: Option<Vec<String>>,
    },
    PeerLeave {
        scope: SignalScope,
        room_id: Option<i64>,
        resource_id: Option<String>,
        peer_id: String,
    },
    Offer {
        scope: SignalScope,
        room_id: Option<i64>,
        resource_id: Option<String>,
        from_peer_id: String,
        to_peer_id: String,
        sdp: String,
    },
    Answer {
        scope: SignalScope,
        room_id: Option<i64>,
        resource_id: Option<String>,
        from_peer_id: String,
        to_peer_id: String,
        sdp: String,
    },
    IceCandidate {
        scope: SignalScope,
        room_id: Option<i64>,
        resource_id: Option<String>,
        from_peer_id: String,
        to_peer_id: String,
        candidate: String,
        sdp_mid: Option<String>,
        sdp_mline_index: Option<u32>,
    },
}

#[derive(Debug, Serialize)]
pub struct SignalingInfoResponse {
    pub room_id: Option<i64>,
    pub signaling_ready: bool,
    pub transport: String,
}