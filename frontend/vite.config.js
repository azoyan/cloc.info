import vituum from 'vituum'
import pug from '@vituum/vite-plugin-pug'
import pluginPurgeCss from '@mojojoejo/vite-plugin-purgecss'
import purgeHtml from 'purgecss-from-html'
import purgePug from 'purgecss-from-pug'

import { viteStaticCopy } from 'vite-plugin-static-copy'
import { BORDER_T, ROUNDED_B_LG, H_5, LIST_GROUP_ITEM, TEXT_BASE, DARK_HOVER_BG_ZINC_800, DARK_HOVER_TEXT_NEUTRAL_100, JUSTIFY_START, RIGHT_2, BG_NEUTRAL_100, W_8, SPACE_Y_4, LINK_SECONDARY, PB_8, FOCUS_OUTLINE_NONE, MIN_W_28, BORDER_RED_300, OVERFLOW_AUTO, PY_3, RING_GRAY_500_10, BG_NEUTRAL_50, MD, SPACE_X_2, PY_4, PX_2_5, FONT_LIGHT, PX_5, FLEX, GAP_12, FONT_MEDIUM, TOP_1_2, FLEX_WRAP, TEXT_NEUTRAL_100, BORDER, JUSTIFY_BETWEEN, FIRST_ROUNDED_T_LG, ALIGN_MIDDLE, W_5, DISABLED_BG_ZINC_500, FLEX_NONE, FONT_MONOSPACE, H_16, TOP_0, TEXT_START, MY_10, ROUNDED_LG, FIXED, SM_W_1_12, GAP_X_3_5, TEXT_NEUTRAL_900, PY_1_5, ANIMATE_SPIN, FIRST_MT_0, W_FULL, DARK_BG_ZINC_800, DISABLED_BORDER_ZINC_500, LIST_INSIDE, MB_2, TEXT_NOWRAP, DARK_TEXT_NEUTRAL_300, H_12, MX_1, DARK_BG_GREEN_800, TRANSITION, DIVIDE_Y, MAX_W_72REM, SM_W_64, Z_10, SM_W_52, BORDER_R, PT_1, ROUNDED_R_LG, W_36, P_2, INLINE_FLEX, TRANSFORM, HOVER_BG_NEUTRAL_100, PLACEHOLDER_TEXT_SM, ITEMS_CENTER, DARK_HOVER_ROUNDED_LG, SPACE_Y_2, P_1, DARK_BG_ZINC_950, DARK_BG_NEUTRAL_800, H_6, DARK, TRUNCATE, TEXT_2XL, TEXT_NEUTRAL_500, DARK_TEXT_NEUTRAL_100, ROUNDED_T_LG, MY_1, TEXT_CENTER, PY_12, MD_TABLE_FIXED, TEXT_SM, FOCUS_RING_GREEN_300, MD_BLOCK, DARK_TEXT_NEUTRAL_200, FONT_MONO, DARK_BG_ZINC_900, TEXT_RED_800, DARK_TEXT_WHITE, RING_1, MIN_W_64, SM_PL_2, LINK_HOVER, GRID_FLOW_COL, HOVER_TEXT_NEUTRAL_800, BORDER_NEUTRAL, ML_AUTO, HOVER_TEXT_WHITE, BORDER_L, LINK, UNDERLINE, TEXT_NEUTRAL_300, FONT_NORMAL, ROUNDED_BL_LG, GRID, TABLE_FIXED, SM, DARK_HOVER_TEXT_NEUTRAL_300, MIN_W_SCREEN_LG, MIN_H_SVH, W_3_4, P_4, BORDER_RED, MAX_H_72, CONTAINER, TEXT_XL, COLLAPSABLE, RING_INSET, ROUNDED_MD, PT_1_5, BORDER_NEUTRAL_200, GAP_10, ME_3, BG_RED_50, DIVIDE_ZINC_600, TABLE, HIDDEN, CURSOR_POINTER, FLEX_SHRINK_0, ABSOLUTE, SPACE_Y_12, HOVER_BG_NEUTRAL_50, TEXT_SECONDARY_EMPHASIS, HOVER_UNDERLINE, MB_4, MAX_W_SCREEN_LG, ORIGIN_TOP, INLINE, SPACE_Y_8, DARK_HOVER_BG_GREEN_700, BG_ZINC_950, COLLAPSE, STICKY, MIN_H_SCREEN, HOVER_ROUNDED, DARK_BG_NEUTRAL_300, OVERFLOW_X_AUTO, LIST_NONE, W_4, BG_GREEN_700, PX_2, MIN_W_12, MY_4, SM_H_AUTO, SM_BLOCK, WHITESPACE_NOWRAP, LG_W_1_6, WHITESPACE_PRE, PT_10, _MT_PX, W_28, DARK_FOCUS_OUTLINE_1, SPACE_X_3, DARK_TEXT_GREEN_500, W_32, DARK_HOVER_TEXT_WHITE, JUSTIFY_SELF_CENTER, TABLE_AUTO, BLOCK, PB_1, FOCUS_RING, INVISIBLE, PX_1, TEXT_WHITE, BG_ZINC_900, H_4, PY_5, TEXT, DARK_PLACEHOLDER_NEUTRAL_500, TEXT_NEUTRAL_700, SM_PR_4, JUSTIFY_CENTER, HOVER_TEXT_NEUTRAL_900, ROUNDED, SM_W_32, TEXT_3XL, PX_4, FOCUS_BG_WHITE, W_2, W_24, PR_8, MAX_H_SCREEN, LAST_ROUNDED_B_LG, ROUNDED_BR_LG, PY_2, DARK_BORDER_ZINC_500, ROUNDED_L_LG, PE_10, TEXT_4XL, TEXT_GREEN_500, TEXT_NEUTRAL_600, ME_2, BG_WHITE, H_8, HOVER_TEXT_NEUTRAL_700, SHADOW_LG, H_96, SELF_END, DARK_TEXT_NEUTRAL_400, TEXT_NEUTRAL_800, M_5, FLEX_COL, HOVER_BG_GREEN_800, DARK_DIVIDE_ZINC_600, UNDERLINE_OFFSET_2, GROW, BORDER_B, W_6, MX_AUTO, BORDER_NEUTRAL_300, HOVER_TEXT_BLACK, MD_W_1_5, PT_2, FOCUS_RING_4, MAX_W_FULL, PLACE_ITEMS_CENTER, PE_4, LINK_DARK, TRANSITION_TRANSFORM, W_1_2, _TRANSLATE_Y_1_2, MX_2, TEXT_END } from './src/js/tailwind-classes.js'

