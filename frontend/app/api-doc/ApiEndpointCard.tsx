"use client";

import "./api-doc.css";
import { useState } from "react";
import type { ApiEndpoint, HttpMethod } from "./api-docs.data";

const METHOD_STYLES: Record<HttpMethod, string> = {
  GET: "border-sky-400/30 bg-sky-500/15 text-sky-300",
  POST: "border-emerald-400/30 bg-emerald-500/15 text-emerald-300",
  PUT: "border-amber-400/30 bg-amber-500/15 text-amber-300",
  PATCH: "border-purple-400/30 bg-purple-500/15 text-purple-300",
  DELETE: "border-rose-400/30 bg-rose-500/15 text-rose-300",
};

export function ApiEndpointCard({ endpoint }: { endpoint: ApiEndpoint }) {
  const [open, setOpen] = useState(false);
  const [copied, setCopied] = useState(false);

  async function handleCopyPath() {
    try {
      await navigator.clipboard.writeText(endpoint.path);
      setCopied(true);
      setTimeout(() => setCopied(false), 1400);
    } catch {
      setCopied(false);
    }
  }

  return (
    <article className="rounded-2xl border border-white/10 bg-zinc-900/70 p-5 shadow-lg shadow-black/25 backdrop-blur-sm transition hover:border-emerald-400/30 hover:bg-zinc-900">
      <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div className="flex flex-wrap items-center gap-2.5">
          <span
            className={`rounded-md border px-2.5 py-1 font-mono text-xs font-semibold ${
              METHOD_STYLES[endpoint.method]
            }`}
          >
            {endpoint.method}
          </span>

          <code className="rounded-lg bg-zinc-950/90 px-3 py-1 font-mono text-sm text-zinc-100 ring-1 ring-white/10">
            {endpoint.path}
          </code>

          <span className="rounded-full border border-white/10 bg-zinc-800/80 px-2.5 py-0.5 text-[11px] font-medium uppercase tracking-wider text-zinc-300">
            {endpoint.module}
          </span>

          <span
            className={`rounded-full px-2.5 py-0.5 text-[11px] font-medium ${
              endpoint.authRequired
                ? "bg-amber-500/10 text-amber-300 ring-1 ring-amber-400/20"
                : "bg-emerald-500/10 text-emerald-300 ring-1 ring-emerald-400/20"
            }`}
          >
            {endpoint.authRequired ? "Auth requerida" : "Público"}
          </span>
        </div>

        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={handleCopyPath}
            className="rounded-lg border border-white/10 bg-zinc-950/70 px-2.5 py-1.5 text-xs text-zinc-300 transition hover:border-white/25 hover:text-white"
          >
            {copied ? "Copiado" : "Copiar ruta"}
          </button>

          <button
            type="button"
            onClick={() => setOpen((prev) => !prev)}
            className="rounded-lg border border-emerald-400/30 bg-emerald-400/10 px-3 py-1.5 text-xs font-medium text-emerald-300 transition hover:bg-emerald-400/20"
          >
            {open ? "Ocultar JSON" : "Ver JSON"}
          </button>
        </div>
      </div>

      <div className="mt-3 space-y-1">
        <h2 className="text-base font-semibold text-white">{endpoint.summary}</h2>
        <p className="text-sm leading-relaxed text-zinc-400">{endpoint.description}</p>
      </div>

      {open && (
        <div className="mt-4 grid gap-4 border-t border-white/10 pt-4 md:grid-cols-2">
          {endpoint.sampleBody && (
            <div>
              <p className="mb-1.5 font-mono text-[11px] uppercase tracking-wider text-zinc-400">
                Request Body
              </p>
              <pre className="overflow-x-auto rounded-xl border border-white/10 bg-zinc-950 p-3.5 font-mono text-xs text-emerald-200">
                {JSON.stringify(endpoint.sampleBody, null, 2)}
              </pre>
            </div>
          )}

          <div className={endpoint.sampleBody ? "" : "md:col-span-2"}>
            <p className="mb-1.5 font-mono text-[11px] uppercase tracking-wider text-zinc-400">
              Sample Response
            </p>
            <pre className="overflow-x-auto rounded-xl border border-white/10 bg-zinc-950 p-3.5 font-mono text-xs text-sky-200">
              {JSON.stringify(endpoint.sampleResponse, null, 2)}
            </pre>
          </div>
        </div>
      )}
    </article>
  );
}