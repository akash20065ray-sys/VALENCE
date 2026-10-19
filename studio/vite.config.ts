import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'
import net from 'node:net'

function valenceSyncPlugin() {
  function queryValence(cmd: string): Promise<string> {
    return new Promise((resolve, reject) => {
      const client = net.createConnection({ port: 6379, host: '127.0.0.1' }, () => {
        client.write(cmd)
      })
      let data = ''
      client.on('data', chunk => {
        data += chunk.toString()
        client.end()
      })
      client.on('end', () => resolve(data))
      client.on('error', err => reject(err))
    })
  }

  return {
    name: 'valence-sync-plugin',
    configureServer(server: any) {
      server.middlewares.use('/api/sync', async (_req: any, res: any) => {
        try {
          // 1. Get all keys from Valence Engine
          const rawKeys = await queryValence('*2\r\n$4\r\nKEYS\r\n$1\r\n*\r\n')
          const lines = rawKeys.split('\r\n')
          const allKeys: string[] = []
          for (let i = 1; i < lines.length; i++) {
            if (lines[i].startsWith('$') && lines[i + 1] !== undefined) {
              allKeys.push(lines[i + 1])
              i++
            }
          }

          // Limit preview to top 50 keys for instant 5ms responsiveness
          const previewKeys = allKeys.slice(-50)

          // 2. Fetch value for each preview key
          const results = []
          for (const k of previewKeys) {
            if (!k) continue
            try {
              const getCmd = `*2\r\n$3\r\nGET\r\n$${Buffer.byteLength(k)}\r\n${k}\r\n`
              const rawVal = await queryValence(getCmd)
              const valLines = rawVal.split('\r\n')
              let val = ''
              if (valLines[0].startsWith('$') && valLines[1] !== undefined) {
                val = valLines[1]
              }
              results.push({
                key: k,
                value: val,
                dataType: 'string',
                sizeBytes: Buffer.byteLength(val),
              })
            } catch {
              // ignore single key error
            }
          }

          res.setHeader('Content-Type', 'application/json')
          res.end(JSON.stringify({ 
            success: true, 
            totalCount: allKeys.length, 
            keys: results 
          }))
        } catch (err: any) {
          res.setHeader('Content-Type', 'application/json')
          res.end(JSON.stringify({ success: false, error: err.message, keys: [], totalCount: 0 }))
        }
      })
    },
  }
}

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), valenceSyncPlugin()],
})
