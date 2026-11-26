"use client";

import "./api-doc.css";
import { useMemo, useState } from "react";
import { API_ENDPOINTS } from "./api-docs.data";
import { ApiEndpointCard } from "./ApiEndpointCard";

export default function ApiDocsPage() {
  const [query, setQuery] = useState("");
  const [selectedModule, setSelectedModule] = useState<string>("all");

  const modules = useMemo(
    () => ["all", ...Array.from(new Set(API_ENDPOINTS.map((e) => e.module)))],
    []
  );

  const filtered = useMemo(() => {
    return API_ENDPOINTS.filter((ep) => {
      const matchesModule = selectedModule === "all" || ep.module === selectedModule;
      const q = query.trim().toLowerCase();
      const matchesQuery =
        !q ||
        ep.path.toLowerCase().includes(q) ||
        ep.summary.toLowerCase().includes(q) ||
        ep.description.toLowerCase().includes(q) ||
        ep.module.toLowerCase().includes(q);
      return matchesModule && matchesQuery;
    });
  }, [query, selectedModule]);

  return (
    <div className="min-h-screen bg-gradient-to-b from-zinc-950 via-zinc-950 to-emerald-950/20 text-zinc-100">
      <main className="mx-auto max-w-6xl px-6 py-12">
        <header className="mb-8 rounded-2xl border border-white/10 bg-zinc-900/60 p-6 shadow-xl shadow-black/30 backdrop-blur-md">
          <div className="flex flex-wrap items-center justify-between gap-4">
            <div className="space-y-2">
              <span className="inline-flex items-center gap-2 rounded-full border border-emerald-400/30 bg-emerald-400/10 px-3 py-1 font-mono text-xs text-emerald-300">
                <span className="h-2 w-2 rounded-full bg-emerald-400" />
                API Reference · /api-doc
              </span>
              <h1 className="text-3xl font-bold tracking-tight text-white sm:text-4xl">
                Documentación viva del Backend
              </h1>
              <p className="max-w-2xl text-sm text-zinc-400">
                Explora los endpoints de <strong className="text-zinc-200">tracks</strong>,{" "}
                <strong className="text-zinc-200">rooms</strong>,{" "}
                <strong className="text-zinc-200">users</strong>,{" "}
                <strong className="text-zinc-200">artists</strong>,{" "}
                <strong className="text-zinc-200">admins</strong>,{" "}
                <strong className="text-zinc-200">studio</strong>,{" "}
                <strong className="text-zinc-200">uploads</strong> y{" "}
                <strong className="text-zinc-200">webrtc</strong> en{" "}
                <code className="rounded bg-zinc-950 px-1.5 py-0.5 text-emerald-300">
                  app/api-doc
                </code>
                .
              </p>
            </div>

            <div className="grid grid-cols-2 gap-3 text-center sm:grid-cols-3">
              <div className="rounded-xl border border-white/10 bg-zinc-950/80 px-4 py-3">
                <p className="font-mono text-xl font-bold text-emerald-300">
                  {API_ENDPOINTS.length}
                </p>
                <p className="text-xs text-zinc-400">Endpoints</p>
              </div>
              <div className="rounded-xl border border-white/10 bg-zinc-950/80 px-4 py-3">
                <p className="font-mono text-xl font-bold text-sky-300">
                  {modules.length - 1}
                </p>
                <p className="text-xs text-zinc-400">Módulos</p>
              </div>
              <div className="col-span-2 rounded-xl border border-white/10 bg-zinc-950/80 px-4 py-3 sm:col-span-1">
                <p className="font-mono text-xl font-bold text-amber-300">{filtered.length}</p>
                <p className="text-xs text-zinc-400">Visibles</p>
              </div>
            </div>
          </div>
        </header>

        <section className="mb-6 flex flex-col gap-4 rounded-2xl border border-white/10 bg-zinc-900/40 p-4 lg:flex-row lg:items-center lg:justify-between">
          <div className="flex flex-wrap gap-2">
            {modules.map((mod) => (
              <button
                key={mod}
                type="button"
                onClick={() => setSelectedModule(mod)}
                className={`rounded-xl border px-3.5 py-1.5 font-mono text-xs font-medium transition ${
                  selectedModule === mod
                    ? "border-emerald-400/60 bg-emerald-400/15 text-emerald-200 shadow-sm shadow-emerald-500/20"
                    : "border-white/10 bg-zinc-950/70 text-zinc-400 hover:border-white/20 hover:text-zinc-100"
                }`}
              >
                {mod === "all" ? "todos" : mod}
              </button>
            ))}
          </div>

          <input
            type="search"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Buscar ruta, módulo o descripción..."
            className="w-full rounded-xl border border-white/10 bg-zinc-950/90 px-4 py-2 text-sm text-zinc-100 placeholder:text-zinc-500 focus:border-emerald-400/50 focus:outline-none lg:w-72"
          />
        </section>

        <section className="grid gap-4">
          {filtered.map((endpoint) => (
            <ApiEndpointCard key={endpoint.id} endpoint={endpoint} />
          ))}
          {filtered.length === 0 && (
            <div className="rounded-2xl border border-dashed border-white/15 bg-zinc-900/40 p-10 text-center">
              <p className="text-sm font-medium text-zinc-300">
                No hay endpoints que coincidan con el filtro actual.
              </p>
              <button
                type="button"
                onClick={() => {
                  setQuery("");
                  setSelectedModule("all");
                }}
                className="mt-3 rounded-lg border border-emerald-400/40 bg-emerald-400/10 px-3 py-1.5 text-xs font-medium text-emerald-300 hover:bg-emerald-400/20"
              >
                Restablecer filtros
              </button>
            </div>
          )}
        </section>
      </main>
    </div>
  );
}