import vituum from 'vituum'
import liquid from '@vituum/vite-plugin-liquid'
import pluginPurgeCss from "@mojojoejo/vite-plugin-purgecss";

export default {
    plugins: [vituum(), liquid({
        root: './src'
    }),
    pluginPurgeCss({ variables: false }),
    ],
    build: {
        minify: true,
    },
    server: {
        cors: true,
        proxy: {
            '/api': {
                target: 'https://cloc.info/',
                changeOrigin: true,
            }
        }
    },
}
