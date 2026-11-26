"use client";

import "./api-doc/api-doc.css";
import React, { Suspense, useEffect, useRef, useState } from "react";
import Link from "next/link";
import { useParams } from "next/navigation";

interface NowPlaying {
  track_id: number;
  started_at_ms: number;
  is_playing: boolean;
}

interface QueueItem {
  track_id: number;
  votes: number;
}

function ApiCornerButton({
  href = "/api-doc",
  label = "API Explorer",
}: {
  href?: string;
  label?: string;
}) {
  return (
    <Link
      href={href}
      aria-label={`Ir a ${label}`}
      className="group fixed bottom-6 right-6 z-50 inline-flex items-center gap-2.5 rounded-full border border-emerald-400/40 bg-zinc-950/90 px-4 py-2.5 text-sm font-medium text-zinc-100 shadow-lg shadow-emerald-500/10 backdrop-blur-md transition-all duration-200 hover:-translate-y-0.5 hover:border-emerald-300/70 hover:bg-zinc-900 hover:shadow-emerald-400/20 active:translate-y-0"
    >
      <span className="relative flex h-2.5 w-2.5">
        <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-emerald-400 opacity-75" />
        <span className="relative inline-flex h-2.5 w-2.5 rounded-full bg-emerald-400" />
      </span>

      <span className="font-mono text-xs tracking-wide text-emerald-300">
        {label}
      </span>

      <span
        aria-hidden="true"
        className="text-zinc-400 transition-transform duration-200 group-hover:translate-x-0.5 group-hover:text-emerald-300"
      >
        →
      </span>
    </Link>
  );
}

