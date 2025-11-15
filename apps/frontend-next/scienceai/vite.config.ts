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
  plugins: [react(), articlesApiPlugin(), mode === "development" && componentTagger()].filter(Boolean),
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  
  // Otimizações de performance para produção
  build: {
    // Minificação otimizada
    minify: 'terser',
    terserOptions: {
      compress: {
        drop_console: mode === 'production', // Remove console.log em produção
        drop_debugger: true,
        pure_funcs: mode === 'production' ? ['console.log', 'console.info'] : [],
      },
    },
    
    // Otimização de chunk splitting
    rollupOptions: {
      output: {
        manualChunks: (id) => {
          // Vendor chunk para node_modules
          if (id.includes('node_modules')) {
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
            // Vendor comum
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
    
    // Otimização de CSS
    cssCodeSplit: true,
    cssMinify: true,
    
    // Otimização de assets
    assetsInlineLimit: 4096, // Inline assets < 4KB
    
    // Otimização de source maps (apenas em dev)
    sourcemap: mode === 'development',
    
    // Otimização de tamanho
    chunkSizeWarningLimit: 1000,
    
    // Otimização de compressão
    reportCompressedSize: true,
  },
  
  // Otimização de preview (produção local)
  preview: {
    port: 8080,
    strictPort: true,
  },
}));
