/**
 * WebGL fingerprinting collector
 * 
 * Generates a unique fingerprint based on WebGL rendering characteristics
 * and GPU information.
 */

import type { WebGLFingerprint } from '../types';
import { sha256 } from '../utils/hash';

export class WebGLCollector {
  /**
   * Collect WebGL fingerprint
   * 
   * @returns Promise resolving to WebGL fingerprint
   */
  async collect(): Promise<WebGLFingerprint | undefined> {
    try {
      const canvas = document.createElement('canvas');
      const gl = canvas.getContext('webgl') || canvas.getContext('experimental-webgl');

      if (!gl) {
        return undefined;
      }

      // Get GPU info
      const debugInfo = gl.getExtension('WEBGL_debug_renderer_info');
      const vendor = debugInfo
        ? gl.getParameter(debugInfo.UNMASKED_VENDOR_WEBGL)
        : 'unknown';
      const renderer = debugInfo
        ? gl.getParameter(debugInfo.UNMASKED_RENDERER_WEBGL)
        : 'unknown';

      // Get supported extensions
      const extensions = gl.getSupportedExtensions() || [];

      // Get WebGL parameters for fingerprinting
      const parameters = this.collectParameters(gl as WebGLRenderingContext);

      // Render test pattern for additional entropy
      const renderHash = await this.renderPattern(gl as WebGLRenderingContext, canvas);

      // Combine all data and hash
      const combined = JSON.stringify({
        vendor,
        renderer,
        extensions: extensions.sort(),
        parameters,
        renderHash,
      });

      const hash = await sha256(combined);

      return {
        hash,
        vendor,
        renderer,
        supportedExtensions: extensions.slice(0, 20), // Limit to prevent bloat
      };
    } catch (error) {
      return undefined;
    }
  }

  /**
   * Collect WebGL parameters
   * 
   * @private
   */
  private collectParameters(gl: WebGLRenderingContext): Record<string, any> {
    const params: Record<string, any> = {};

    const keys = [
      'VERSION',
      'SHADING_LANGUAGE_VERSION',
      'VENDOR',
      'RENDERER',
      'MAX_TEXTURE_SIZE',
      'MAX_VIEWPORT_DIMS',
      'MAX_VERTEX_ATTRIBS',
      'MAX_VERTEX_UNIFORM_VECTORS',
      'MAX_VARYING_VECTORS',
      'MAX_COMBINED_TEXTURE_IMAGE_UNITS',
      'MAX_VERTEX_TEXTURE_IMAGE_UNITS',
      'MAX_TEXTURE_IMAGE_UNITS',
      'MAX_FRAGMENT_UNIFORM_VECTORS',
      'ALIASED_LINE_WIDTH_RANGE',
      'ALIASED_POINT_SIZE_RANGE',
    ];

    for (const key of keys) {
      try {
        const value = gl.getParameter((gl as any)[key]);
        if (value !== null) {
          params[key] = Array.isArray(value) ? Array.from(value) : value;
        }
      } catch {
        // Ignore errors for unsupported parameters
      }
    }

    return params;
  }

  /**
   * Render a test pattern and return hash
   * 
   * @private
   */
  private async renderPattern(
    gl: WebGLRenderingContext,
    canvas: HTMLCanvasElement
  ): Promise<string> {
    // Simple triangle rendering for additional GPU fingerprint
    const vertexShader = gl.createShader(gl.VERTEX_SHADER);
    const fragmentShader = gl.createShader(gl.FRAGMENT_SHADER);

    if (!vertexShader || !fragmentShader) {
      return 'no_render';
    }

    // Vertex shader
    gl.shaderSource(
      vertexShader,
      `
      attribute vec2 position;
      void main() {
        gl_Position = vec4(position, 0.0, 1.0);
      }
    `
    );
    gl.compileShader(vertexShader);

    // Fragment shader
    gl.shaderSource(
      fragmentShader,
      `
      precision mediump float;
      void main() {
        gl_FragColor = vec4(1.0, 0.0, 0.5, 1.0);
      }
    `
    );
    gl.compileShader(fragmentShader);

    // Create program
    const program = gl.createProgram();
    if (!program) {
      return 'no_render';
    }

    gl.attachShader(program, vertexShader);
    gl.attachShader(program, fragmentShader);
    gl.linkProgram(program);
    gl.useProgram(program);

    // Set up geometry
    const vertices = new Float32Array([-0.5, -0.5, 0.5, -0.5, 0.0, 0.5]);

    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(gl.ARRAY_BUFFER, vertices, gl.STATIC_DRAW);

    const position = gl.getAttribLocation(program, 'position');
    gl.enableVertexAttribArray(position);
    gl.vertexAttribPointer(position, 2, gl.FLOAT, false, 0, 0);

    // Render
    gl.clearColor(0, 0, 0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.drawArrays(gl.TRIANGLES, 0, 3);

    // Get rendered data
    const pixels = new Uint8Array(canvas.width * canvas.height * 4);
    gl.readPixels(0, 0, canvas.width, canvas.height, gl.RGBA, gl.UNSIGNED_BYTE, pixels);

    // Hash the pixel data (sample to reduce size)
    const sample = Array.from(pixels.slice(0, 100)).join(',');
    return await sha256(sample);
  }
}
