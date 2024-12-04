// vite.config.ts
import path from "path";
import { defineConfig } from "file:///home/guillem/projects/messenger-demo/node_modules/.pnpm/vite@4.5.5/node_modules/vite/dist/node/index.js";
import { viteStaticCopy } from "file:///home/guillem/projects/messenger-demo/node_modules/.pnpm/vite-plugin-static-copy@0.13.1_vite@4.5.5/node_modules/vite-plugin-static-copy/dist/index.js";
var __vite_injected_original_dirname = "/home/guillem/projects/messenger-demo/ui";
var vite_config_default = defineConfig({
  server: {
    port: 1420,
    strictPort: true,
    host: process.env.TAURI_DEV_HOST || false,
    hmr: process.env.TAURI_DEV_HOST ? {
      protocol: "ws",
      host: process.env.TAURI_DEV_HOST,
      port: 1430
    } : void 0
  },
  plugins: [
    viteStaticCopy({
      targets: [
        {
          src: path.resolve(
            __vite_injected_original_dirname,
            "node_modules/@shoelace-style/shoelace/dist/assets"
          ),
          dest: path.resolve(__vite_injected_original_dirname, "dist/shoelace")
        }
      ]
    })
  ]
});
export {
  vite_config_default as default
};
//# sourceMappingURL=data:application/json;base64,ewogICJ2ZXJzaW9uIjogMywKICAic291cmNlcyI6IFsidml0ZS5jb25maWcudHMiXSwKICAic291cmNlc0NvbnRlbnQiOiBbImNvbnN0IF9fdml0ZV9pbmplY3RlZF9vcmlnaW5hbF9kaXJuYW1lID0gXCIvaG9tZS9ndWlsbGVtL3Byb2plY3RzL21lc3Nlbmdlci1kZW1vL3VpXCI7Y29uc3QgX192aXRlX2luamVjdGVkX29yaWdpbmFsX2ZpbGVuYW1lID0gXCIvaG9tZS9ndWlsbGVtL3Byb2plY3RzL21lc3Nlbmdlci1kZW1vL3VpL3ZpdGUuY29uZmlnLnRzXCI7Y29uc3QgX192aXRlX2luamVjdGVkX29yaWdpbmFsX2ltcG9ydF9tZXRhX3VybCA9IFwiZmlsZTovLy9ob21lL2d1aWxsZW0vcHJvamVjdHMvbWVzc2VuZ2VyLWRlbW8vdWkvdml0ZS5jb25maWcudHNcIjtpbXBvcnQgcGF0aCBmcm9tIFwicGF0aFwiO1xuaW1wb3J0IHsgZGVmaW5lQ29uZmlnIH0gZnJvbSBcInZpdGVcIjtcbmltcG9ydCB7IHZpdGVTdGF0aWNDb3B5IH0gZnJvbSBcInZpdGUtcGx1Z2luLXN0YXRpYy1jb3B5XCI7XG5cbmV4cG9ydCBkZWZhdWx0IGRlZmluZUNvbmZpZyh7XG4gIHNlcnZlcjoge1xuICAgIHBvcnQ6IDE0MjAsXG4gICAgc3RyaWN0UG9ydDogdHJ1ZSxcbiAgICBob3N0OiBwcm9jZXNzLmVudi5UQVVSSV9ERVZfSE9TVCB8fCBmYWxzZSxcbiAgICBobXI6IHByb2Nlc3MuZW52LlRBVVJJX0RFVl9IT1NUXG4gICAgICA/IHtcbiAgICAgICAgICBwcm90b2NvbDogXCJ3c1wiLFxuICAgICAgICAgIGhvc3Q6IHByb2Nlc3MuZW52LlRBVVJJX0RFVl9IT1NULFxuICAgICAgICAgIHBvcnQ6IDE0MzAsXG4gICAgICAgIH1cbiAgICAgIDogdW5kZWZpbmVkLFxuICB9LFxuICBwbHVnaW5zOiBbXG4gICAgdml0ZVN0YXRpY0NvcHkoe1xuICAgICAgdGFyZ2V0czogW1xuICAgICAgICB7XG4gICAgICAgICAgc3JjOiBwYXRoLnJlc29sdmUoXG4gICAgICAgICAgICBfX2Rpcm5hbWUsXG4gICAgICAgICAgICBcIm5vZGVfbW9kdWxlcy9Ac2hvZWxhY2Utc3R5bGUvc2hvZWxhY2UvZGlzdC9hc3NldHNcIixcbiAgICAgICAgICApLFxuICAgICAgICAgIGRlc3Q6IHBhdGgucmVzb2x2ZShfX2Rpcm5hbWUsIFwiZGlzdC9zaG9lbGFjZVwiKSxcbiAgICAgICAgfSxcbiAgICAgIF0sXG4gICAgfSksXG4gIF0sXG59KTtcbiJdLAogICJtYXBwaW5ncyI6ICI7QUFBMFMsT0FBTyxVQUFVO0FBQzNULFNBQVMsb0JBQW9CO0FBQzdCLFNBQVMsc0JBQXNCO0FBRi9CLElBQU0sbUNBQW1DO0FBSXpDLElBQU8sc0JBQVEsYUFBYTtBQUFBLEVBQzFCLFFBQVE7QUFBQSxJQUNOLE1BQU07QUFBQSxJQUNOLFlBQVk7QUFBQSxJQUNaLE1BQU0sUUFBUSxJQUFJLGtCQUFrQjtBQUFBLElBQ3BDLEtBQUssUUFBUSxJQUFJLGlCQUNiO0FBQUEsTUFDRSxVQUFVO0FBQUEsTUFDVixNQUFNLFFBQVEsSUFBSTtBQUFBLE1BQ2xCLE1BQU07QUFBQSxJQUNSLElBQ0E7QUFBQSxFQUNOO0FBQUEsRUFDQSxTQUFTO0FBQUEsSUFDUCxlQUFlO0FBQUEsTUFDYixTQUFTO0FBQUEsUUFDUDtBQUFBLFVBQ0UsS0FBSyxLQUFLO0FBQUEsWUFDUjtBQUFBLFlBQ0E7QUFBQSxVQUNGO0FBQUEsVUFDQSxNQUFNLEtBQUssUUFBUSxrQ0FBVyxlQUFlO0FBQUEsUUFDL0M7QUFBQSxNQUNGO0FBQUEsSUFDRixDQUFDO0FBQUEsRUFDSDtBQUNGLENBQUM7IiwKICAibmFtZXMiOiBbXQp9Cg==
