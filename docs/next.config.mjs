/** @type {import('next').NextConfig} */
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const localReviewOrigins = ["127.0.0.1", "192.168.1.26"];
const docsDir = dirname(fileURLToPath(import.meta.url));

const nextConfig = {
  allowedDevOrigins: localReviewOrigins,
  pageExtensions: ["ts", "tsx", "md", "mdx"],
  serverExternalPackages: ["satteri"],
  turbopack: {
    root: join(docsDir, ".."),
  },
};

export default nextConfig;
