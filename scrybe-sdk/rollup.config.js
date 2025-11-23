import typescript from '@rollup/plugin-typescript';
import resolve from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import { terser } from 'rollup-plugin-terser';

const production = process.env.NODE_ENV === 'production';

export default [
  // UMD build (for browser <script> tags)
  {
    input: 'src/index.ts',
    output: {
      file: 'dist/scrybe.umd.js',
      format: 'umd',
      name: 'Scrybe',
      sourcemap: true,
    },
    plugins: [
      resolve(),
      commonjs(),
      typescript({
        tsconfig: './tsconfig.json',
        declaration: false,
      }),
      production && terser(),
    ],
  },
  
  // ESM build (for modern bundlers)
  {
    input: 'src/index.ts',
    output: {
      file: 'dist/scrybe.esm.js',
      format: 'esm',
      sourcemap: true,
    },
    plugins: [
      resolve(),
      commonjs(),
      typescript({
        tsconfig: './tsconfig.json',
        declaration: true,
        declarationDir: 'dist',
      }),
    ],
  },
  
  // CJS build (for Node.js)
  {
    input: 'src/index.ts',
    output: {
      file: 'dist/scrybe.cjs.js',
      format: 'cjs',
      sourcemap: true,
    },
    plugins: [
      resolve(),
      commonjs(),
      typescript({
        tsconfig: './tsconfig.json',
        declaration: false,
      }),
    ],
  },
];