// import tailwindcss from '@vituum/vite-plugin-tailwindcss'
// import postcss from '@vituum/vite-plugin-postcss'


export default {
    plugins: [vituum(), pug({
        root: './src'
    }),
    viteStaticCopy({
        targets: [
            {
                src: 'public/*',
                dest: 'assets/'
            }
        ]
    }),
    pluginPurgeCss({
        variables: true,
        content: ['./src/**/*.html', './src/**/*.js', './src/**/*.pug'],
        extractors: [
            {
                extractor: purgeHtml,
                extensions: ['js', 'html']
            },
            {
                extractor: purgePug,
                extensions: ['pug']
            }
        ],
        safelist: [
            BORDER_T, ROUNDED_B_LG, H_5, LIST_GROUP_ITEM, TEXT_BASE, DARK_HOVER_BG_ZINC_800, DARK_HOVER_TEXT_NEUTRAL_100, JUSTIFY_START, RIGHT_2, BG_NEUTRAL_100, W_8, SPACE_Y_4, LINK_SECONDARY, PB_8, FOCUS_OUTLINE_NONE, MIN_W_28, BORDER_RED_300, OVERFLOW_AUTO, PY_3, RING_GRAY_500_10, BG_NEUTRAL_50, MD, SPACE_X_2, PY_4, PX_2_5, FONT_LIGHT, PX_5, FLEX, GAP_12, FONT_MEDIUM, TOP_1_2, FLEX_WRAP, TEXT_NEUTRAL_100, BORDER, JUSTIFY_BETWEEN, FIRST_ROUNDED_T_LG, ALIGN_MIDDLE, W_5, DISABLED_BG_ZINC_500, FLEX_NONE, FONT_MONOSPACE, H_16, TOP_0, TEXT_START, MY_10, ROUNDED_LG, FIXED, SM_W_1_12, GAP_X_3_5, TEXT_NEUTRAL_900, PY_1_5, ANIMATE_SPIN, FIRST_MT_0, W_FULL, DARK_BG_ZINC_800, DISABLED_BORDER_ZINC_500, LIST_INSIDE, MB_2, TEXT_NOWRAP, DARK_TEXT_NEUTRAL_300, H_12, MX_1, DARK_BG_GREEN_800, TRANSITION, DIVIDE_Y, MAX_W_72REM, SM_W_64, Z_10, SM_W_52, BORDER_R, PT_1, ROUNDED_R_LG, W_36, P_2, INLINE_FLEX, TRANSFORM, HOVER_BG_NEUTRAL_100, PLACEHOLDER_TEXT_SM, ITEMS_CENTER, DARK_HOVER_ROUNDED_LG, SPACE_Y_2, P_1, DARK_BG_ZINC_950, DARK_BG_NEUTRAL_800, H_6, DARK, TRUNCATE, TEXT_2XL, TEXT_NEUTRAL_500, DARK_TEXT_NEUTRAL_100, ROUNDED_T_LG, MY_1, TEXT_CENTER, PY_12, MD_TABLE_FIXED, TEXT_SM, FOCUS_RING_GREEN_300, MD_BLOCK, DARK_TEXT_NEUTRAL_200, FONT_MONO, DARK_BG_ZINC_900, TEXT_RED_800, DARK_TEXT_WHITE, RING_1, MIN_W_64, SM_PL_2, LINK_HOVER, GRID_FLOW_COL, HOVER_TEXT_NEUTRAL_800, BORDER_NEUTRAL, ML_AUTO, HOVER_TEXT_WHITE, BORDER_L, LINK, UNDERLINE, TEXT_NEUTRAL_300, FONT_NORMAL, ROUNDED_BL_LG, GRID, TABLE_FIXED, SM, DARK_HOVER_TEXT_NEUTRAL_300, MIN_W_SCREEN_LG, MIN_H_SVH, W_3_4, P_4, BORDER_RED, MAX_H_72, CONTAINER, TEXT_XL, COLLAPSABLE, RING_INSET, ROUNDED_MD, PT_1_5, BORDER_NEUTRAL_200, GAP_10, ME_3, BG_RED_50, DIVIDE_ZINC_600, TABLE, HIDDEN, CURSOR_POINTER, FLEX_SHRINK_0, ABSOLUTE, SPACE_Y_12, HOVER_BG_NEUTRAL_50, TEXT_SECONDARY_EMPHASIS, HOVER_UNDERLINE, MB_4, MAX_W_SCREEN_LG, ORIGIN_TOP, INLINE, SPACE_Y_8, DARK_HOVER_BG_GREEN_700, BG_ZINC_950, COLLAPSE, STICKY, MIN_H_SCREEN, HOVER_ROUNDED, DARK_BG_NEUTRAL_300, OVERFLOW_X_AUTO, LIST_NONE, W_4, BG_GREEN_700, PX_2, MIN_W_12, MY_4, SM_H_AUTO, SM_BLOCK, WHITESPACE_NOWRAP, LG_W_1_6, WHITESPACE_PRE, PT_10, _MT_PX, W_28, DARK_FOCUS_OUTLINE_1, SPACE_X_3, DARK_TEXT_GREEN_500, W_32, DARK_HOVER_TEXT_WHITE, JUSTIFY_SELF_CENTER, TABLE_AUTO, BLOCK, PB_1, FOCUS_RING, INVISIBLE, PX_1, TEXT_WHITE, BG_ZINC_900, H_4, PY_5, TEXT, DARK_PLACEHOLDER_NEUTRAL_500, TEXT_NEUTRAL_700, SM_PR_4, JUSTIFY_CENTER, HOVER_TEXT_NEUTRAL_900, ROUNDED, SM_W_32, TEXT_3XL, PX_4, FOCUS_BG_WHITE, W_2, W_24, PR_8, MAX_H_SCREEN, LAST_ROUNDED_B_LG, ROUNDED_BR_LG, PY_2, DARK_BORDER_ZINC_500, ROUNDED_L_LG, PE_10, TEXT_4XL, TEXT_GREEN_500, TEXT_NEUTRAL_600, ME_2, BG_WHITE, H_8, HOVER_TEXT_NEUTRAL_700, SHADOW_LG, H_96, SELF_END, DARK_TEXT_NEUTRAL_400, TEXT_NEUTRAL_800, M_5, FLEX_COL, HOVER_BG_GREEN_800, DARK_DIVIDE_ZINC_600, UNDERLINE_OFFSET_2, GROW, BORDER_B, W_6, MX_AUTO, BORDER_NEUTRAL_300, HOVER_TEXT_BLACK, MD_W_1_5, PT_2, FOCUS_RING_4, MAX_W_FULL, PLACE_ITEMS_CENTER, PE_4, LINK_DARK, TRANSITION_TRANSFORM, W_1_2, _TRANSLATE_Y_1_2, MX_2, TEXT_END
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

                    if (names.some((name) => name.endsWith("main.css"))) {
                        return "assets/main.css"
                    }

                    return "assets/[name]-[hash][extname]"
                }
            },
        },
        assetsInlineLimit: 0,
        minify: true,
        terserOptions: {
            // mangle: {
            //     toplevel: true,
            //     properties: {
            //         keep_quoted: 'strict',
            //         undeclared: true
            //     }
            // },
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
            // '/api': {
            //     target: 'http://cloc.info/api/',
            //     changeOrigin: true,
            //     timeout: 10000
            // },
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
