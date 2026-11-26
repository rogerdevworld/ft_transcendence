import type { ReactNode } from "react";
import "./api-doc/api-doc.css";
import { ApiCornerButton } from "@/app/api-doc/ApiCornerButton";

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="es">
      <body className="bg-zinc-950 text-zinc-100 antialiased">
        {children}
        <ApiCornerButton href="/api-dic" label="API Explorer" />
      </body>
    </html>
  );
}