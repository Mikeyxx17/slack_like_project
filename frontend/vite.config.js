import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'


// // 本地开发打开
// export default defineConfig({
//   plugins: [vue()],
//   server: {
//     proxy: {
//       '/ws': {
//         target: 'ws://127.0.0.1:3000',
//         ws: true
//       }
//     }
//   }
// })



// 远程开发打开
export default defineConfig({
  plugins: [vue()],
  server: {
    host: '0.0.0.0', // 监听所有地址
    port: 5173,
    hmr: {
      // 强制热更新使用公网 IP，替换成你真实的腾讯云公网 IP
      host: '119.29.224.113'
    }
  }
})