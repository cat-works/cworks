import { defineConfig } from 'vite'
import { sveltekit } from '@sveltejs/kit/vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

// https://vitejs.dev/config/
export default defineConfig({
  server: {allowedHosts: ["syoch-arch.tail39b6e.ts.net"]},
  build: {
    target: "esnext",
  },
  plugins: [
    sveltekit()
  ],
  resolve: {
    alias: {
      "@components/": `${__dirname}/src/components/`,
      "@datas/": `${__dirname}/src/datas/`,
      "@src/": `${__dirname}/src/`,
    },
  },
  optimizeDeps: {
    exclude: [
      "@battlefieldduck/xterm-svelte"
    ]
  }
})
