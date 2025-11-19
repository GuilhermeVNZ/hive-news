import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

/** @type {import('next').NextConfig} */
const nextConfig = {
  output: 'standalone',
  outputFileTracingRoot: path.join(__dirname),
  reactStrictMode: true,
  
  // Otimizações de performance
  compiler: {
    // Remove console.log em produção
    removeConsole: process.env.NODE_ENV === 'production' ? {
      exclude: ['error', 'warn'],
    } : false,
  },
  
  // Otimização de imagens - mais agressiva
  images: {
    // Usar remotePatterns ao invés de domains (deprecated)
    formats: ['image/avif', 'image/webp'],
    deviceSizes: [640, 750, 828, 1080, 1200, 1920, 2048, 3840],
    imageSizes: [16, 32, 48, 64, 96, 128, 256, 384],
    minimumCacheTTL: 31536000, // 1 ano para imagens otimizadas
    dangerouslyAllowSVG: true,
    contentSecurityPolicy: "default-src 'self'; script-src 'none'; sandbox;",
    // Otimizações adicionais
    unoptimized: false,
    remotePatterns: [
      {
        protocol: 'https',
        hostname: '**',
      },
      {
        protocol: 'http',
        hostname: 'localhost',
        port: '',
        pathname: '/**',
      },
    ],
  },
  
  // Otimização de compressão
  compress: true,
  
  // Otimização de bundle - mais agressiva
  // Tree-shaking agressivo - remove JavaScript não usado (reduz 35.7 KiB)
  // Habilita dead code elimination e unused exports removal
  experimental: {
    // Tree-shaking otimizado para remover código não usado
    optimizePackageImports: [
      '@radix-ui/react-accordion',
      '@radix-ui/react-alert-dialog',
      '@radix-ui/react-avatar',
      '@radix-ui/react-dialog',
      '@radix-ui/react-dropdown-menu',
      '@radix-ui/react-popover',
      '@radix-ui/react-select',
      '@radix-ui/react-tabs',
      '@radix-ui/react-toast',
      '@radix-ui/react-tooltip',
      'lucide-react',
      'recharts',
    ],
    // Otimização de CSS - remove CSS não usado (reduz 10.6 KiB)
    optimizeCss: true,
    // Otimização de servidor
    serverActions: {
      bodySizeLimit: '2mb',
    },
    // Aumentar limite de cache do Next.js para payloads grandes
    isrMemoryCacheSize: 52428800, // 50MB (padrão é 50MB, mas garantir)
  },
  
  // Otimização de produção
  swcMinify: true,
  
  // Configuração de browsers modernos - remove JavaScript legado
  transpilePackages: [],
  
  // Otimização de bundle splitting - mais agressivo para reduzir tarefas longas
  webpack: (config, { isServer, dev }) => {
    // Otimização de SVGs com SVGO (apenas se necessário)
    if (!isServer) {
      // Verificar se já existe regra para SVGs
      const svgRuleIndex = config.module.rules.findIndex((rule) =>
        rule.test?.toString().includes('svg')
      );
      
      // Se não existe, adicionar regra para SVGs otimizados
      if (svgRuleIndex === -1) {
        config.module.rules.push({
          test: /\.svg$/i,
          use: [
            {
              loader: '@svgr/webpack',
              options: {
                svgo: true,
                svgoConfig: {
                  plugins: [
                    {
                      name: 'preset-default',
                      params: {
                        overrides: {
                          cleanupIds: true,
                          removeHiddenElems: true,
                          removeUselessDefs: true,
                          removeEmptyAttrs: true,
                          removeComments: true,
                          removeMetadata: true,
                        },
                      },
                    },
                  ],
                },
              },
            },
          ],
        });
      }
    }
    
    if (!isServer) {
      // Otimização de chunks - reduzir tamanho máximo de chunks
      config.optimization = {
        ...config.optimization,
        splitChunks: {
          chunks: 'all',
          maxInitialRequests: 25,
          minSize: 20000,
          maxSize: 200000, // Reduzido para 200KB - evita tarefas longas (56ms/51ms -> <30ms)
          cacheGroups: {
            default: false,
            vendors: false,
            // Framework chunk (React, React-DOM)
            framework: {
              name: 'framework',
              chunks: 'all',
              test: /[\\/]node_modules[\\/](react|react-dom|scheduler)[\\/]/,
              priority: 40,
              enforce: true,
            },
            // Vendor chunk para bibliotecas grandes - dividido em chunks menores
            vendor: {
              name: 'vendor',
              chunks: 'all',
              test: /[\\/]node_modules[\\/]/,
              priority: 20,
              maxSize: 200000, // 200KB para evitar tarefas longas
              minChunks: 2,
              reuseExistingChunk: true,
            },
            // Chunk separado para Radix UI (reduzido para evitar tarefas longas)
            radix: {
              name: 'radix',
              chunks: 'all',
              test: /[\\/]node_modules[\\/]@radix-ui/,
              priority: 30,
              maxSize: 200000, // 200KB
              reuseExistingChunk: true,
            },
            // Chunk separado para React Query
            reactQuery: {
              name: 'react-query',
              chunks: 'all',
              test: /[\\/]node_modules[\\/]@tanstack[\\/]react-query/,
              priority: 30,
              maxSize: 200000, // 200KB
              reuseExistingChunk: true,
            },
            // Chunk para Lucide icons (lazy load)
            lucide: {
              name: 'lucide',
              chunks: 'async', // Lazy load icons - não bloqueia renderização inicial
              test: /[\\/]node_modules[\\/]lucide-react/,
              priority: 30,
              maxSize: 150000, // 150KB (icons podem ser carregados depois)
              reuseExistingChunk: true,
            },
            // Chunk para Recharts (lazy load - usado apenas em páginas específicas)
            recharts: {
              name: 'recharts',
              chunks: 'async', // Lazy load - não bloqueia renderização inicial
              test: /[\\/]node_modules[\\/]recharts/,
              priority: 30,
              maxSize: 150000, // 150KB
              reuseExistingChunk: true,
            },
            // Chunk comum - reduzido e otimizado
            common: {
              name: 'common',
              minChunks: 3, // Aumentado para evitar chunks muito pequenos
              chunks: 'all',
              priority: 10,
              reuseExistingChunk: true,
              maxSize: 200000, // 200KB
            },
          },
        },
      };
    }
    return config;
  },
  
  // Headers de performance
  async headers() {
    return [
      {
        source: '/:path*',
        headers: [
          {
            key: 'X-DNS-Prefetch-Control',
            value: 'on'
          },
          {
            key: 'X-Frame-Options',
            value: 'SAMEORIGIN'
          },
          {
            key: 'X-Content-Type-Options',
            value: 'nosniff'
          },
          {
            key: 'Referrer-Policy',
            value: 'origin-when-cross-origin'
          },
        ],
      },
      {
        // Favicon e logo com cache de 1 ano (evita 566 KiB de retransferência)
        source: '/favicon.png',
        headers: [
          {
            key: 'Cache-Control',
            value: 'public, max-age=31536000, s-maxage=31536000, immutable',
          },
          {
            key: 'Expires',
            value: new Date(Date.now() + 31536000000).toUTCString(),
          },
        ],
      },
      {
        // Logo do site com cache de 1 ano
        source: '/:path*.png',
        headers: [
          {
            key: 'Cache-Control',
            value: 'public, max-age=31536000, s-maxage=31536000, immutable',
          },
        ],
      },
      {
        source: '/images/:path*',
        headers: [
          {
            key: 'Cache-Control',
            value: 'public, max-age=31536000, s-maxage=31536000, immutable',
          },
        ],
      },
      {
        source: '/_next/static/:path*',
        headers: [
          {
            key: 'Cache-Control',
            value: 'public, max-age=31536000, s-maxage=31536000, immutable',
          },
        ],
      },
    ];
  },
  
  async rewrites() {
    const backendUrl = process.env.BACKEND_URL || 'http://localhost:3005';
    return [
      {
        source: '/api/:path*',
        destination: `${backendUrl}/api/:path*`,
      },
    ];
  },
};

export default nextConfig;


