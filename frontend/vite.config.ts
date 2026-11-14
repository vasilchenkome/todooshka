import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  server: {
    // В разработке фронт и API живут на одном origin: Vite проксирует /api на бэкенд.
    proxy: {
      "/api": "http://127.0.0.1:8080",
    },
  },
});
