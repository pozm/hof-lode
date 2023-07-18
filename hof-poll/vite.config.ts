import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [svelte()],
  server:{
    proxy:{
      "/api":{
        target:"http://localhost:3133",
        changeOrigin:true,
      }
    }
  },
  build:{
    outDir:"../public/poll",
  },
  base:"/p/poll/",
})
