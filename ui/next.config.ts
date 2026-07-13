import type { NextConfig } from "next";
import createNextIntlPlugin from 'next-intl/plugin';

const withNextIntl = createNextIntlPlugin('./src/i18n/request.ts');

const nextConfig: NextConfig = {
  // Static export for embedding in Rust binary. Keep dev server on the normal runtime
  // so rewrites work and it does not read stale export artifacts.
  ...(process.env.NODE_ENV === "production" ? { output: "export" as const } : {}),
  basePath: '',
  trailingSlash: true,
  /** Dev-only: same-origin `/api` → Rust backend, avoids CORS when UI is on localhost:3840 */
  ...(process.env.NODE_ENV === "development"
    ? {
        async rewrites() {
          return [
            {
              source: "/api/:path*",
              destination: "http://127.0.0.1:3841/api/:path*",
            },
          ];
        },
      }
    : {}),
};

export default withNextIntl(nextConfig);
