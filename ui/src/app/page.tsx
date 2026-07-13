"use client";

import { useEffect } from "react";
import { defaultLocale } from "@/i18n/config";

/**
 * Root `/` for static export (`out/index.html`). Tauri WebView often fails
 * `next/navigation` `router.replace`; full navigation works for dev + bundled app.
 */
export default function RootRedirectPage() {
  useEffect(() => {
    const target = `/${defaultLocale}/`;
    const path = window.location.pathname;
    // Only redirect real site root (not e.g. /en/index.html)
    if (path === "/" || path === "/index.html") {
      window.location.replace(target);
    }
  }, []);

  return (
    <div className="flex min-h-full flex-1 items-center justify-center">
      <p className="text-sm text-muted-foreground">Loading…</p>
    </div>
  );
}
