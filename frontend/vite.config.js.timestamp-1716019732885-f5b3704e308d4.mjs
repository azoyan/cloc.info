// vite.config.js
import vituum from "file:///home/i/vituum/examples/pug/node_modules/vituum/src/index.js";
import pug from "file:///home/i/vituum/examples/pug/node_modules/@vituum/vite-plugin-pug/index.js";
import pluginPurgeCss from "file:///home/i/node_modules/@mojojoejo/vite-plugin-purgecss/dist/index.mjs";
import compression from "file:///home/i/vituum/examples/pug/node_modules/vite-plugin-compression2/dist/index.mjs";
var vite_config_default = {
  plugins: [
    vituum(),
    pug({
      root: "./src"
    }),
    pluginPurgeCss({ variables: true, safelist: ["p-3.5", "md:block", "disabled:bg-neutral-400", "w-1/6", "w-4/5", "md:w-5/6"] }),
    compression({ algorithm: "brotliCompress" })
  ],
  build: {
    minify: true
  },
  server: {
    cors: true,
    proxy: {
      "/api": {
        target: "https://cloc.info/",
        changeOrigin: true,
        timeout: 1e4
      },
      "/ws": {
        target: "ws://127.0.0.1:9999/",
        ws: true,
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/ws/, "")
      },
      "/github.com/azoyan/talua": {
        target: "https://127.0.0.1:9999/",
        changeOrigin: true,
        timeout: 1e4
      }
    }
  }
};
export {
  vite_config_default as default
};
//# sourceMappingURL=data:application/json;base64,ewogICJ2ZXJzaW9uIjogMywKICAic291cmNlcyI6IFsidml0ZS5jb25maWcuanMiXSwKICAic291cmNlc0NvbnRlbnQiOiBbImNvbnN0IF9fdml0ZV9pbmplY3RlZF9vcmlnaW5hbF9kaXJuYW1lID0gXCIvaG9tZS9pL3ZpdHV1bS9leGFtcGxlcy9wdWdcIjtjb25zdCBfX3ZpdGVfaW5qZWN0ZWRfb3JpZ2luYWxfZmlsZW5hbWUgPSBcIi9ob21lL2kvdml0dXVtL2V4YW1wbGVzL3B1Zy92aXRlLmNvbmZpZy5qc1wiO2NvbnN0IF9fdml0ZV9pbmplY3RlZF9vcmlnaW5hbF9pbXBvcnRfbWV0YV91cmwgPSBcImZpbGU6Ly8vaG9tZS9pL3ZpdHV1bS9leGFtcGxlcy9wdWcvdml0ZS5jb25maWcuanNcIjtpbXBvcnQgdml0dXVtIGZyb20gJ3ZpdHV1bSdcbmltcG9ydCBwdWcgZnJvbSAnQHZpdHV1bS92aXRlLXBsdWdpbi1wdWcnXG5pbXBvcnQgcGx1Z2luUHVyZ2VDc3MgZnJvbSAnQG1vam9qb2Vqby92aXRlLXBsdWdpbi1wdXJnZWNzcydcbmltcG9ydCBjb21wcmVzc2lvbiBmcm9tICd2aXRlLXBsdWdpbi1jb21wcmVzc2lvbjInXG4vLyBpbXBvcnQgdGFpbHdpbmRjc3MgZnJvbSAnQHZpdHV1bS92aXRlLXBsdWdpbi10YWlsd2luZGNzcydcbi8vIGltcG9ydCBwb3N0Y3NzIGZyb20gJ0B2aXR1dW0vdml0ZS1wbHVnaW4tcG9zdGNzcydcblxuZXhwb3J0IGRlZmF1bHQge1xuICAgIHBsdWdpbnM6IFt2aXR1dW0oKSwgcHVnKHtcbiAgICAgICAgcm9vdDogJy4vc3JjJ1xuICAgIH0pLFxuICAgIHBsdWdpblB1cmdlQ3NzKHsgdmFyaWFibGVzOiB0cnVlLCBzYWZlbGlzdDogWydwLTMuNScsICdtZDpibG9jaycsICdkaXNhYmxlZDpiZy1uZXV0cmFsLTQwMCcsICd3LTEvNicsICd3LTQvNScsICdtZDp3LTUvNiddIH0pLFxuICAgIGNvbXByZXNzaW9uKHsgYWxnb3JpdGhtOiAnYnJvdGxpQ29tcHJlc3MnIH0pXG4gICAgXSxcbiAgICBidWlsZDoge1xuICAgICAgICBtaW5pZnk6IHRydWVcbiAgICB9LFxuICAgIHNlcnZlcjoge1xuICAgICAgICBjb3JzOiB0cnVlLFxuICAgICAgICBwcm94eToge1xuICAgICAgICAgICAgJy9hcGknOiB7XG4gICAgICAgICAgICAgICAgdGFyZ2V0OiAnaHR0cHM6Ly9jbG9jLmluZm8vJyxcbiAgICAgICAgICAgICAgICBjaGFuZ2VPcmlnaW46IHRydWUsXG4gICAgICAgICAgICAgICAgdGltZW91dDogMTAwMDBcbiAgICAgICAgICAgIH0sXG4gICAgICAgICAgICAnL3dzJzoge1xuICAgICAgICAgICAgICAgIHRhcmdldDogJ3dzOi8vMTI3LjAuMC4xOjk5OTkvJyxcbiAgICAgICAgICAgICAgICB3czogdHJ1ZSxcbiAgICAgICAgICAgICAgICBjaGFuZ2VPcmlnaW46IHRydWUsXG4gICAgICAgICAgICAgICAgcmV3cml0ZTogcGF0aCA9PiBwYXRoLnJlcGxhY2UoL15cXC93cy8sICcnKVxuICAgICAgICAgICAgfSxcbiAgICAgICAgICAgICcvZ2l0aHViLmNvbS9hem95YW4vdGFsdWEnOiB7XG4gICAgICAgICAgICAgICAgdGFyZ2V0OiAnaHR0cHM6Ly8xMjcuMC4wLjE6OTk5OS8nLFxuICAgICAgICAgICAgICAgIGNoYW5nZU9yaWdpbjogdHJ1ZSxcbiAgICAgICAgICAgICAgICB0aW1lb3V0OiAxMDAwMFxuICAgICAgICAgICAgfVxuICAgICAgICB9XG4gICAgfVxufVxuIl0sCiAgIm1hcHBpbmdzIjogIjtBQUFtUSxPQUFPLFlBQVk7QUFDdFIsT0FBTyxTQUFTO0FBQ2hCLE9BQU8sb0JBQW9CO0FBQzNCLE9BQU8saUJBQWlCO0FBSXhCLElBQU8sc0JBQVE7QUFBQSxFQUNYLFNBQVM7QUFBQSxJQUFDLE9BQU87QUFBQSxJQUFHLElBQUk7QUFBQSxNQUNwQixNQUFNO0FBQUEsSUFDVixDQUFDO0FBQUEsSUFDRCxlQUFlLEVBQUUsV0FBVyxNQUFNLFVBQVUsQ0FBQyxTQUFTLFlBQVksMkJBQTJCLFNBQVMsU0FBUyxVQUFVLEVBQUUsQ0FBQztBQUFBLElBQzVILFlBQVksRUFBRSxXQUFXLGlCQUFpQixDQUFDO0FBQUEsRUFDM0M7QUFBQSxFQUNBLE9BQU87QUFBQSxJQUNILFFBQVE7QUFBQSxFQUNaO0FBQUEsRUFDQSxRQUFRO0FBQUEsSUFDSixNQUFNO0FBQUEsSUFDTixPQUFPO0FBQUEsTUFDSCxRQUFRO0FBQUEsUUFDSixRQUFRO0FBQUEsUUFDUixjQUFjO0FBQUEsUUFDZCxTQUFTO0FBQUEsTUFDYjtBQUFBLE1BQ0EsT0FBTztBQUFBLFFBQ0gsUUFBUTtBQUFBLFFBQ1IsSUFBSTtBQUFBLFFBQ0osY0FBYztBQUFBLFFBQ2QsU0FBUyxVQUFRLEtBQUssUUFBUSxTQUFTLEVBQUU7QUFBQSxNQUM3QztBQUFBLE1BQ0EsNEJBQTRCO0FBQUEsUUFDeEIsUUFBUTtBQUFBLFFBQ1IsY0FBYztBQUFBLFFBQ2QsU0FBUztBQUFBLE1BQ2I7QUFBQSxJQUNKO0FBQUEsRUFDSjtBQUNKOyIsCiAgIm5hbWVzIjogW10KfQo=
