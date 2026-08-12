export function normalizeApiPath(path: string): string {
  const cleanPath = path.trim().replace(/^\//, "");
  if (
    !cleanPath ||
    cleanPath.includes("://") ||
    cleanPath.includes("\\") ||
    cleanPath.split("/").some((part) => part === "..") ||
    !/^[A-Za-z0-9/_.-]+$/.test(cleanPath)
  ) {
    throw new Error("Invalid API path.");
  }
  return cleanPath;
}
