import { defineConfig } from "vite";
import react from "@vitejs/plugin-react-swc";
import path from "path";
import { componentTagger } from "lovable-tagger";
import { articlesApiPlugin } from "./vite-plugin-articles-api";

// https://vitejs.dev/config/
export default defineConfig(({ mode }) => ({
  server: {
    host: "::",
    port: 8080,
    fs: {
      // Allow serving files from one level up to the project root
      allow: ['..'],
    },
  },
  plugins: [
    react({
      // Configurar SWC para não transpilar features modernas desnecessariamente
      // Target moderno: ES2022+ (remover polyfills desnecessários)
      swcOptions: {
        jsc: {
          target: 'es2022', // Usar ES2022 para evitar transpilação de features modernas
          parser: {
            syntax: 'typescript',
            tsx: true,
            decorators: false,
            dynamicImport: true,
          },
          transform: {
            react: {
              runtime: 'automatic',
            },
          },
          // Não adicionar helpers/polyfills para features modernas (Array.at, Object.hasOwn, etc)
          // Os navegadores modernos já suportam essas features
          loose: false,
          externalHelpers: false,
        },
        minify: mode === 'production',
      },
    }),
    articlesApiPlugin(),
    mode === "development" && componentTagger(),
  ].filter(Boolean),
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  
  // Otimização de dependências - garantir que React seja pré-empacotado corretamente
  optimizeDeps: {
    include: ['react', 'react-dom', 'react/jsx-runtime'],
    exclude: [],
  },
  
  // Otimizações de performance para produção
  build: {
    // Minificação otimizada - melhor compressão
    minify: 'terser',
    terserOptions: {
      compress: {
        drop_console: mode === 'production', // Remove console.log em produção
        drop_debugger: true,
        pure_funcs: mode === 'production' ? ['console.log', 'console.info', 'console.debug'] : [],
        passes: 3, // Aumentado para 3 passadas - melhor compressão
        dead_code: true,
        unused: true,
        collapse_vars: true,
        reduce_vars: true,
      },
      mangle: {
        safari10: true, // Compatibilidade Safari
        toplevel: mode === 'production', // Mangling de nível superior em produção
      },
      format: {
        comments: false, // Remove comentários
      },
    },
    
    // Otimização de chunk splitting (equivalente ao LiteSpeed Cache - combina e minifica)
    rollupOptions: {
      output: {
        manualChunks: (id) => {
          // Vendor chunk para node_modules
          if (id.includes('node_modules')) {
            // React core em chunk separado (grande, raramente muda)
            if (id.includes('react/') || id.includes('react-dom/')) {
              return 'react-vendor';
            }
            // Radix UI em chunk separado
            if (id.includes('@radix-ui')) {
              return 'radix-ui';
            }
            // React Query em chunk separado
            if (id.includes('@tanstack/react-query')) {
              return 'react-query';
            }
            // React Router em chunk separado
            if (id.includes('react-router')) {
              return 'react-router';
            }
            // Lucide icons em chunk separado
            if (id.includes('lucide-react')) {
              return 'lucide-icons';
            }
            // Outras bibliotecas grandes
            if (id.includes('recharts')) {
              return 'recharts';
            }
            // Vendor comum - combina bibliotecas menores
            return 'vendor';
          }
        },
        // Otimização de nomes de chunks
        chunkFileNames: 'assets/js/[name]-[hash].js',
        entryFileNames: 'assets/js/[name]-[hash].js',
        assetFileNames: (assetInfo) => {
          if (assetInfo.name?.endsWith('.css')) {
            return 'assets/css/[name]-[hash][extname]';
          }
          if (/\.(png|jpe?g|svg|gif|tiff|bmp|ico)$/i.test(assetInfo.name || '')) {
            return 'assets/images/[name]-[hash][extname]';
          }
          return 'assets/[name]-[hash][extname]';
        },
      },
    },
    
    // Otimização de CSS - minificação mais agressiva
    cssCodeSplit: true,
    cssMinify: true,
    
    // Otimização de SVGs (via build do Vite)
    assetsInclude: ['**/*.svg'],
    
    // Otimização de assets
    assetsInlineLimit: 2048, // Reduzido para 2KB - inline apenas assets muito pequenos
    
    // Otimização de source maps (desabilitado em produção para melhor performance)
    sourcemap: mode === 'development',
    
    // Otimização de relatório de tamanho
    reportCompressedSize: true,
    
    // Otimização de tamanho
    chunkSizeWarningLimit: 500, // Alertar sobre chunks grandes
    
    // Otimização de target
    target: 'esnext',
    
    // Otimização de module format
    modulePreload: {
      polyfill: false, // Desabilitar polyfill para melhor performance
    },
  },
  
  // Otimização de preview (produção local)
  preview: {
    port: 8080,
    strictPort: true,
  },
}));
