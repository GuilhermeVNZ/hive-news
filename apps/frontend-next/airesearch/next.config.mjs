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
    domains: ['localhost'],
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
    ],
  },
  
  // Otimização de compressão
  compress: true,
  
  // Otimização de bundle - mais agressiva
  experimental: {
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
    // Otimização de CSS
    optimizeCss: true,
    // Otimização de servidor
    serverActions: {
      bodySizeLimit: '2mb',
    },
  },
  
  // Otimização de produção
  swcMinify: true,
  
  // Configuração de browsers modernos - remove JavaScript legado
  transpilePackages: [],
  
  // Otimização de bundle splitting - mais agressivo para reduzir tarefas longas
  webpack: (config, { isServer, dev }) => {
    if (!isServer) {
      // Otimização de chunks - reduzir tamanho máximo de chunks
      config.optimization = {
        ...config.optimization,
        splitChunks: {
          chunks: 'all',
          maxInitialRequests: 25,
          minSize: 20000,
          maxSize: 244000, // Reduzido para evitar chunks grandes que causam tarefas longas
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
            // Vendor chunk para bibliotecas grandes
            vendor: {
              name: 'vendor',
              chunks: 'all',
              test: /[\\/]node_modules[\\/]/,
              priority: 20,
              maxSize: 244000,
            },
            // Chunk separado para Radix UI
            radix: {
              name: 'radix',
              chunks: 'all',
              test: /[\\/]node_modules[\\/]@radix-ui/,
              priority: 30,
              maxSize: 244000,
            },
            // Chunk separado para React Query
            reactQuery: {
              name: 'react-query',
              chunks: 'all',
              test: /[\\/]node_modules[\\/]@tanstack[\\/]react-query/,
              priority: 30,
              maxSize: 244000,
            },
            // Chunk para Lucide icons
            lucide: {
              name: 'lucide',
              chunks: 'all',
              test: /[\\/]node_modules[\\/]lucide-react/,
              priority: 30,
              maxSize: 244000,
            },
            // Chunk para Recharts
            recharts: {
              name: 'recharts',
              chunks: 'all',
              test: /[\\/]node_modules[\\/]recharts/,
              priority: 30,
              maxSize: 244000,
            },
            // Chunk comum - reduzido
            common: {
              name: 'common',
              minChunks: 2,
              chunks: 'all',
              priority: 10,
              reuseExistingChunk: true,
              maxSize: 244000,
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
        source: '/images/:path*',
        headers: [
          {
            key: 'Cache-Control',
            value: 'public, max-age=31536000, immutable',
          },
        ],
      },
      {
        source: '/_next/static/:path*',
        headers: [
          {
            key: 'Cache-Control',
            value: 'public, max-age=31536000, immutable',
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


