import { defineConfig } from 'eslint-define-config';

export default defineConfig({
  extends: ['eslint:recommended'],
  parser: '@typescript-eslint/parser',
  plugins: ['@typescript-eslint'],
  rules: {
    '@typescript-eslint/no-unused-vars': [
      'error',
      { argsIgnorePattern: '^_' }
    ],
    '@typescript-eslint/explicit-function-return-type': 'warn',
    '@typescript-eslint/no-explicit-any': 'warn'
  },
  ignorePatterns: [
    'node_modules',
    'dist',
    'build',
    '.next',
    'coverage'
  ]
});


