import Link from "next/link";

export default function NotFound() {
  return (
    <main className="flex min-h-screen flex-col items-center justify-center bg-black px-6 text-center text-white">
      <h1 className="text-3xl font-semibold">Page not found</h1>
      <p className="mt-3 text-sm text-gray-400">The requested page is unavailable.</p>
      <Link className="mt-6 rounded-lg bg-white px-4 py-2 text-sm font-semibold text-black" href="/">
        Back home
      </Link>
    </main>
  );
}
