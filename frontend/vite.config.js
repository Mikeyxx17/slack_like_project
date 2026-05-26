// import { defineConfig } from 'vite'
// import vue from '@vitejs/plugin-vue'
// import tailwindcss from "@tailwindcss/vite"

// export default defineConfig({
//   plugins: [
//     vue(),
//     tailwindcss(),
//   ],
//   server: {
//     host: '0.0.0.0',
//     port: 5173,
//     proxy: {
//       '/ws': {
//         target: 'ws://127.0.0.1:3000',
//         ws: true
//       },
//       '/api': {
//         target: 'http://127.0.0.1:3000',
//         changeOrigin: true
//       }
//     }
//   }
// })


import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import tailwindcss from "@tailwindcss/vite"

export default defineConfig({
  plugins: [
    vue(),
    tailwindcss(),
  ],
  server: {
    host: '0.0.0.0',
    port: 5173,
    hmr: {
      host: '139.159.144.79'
    },
    proxy: {

      '/ws': {

        target: 'ws://127.0.0.1:3000',
        ws: true,
        rewrite: (path) => path
      },

      '/api': {
        target: 'http://127.0.0.1:3000',
        changeOrigin: true
      }
    }
  }
})