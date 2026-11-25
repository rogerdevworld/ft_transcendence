"use client";

import { useEffect, useState } from "react";

interface TrackAsset {
  file_size: number;
  chunk_size: number;
  total_chunks: number;
  mime_type: string;
}

interface Track {
  id: number;
  title: string;
  artist: string;
  media_kind: string;
  source_kind: string;
  external_source: string | null;
  external_id: string | null;
  is_ad: boolean;
  asset: TrackAsset | null;
}

export default function TestPlayerPage() {
  const [tracks, setTracks] = useState<Track[]>([]);
  const [selected, setSelected] = useState<Track | null>(null);
  const [chunkInfo, setChunkInfo] = useState<string>("");

  useEffect(() => {
    fetch("/api/tracks")
      .then((res) => res.json())
      .then((data: Track[]) => {
        setTracks(data);
        if (data.length > 0) setSelected(data[0]);
      })
      .catch((err) => console.error("Error cargando /api/tracks:", err));
  }, []);

  async function testChunk(trackId: number, index: number) {
    setChunkInfo("Descargando chunk...");
    const res = await fetch(`/api/tracks/${trackId}/chunks/${index}`);
    if (!res.ok) {
      const errText = await res.text();
      setChunkInfo(`Error (${res.status}): ${errText}`);
      return;
    }
    const blob = await res.blob();
    const chunkIdx = res.headers.get("x-chunk-index");
    const totalChunks = res.headers.get("x-total-chunks");
    setChunkInfo(
      `Chunk ${chunkIdx} de ${totalChunks} -> ${blob.size} bytes (${res.headers.get("content-type")})`
    );
  }

  const streamUrl = selected ? `/api/tracks/${selected.id}/stream` : "";

  return (
    <main style={{ maxWidth: 720, margin: "2rem auto", fontFamily: "sans-serif" }}>
      <h1>Music Room — Test Player Básico</h1>

      <h2>Catálogo (/api/tracks)</h2>
      <ul>
        {tracks.map((t) => (
          <li key={t.id} style={{ marginBottom: "0.5rem" }}>
            <button onClick={() => { setSelected(t); setChunkInfo(""); }}>
              #{t.id} — {t.title} ({t.artist}) [{t.media_kind} / {t.source_kind}]
            </button>
          </li>
        ))}
      </ul>

      {selected && (
        <section style={{ marginTop: "1.5rem", padding: "1rem", border: "1px solid #ccc" }}>
          <h3>Reproduciendo: {selected.title}</h3>
          <p>
            Tipo: <strong>{selected.media_kind}</strong> | Fuente:{" "}
            <strong>{selected.source_kind}</strong>
            {selected.is_ad ? " | ANUNCIO (FREE)" : ""}
          </p>

          {selected.source_kind === "LOCAL" && selected.asset ? (
            <>
              {selected.media_kind === "VIDEO" ? (
                <video
                  key={streamUrl}
                  src={streamUrl}
                  controls
                  style={{ width: "100%", maxHeight: 360 }}
                />
              ) : (
                <audio
                  key={streamUrl}
                  src={streamUrl}
                  controls
                  style={{ width: "100%" }}
                />
              )}

              <div style={{ marginTop: "1rem" }}>
                <p>
                  Total chunks: <strong>{selected.asset.total_chunks}</strong> (
                  {selected.asset.chunk_size} bytes c/u)
                </p>
                {Array.from({ length: selected.asset.total_chunks }).map((_, idx) => (
                  <button
                    key={idx}
                    onClick={() => testChunk(selected.id, idx)}
                    style={{ marginRight: "0.5rem", marginBottom: "0.5rem" }}
                  >
                    Chunk {idx}
                  </button>
                ))}
                {chunkInfo && <p><code>{chunkInfo}</code></p>}
              </div>
            </>
          ) : (
            <p>
              Pista externa ({selected.external_source ?? "API"} — ID:{" "}
              {selected.external_id ?? "N/A"}). Aquí irá el reproductor/SDK externo.
            </p>
          )}
        </section>
      )}
    </main>
  );
}