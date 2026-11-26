"use client";
import "./api-doc.css";
import Link from "next/link";

interface ApiCornerButtonProps {
  href?: string;
  label?: string;
}

export function ApiCornerButton({
  href = "/api",
  label = "API Explorer",
}: ApiCornerButtonProps) {
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