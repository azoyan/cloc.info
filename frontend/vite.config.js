import vituum from 'vituum'
import pug from '@vituum/vite-plugin-pug'
import { viteStaticCopy } from 'vite-plugin-static-copy'

export default {
    plugins: [
        vituum(),
        pug({
            root: './src'
        }),
        viteStaticCopy({
            targets: [
                {
                    src: 'public/*',
                    dest: 'assets/'
                }
            ]
        })
    ],
    build: {
        rollupOptions: {
            output: {
                assetFileNames: (assetInfo) => {
                    const names = [
                        assetInfo.name,
                        ...(assetInfo.names || []),
                        ...(assetInfo.originalFileNames || []),
                    ].filter(Boolean)

                    if (names.some((name) => name.endsWith('main.css'))) {
                        return 'assets/main.css'
                    }

                    return 'assets/[name]-[hash][extname]'
                }
            },
        },
        assetsInlineLimit: 0,
        minify: true,
        terserOptions: {
            compress: {
                toplevel: true,
                defaults: true,
                passes: 1,
                hoist_props: true,
                hoist_vars: true,
                hoist_funs: true,
                module: true,
                arguments: true,
                drop_console: true
            }
        }
    },
    server: {
        cors: true,
        proxy: {
            '/ws': {
                target: 'ws://127.0.0.1:9999/',
                ws: true,
                changeOrigin: true,
                rewrite: path => path.replace(/^\/ws/, '')
            },
            '/': {
                target: 'http://127.0.0.1:9999/',
                changeOrigin: true,
                timeout: 10000
            }
        }
    }
}
