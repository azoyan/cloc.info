import vituum from 'vituum'
import pug from '@vituum/vite-plugin-pug'
import pluginPurgeCss from "@mojojoejo/vite-plugin-purgecss";

export default {
    plugins: [vituum(), pug({
        root: './src'
    }),
    pluginPurgeCss()
    ],

    build: {
        rollupOptions: {
            output: {

            }
        }
    },
    server: {
        cors: false,
        proxy: {
            '/api': {
                target: 'https://cloc.info',
                changeOrigin: true,
                rewrite: path => path.replace(/^\/ws/, '')
            }
        }
    },
}
