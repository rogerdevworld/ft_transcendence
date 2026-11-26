use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use redis::AsyncCommands;
use serde::Serialize;
use tokio::sync::broadcast;

use crate::features::music_rooms::dto::{ QueuedTrack, NowPlaying};
use crate::features::webrtc_signaling::dto::SignalMessage;

pub mod dto;
pub mod handlers;
pub mod model;
pub mod routes;

const ROOM_CHANNEL_CAPACITY: usize = 256;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum RoomWsEvent {
    #[serde(rename = "room_snapshot")]
    RoomSnapshot {
        room_id: i64,
        now_playing: Option<NowPlaying>,
        queue: Vec<QueuedTrack>,
    },
    #[serde(rename = "member_joined")]
    MemberJoined { room_id: i64 },
    #[serde(rename = "member_left")]
    MemberLeft { room_id: i64 },
    #[serde(rename = "track_queued")]
    TrackQueued {
        room_id: i64,
        track_id: i64,
        votes: i64,
    },
    #[serde(rename = "vote_updated")]
    VoteUpdated {
        room_id: i64,
        track_id: i64,
        user_id: i64,
        voted: bool,
        votes: i64,
    },
    #[serde(rename = "now_playing_changed")]
    NowPlayingChanged {
        room_id: i64,
        now_playing: Option<NowPlaying>,
        queue: Vec<QueuedTrack>,
    },
    WebrtcSignal(SignalMessage),
}

#[derive(Clone, Default)]
pub struct RoomHub {
    channels: Arc<RwLock<HashMap<i64, broadcast::Sender<String>>>>,
}

impl RoomHub {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn send_webrtc_signal(&self, room_id: i64, signal: SignalMessage) {
        self.publish(room_id, RoomWsEvent::WebrtcSignal(signal));
    }
    pub fn subscribe(&self, room_id: i64) -> broadcast::Receiver<String> {
        let sender = {
            let mut guard = self.channels.write().expect("RoomHub lock poisoned");
            guard
                .entry(room_id)
                .or_insert_with(|| {
                    let (tx, _) = broadcast::channel(ROOM_CHANNEL_CAPACITY);
                    tx
                })
                .clone()
        };
        sender.subscribe()
    }

    pub fn publish(&self, room_id: i64, event: RoomWsEvent) {
        let sender_opt = {
            let guard = self.channels.read().expect("RoomHub lock poisoned");
            guard.get(&room_id).cloned()
        };

        if let Some(tx) = sender_opt {
            if let Ok(json) = serde_json::to_string(&event) {
                let _ = tx.send(json);
            }
        }
    }
}

fn queue_key(room_id: i64) -> String {
    format!("room:{room_id}:queue")
}

fn voters_key(room_id: i64, track_id: i64) -> String {
    format!("room:{room_id}:votes:{track_id}")
}

pub async fn get_queue(
    conn: &mut redis::aio::ConnectionManager,
    room_id: i64,
) -> Result<Vec<QueuedTrack>, redis::RedisError> {
    let raw: Vec<(String, f64)> = conn.zrevrange_withscores(queue_key(room_id), 0, -1).await?;

    let mut queue = Vec::with_capacity(raw.len());
    for (member, score) in raw {
        if let Ok(track_id) = member.parse::<i64>() {
            queue.push(QueuedTrack {
                track_id,
                votes: score as i64,
            });
        }
    }
    Ok(queue)
}

pub async fn enqueue_track_in_redis(
    conn: &mut redis::aio::ConnectionManager,
    room_id: i64,
    track_id: i64,
) -> Result<(bool, i64), redis::RedisError> {
    let key = queue_key(room_id);
    let member = track_id.to_string();

    let inserted: i64 = redis::cmd("ZADD")
        .arg(&key)
        .arg("NX")
        .arg(0_i64)
        .arg(&member)
        .query_async(conn)
        .await?;

    let was_added = inserted == 1;
    let score: Option<f64> = conn.zscore(&key, &member).await?;
    Ok((was_added, score.unwrap_or(0.0) as i64))
}

pub async fn toggle_vote_in_redis(
    conn: &mut redis::aio::ConnectionManager,
    room_id: i64,
    track_id: i64,
    user_id: i64,
) -> Result<(bool, i64), redis::RedisError> {
    let q_key = queue_key(room_id);
    let v_key = voters_key(room_id, track_id);
    let member = track_id.to_string();
    let _: i64 = redis::cmd("ZADD")
        .arg(&q_key)
        .arg("NX")
        .arg(0_i64)
        .arg(&member)
        .query_async(conn)
        .await?;
    let added: i64 = conn.sadd(&v_key, user_id).await?;
    let (voted, delta): (bool, i64) = if added == 1 {
        (true, 1)
    } else {
        let _: i64 = conn.srem(&v_key, user_id).await?;
        (false, -1)
    };
    let new_score: f64 = conn.zincr(&q_key, &member, delta as f64).await?;
    Ok((voted, new_score as i64))
}

fn now_playing_key(room_id: i64) -> String {
    format!("room:{room_id}:now_playing")
}

pub async fn get_now_playing(
    conn: &mut redis::aio::ConnectionManager,
    room_id: i64,
) -> Result<Option<crate::features::music_rooms::dto::NowPlaying>, redis::RedisError> {
    let raw: Option<String> = conn.get(now_playing_key(room_id)).await?;
    match raw {
        Some(json) => Ok(serde_json::from_str(&json).ok()),
        None => Ok(None),
    }
}

pub async fn set_now_playing(
    conn: &mut redis::aio::ConnectionManager,
    room_id: i64,
    now_playing: &crate::features::music_rooms::dto::NowPlaying,
) -> Result<(), redis::RedisError> {
    let json = serde_json::to_string(now_playing).unwrap_or_default();
    let _: () = conn.set(now_playing_key(room_id), json).await?;
    Ok(())
}

pub async fn advance_to_top_voted_track(
    conn: &mut redis::aio::ConnectionManager,
    room_id: i64,
) -> Result<Option<crate::features::music_rooms::dto::NowPlaying>, redis::RedisError> {
    let popped: Vec<(String, f64)> = redis::cmd("ZPOPMAX")
        .arg(queue_key(room_id))
        .arg(1_i64)
        .query_async(conn)
        .await?;

    let Some((member, _score)) = popped.into_iter().next() else {
        return Ok(None);
    };

    let Ok(track_id) = member.parse::<i64>() else {
        return Ok(None);
    };
    let _: i64 = conn.del(voters_key(room_id, track_id)).await?;
    let now_playing = crate::features::music_rooms::dto::NowPlaying {
        track_id,
        started_at_ms: chrono::Utc::now().timestamp_millis(),
        is_playing: true,
    };

    set_now_playing(conn, room_id, &now_playing).await?;
    Ok(Some(now_playing))
}