function RoomContent() {
  // Lee el {id} de la URL, por ejemplo /rooms/1 -> roomId = 1
  const params = useParams<{ id: string }>();
  const roomId = Number(params?.id || 1);

  // Usuario de prueba por ahora (luego vendrá de auth/sesión)
  const userId = 1;

  // Estados principales de la sala
  const [connected, setConnected] = useState(false);
  const [nowPlaying, setNowPlaying] = useState<NowPlaying | null>(null);
  const [queue, setQueue] = useState<QueueItem[]>([]);
  const [trackIdInput, setTrackIdInput] = useState("4");
  const [statusMsg, setStatusMsg] = useState("");

  const audioRef = useRef<HTMLAudioElement | null>(null);

  // Helper para ordenar la cola: más votos arriba
  function sortByVotes(items: QueueItem[]) {
    return [...items].sort((a, b) => b.votes - a.votes || a.track_id - b.track_id);
  }

  // 1. Conexión en tiempo real por WebSocket
  useEffect(() => {
    const wsUrl = `wss://${window.location.host}/api/rooms/${roomId}/ws`;
    const ws = new WebSocket(wsUrl);

    ws.onopen = () => {
      setConnected(true);
      setStatusMsg("Conectado al WebSocket de la sala");
    };

    ws.onclose = () => {
      setConnected(false);
      setStatusMsg("WebSocket desconectado");
    };

    ws.onmessage = (event) => {
      const msg = JSON.parse(event.data);

      if (msg.type === "room_snapshot") {
        setNowPlaying(msg.payload.now_playing ?? null);
        setQueue(sortByVotes(msg.payload.queue ?? []));
      } else if (msg.type === "track_queued") {
        setQueue((prev) => {
          const exists = prev.some((t) => t.track_id === msg.payload.track_id);
          if (exists) return prev;
          return sortByVotes([
            ...prev,
            { track_id: msg.payload.track_id, votes: msg.payload.votes ?? 0 },
          ]);
        });
      } else if (msg.type === "vote_updated") {
        setQueue((prev) => {
          const exists = prev.some((t) => t.track_id === msg.payload.track_id);
          const nextQueue = exists
            ? prev.map((t) =>
                t.track_id === msg.payload.track_id ? { ...t, votes: msg.payload.votes } : t
              )
            : [...prev, { track_id: msg.payload.track_id, votes: msg.payload.votes }];
          return sortByVotes(nextQueue);
        });
      } else if (msg.type === "now_playing_changed") {
        setNowPlaying(msg.payload.now_playing ?? null);
        setQueue(sortByVotes(msg.payload.queue ?? []));
      }
    };

    return () => {
      ws.close();
    };
  }, [roomId]);

  // 2. Sincronizar el reproductor <audio> cuando cambia nowPlaying
  useEffect(() => {
    const audio = audioRef.current;
    if (!audio) return;

    if (!nowPlaying) {
      audio.pause();
      audio.removeAttribute("src");
      return;
    }

    const url = `/api/tracks/${nowPlaying.track_id}/stream`;
    const elapsedSeconds = Math.max(0, (Date.now() - nowPlaying.started_at_ms) / 1000);

    const applySync = () => {
      audio.currentTime = elapsedSeconds;
      if (nowPlaying.is_playing) {
        audio.play().catch(() => {
          // Si el navegador bloquea autoplay hasta que el usuario haga clic, no rompe nada
        });
      } else {
        audio.pause();
      }
    };

    if (!audio.src.endsWith(url)) {
      audio.src = url;
      audio.addEventListener("loadedmetadata", applySync, { once: true });
      audio.load();
    } else {
      applySync();
    }
  }, [nowPlaying]);

  // 3. Acciones hacia tu backend en Rust
  async function handleEnqueue(e: React.FormEvent) {
    e.preventDefault();
    const track_id = Number(trackIdInput);
    if (!track_id) return;

    const res = await fetch(`/api/rooms/${roomId}/queue`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ track_id }),
    });

    setStatusMsg(res.ok ? `Pista ${track_id} enviada a la cola` : `Error al encolar (${res.status})`);
  }

  async function handleVote(trackId: number) {
    const res = await fetch(`/api/rooms/${roomId}/queue/${trackId}/vote`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ user_id: userId }),
    });

    setStatusMsg(res.ok ? `Voto actualizado en pista ${trackId}` : `Error al votar (${res.status})`);
  }

  async function handleNext() {
    const res = await fetch(`/api/rooms/${roomId}/next`, {
      method: "POST",
    });

    setStatusMsg(res.ok ? "Avanzado a la siguiente pista" : `Error en /next (${res.status})`);
  }

  return (
    <main style={{ maxWidth: 640, margin: "2rem auto", fontFamily: "sans-serif" }}>
      <h1>Music Room #{roomId}</h1>
      <p>
        Estado WS: <strong>{connected ? "Conectado" : "Desconectado"}</strong>
        {statusMsg ? ` — ${statusMsg}` : ""}
      </p>

      <hr />

      <section>
        <h2>Sonando ahora</h2>
        {nowPlaying ? (
          <p>
            Track ID: <strong>{nowPlaying.track_id}</strong> |{" "}
            {nowPlaying.is_playing ? "Reproduciendo" : "Pausado"}
          </p>
        ) : (
          <p>Ninguna canción reproduciéndose ahora mismo.</p>
        )}

        <audio ref={audioRef} controls style={{ width: "100%" }} />

        <div style={{ marginTop: "0.75rem" }}>
          <button type="button" onClick={handleNext}>
            Pasar a la más votada (/next)
          </button>
        </div>
      </section>

      <hr />

      <section>
        <h2>Encolar pista</h2>
        <form onSubmit={handleEnqueue} style={{ display: "flex", gap: "0.5rem" }}>
          <input
            type="number"
            value={trackIdInput}
            onChange={(e) => setTrackIdInput(e.target.value)}
            min={1}
          />
          <button type="submit">Encolar</button>
        </form>
      </section>

      <hr />

      <section>
        <h2>Cola de reproducción</h2>
        {queue.length === 0 ? (
          <p>La cola está vacía.</p>
        ) : (
          <ul>
            {queue.map((item) => (
              <li key={item.track_id} style={{ marginBottom: "0.5rem" }}>
                Track <strong>#{item.track_id}</strong> — <strong>{item.votes}</strong> votos{" "}
                <button type="button" onClick={() => handleVote(item.track_id)}>
                  Votar / Quitar voto
                </button>
              </li>
            ))}
          </ul>
        )}
      </section>

      <ApiCornerButton href="/api" label="API Explorer" />
    </main>
  );
}

export default function RoomPage() {
  return (
    <Suspense fallback={<p style={{ maxWidth: 640, margin: "2rem auto" }}>Cargando sala...</p>}>
      <RoomContent />
    </Suspense>
  );
}