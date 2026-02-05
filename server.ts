import { serve, file } from "bun";
import { join } from "path";

const mimeTypes: Record<string, string> = {
  ".html": "text/html",
  ".js": "application/javascript",
  ".wasm": "application/wasm",
  ".css": "text/css",
  ".json": "application/json",
};

serve({
  port: 8080,
  async fetch(req) {
    const url = new URL(req.url);
    let pathname = url.pathname;

    if (pathname === "/") {
      pathname = "/index.html";
    }

    const filePath = join(import.meta.dir, pathname);
    const f = file(filePath);

    if (await f.exists()) {
      const ext = pathname.substring(pathname.lastIndexOf("."));
      const contentType = mimeTypes[ext] || "application/octet-stream";

      return new Response(f, {
        headers: {
          "Content-Type": contentType,
        },
      });
    }

    return new Response("Not Found", { status: 404 });
  },
});

console.log("Server running at http://localhost:8080");
