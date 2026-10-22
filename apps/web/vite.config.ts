import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// G-03.08: port 4102 zapíná jen explicitní lokální opt-in.
const apiTarget = process.env.AIRRADAR_V2_G03_API === "1"
  ? "http://127.0.0.1:4102"
  : "http://127.0.0.1:4101";

export default defineConfig({
  plugins: [react()],
  server: {
    host: "127.0.0.1",
    port: 4100,
    strictPort: true,
    proxy: {
      "/api": {
        target: apiTarget,
        changeOrigin: false,
      },
    },
  },
  preview: {
    host: "127.0.0.1",
    port: 4100,
    strictPort: true,
  },
});
