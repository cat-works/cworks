import { defineConfig } from 'vite'
import { sveltekit } from '@sveltejs/kit/vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

// https://vitejs.dev/config/
export default defineConfig({
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
  }
})
