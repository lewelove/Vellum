<script>
  import { onMount, onDestroy } from "svelte";

  let { colors = [], coverSize = 0 } = $props();

  let canvasEl;
  let gl;
  let program;
  let animationFrame;
  let startTime;
  let randomOffset = Math.random() * 1000.0;
  let uniforms = {};

  const DEFAULT_PALETTE = [
    { color: "#1a1a1a", ratio: 0.4 },
    { color: "#242424", ratio: 0.3 },
    { color: "#0d1117", ratio: 0.2 },
    { color: "#161b22", ratio: 0.1 }
  ];

  // Renders at half logical resolution. On a 1080p display, this renders at 960x540.
  // This drastically reduces fill-rate pressure on integrated GPUs by 75%.
  const RENDER_SCALE = 1;

  const vertexShaderSource = `#version 300 es
    in vec2 position;
    void main() {
      gl_Position = vec4(position, 0.0, 1.0);
    }
  `;

  // highp float is strictly required. mediump collapses the hash() mantissa.
  const fragmentShaderSource = `#version 300 es
    precision highp float;
    precision highp int;

    uniform float iTime;
    uniform float iRandom;
    uniform vec2 iResolution;
    uniform float iCoverSize;
    uniform vec3 iColors[8];
    uniform float iRatios[8];

    out vec4 fragColor;

    const float SPEED = 0.04;
    const float GRAIN_AMOUNT = 0.02;

    float hash(vec2 p) {
        p = fract(p * vec2(123.34, 456.21));
        p += dot(p, p + 45.32);
        return fract(p.x * p.y);
    }

    float noise(vec2 p) {
        vec2 i = floor(p);
        vec2 f = fract(p);
        f = f * f * (3.0 - 2.0 * f);
        float a = hash(i);
        float b = hash(i + vec2(1.0, 0.0));
        float c = hash(i + vec2(0.0, 1.0));
        float d = hash(i + vec2(1.0, 1.0));
        return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
    }

    float fbm(vec2 p) {
        float v = 0.0;
        float a = 0.5;
        // 3 octaves is sufficient for fluid blobs.
        for (int i = 0; i < 3; i++) {
            v += a * noise(p);
            p *= 2.0;
            a *= 0.5;
        }
        return v;
    }

    void main() {
        vec2 uv = gl_FragCoord.xy / iResolution.xy;
        float aspect = iResolution.x / iResolution.y;
        vec2 p = uv;
        p.x *= aspect;
        
        float t = (iTime + iRandom) * SPEED;
        
        vec2 center = vec2(0.5 * aspect, 0.5);
        float dist = length(p - center);
        float cRad = (iCoverSize / iResolution.y) * 0.5;
        
        p += (p - center) * smoothstep(cRad + 0.5, cRad - 0.1, dist) * 0.3;

        float val = fbm(p * 1.5 + t);
        val = clamp(val, 0.0, 1.0);

        vec3 finalColor = vec3(0.0);
        float cumulative = 0.0;
        const float softness = 0.02; 

        for(int i = 0; i < 8; i++) {
            if (iRatios[i] <= 0.0) continue;
            
            float nextCumulative = cumulative + iRatios[i];
            
            float weight = smoothstep(cumulative - softness, cumulative + softness, val) - 
                           smoothstep(nextCumulative - softness, nextCumulative + softness, val);
            
            finalColor += iColors[i] * max(0.0, weight);
            cumulative = nextCumulative;
        }

        float noiseFloor = (hash(uv + iTime) - 0.5) * GRAIN_AMOUNT;
        finalColor += noiseFloor;
        
        fragColor = vec4(finalColor, 1.0);
    }
  `;

  function createShader(gl, type, source) {
    const shader = gl.createShader(type);
    gl.shaderSource(shader, source);
    gl.compileShader(shader);
    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
      console.error(gl.getShaderInfoLog(shader));
      gl.deleteShader(shader);
      return null;
    }
    return shader;
  }

  function initGL() {
    gl = canvasEl.getContext("webgl2", { 
      alpha: false, 
      antialias: false,
      premultipliedAlpha: false 
    });
    
    if (!gl) return;

    const vs = createShader(gl, gl.VERTEX_SHADER, vertexShaderSource);
    const fs = createShader(gl, gl.FRAGMENT_SHADER, fragmentShaderSource);

    program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);

    const vertices = new Float32Array([-1, -1, 1, -1, -1, 1, -1, 1, 1, -1, 1, 1]);
    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(gl.ARRAY_BUFFER, vertices, gl.STATIC_DRAW);

    const positionLoc = gl.getAttribLocation(program, "position");
    gl.enableVertexAttribArray(positionLoc);
    gl.vertexAttribPointer(positionLoc, 2, gl.FLOAT, false, 0, 0);

    gl.useProgram(program);

    // Cache uniform locations to avoid expensive driver lookups every frame
    uniforms.iTime = gl.getUniformLocation(program, "iTime");
    uniforms.iRandom = gl.getUniformLocation(program, "iRandom");
    uniforms.iResolution = gl.getUniformLocation(program, "iResolution");
    uniforms.iCoverSize = gl.getUniformLocation(program, "iCoverSize");
    uniforms.iColors = gl.getUniformLocation(program, "iColors[0]") || gl.getUniformLocation(program, "iColors");
    uniforms.iRatios = gl.getUniformLocation(program, "iRatios[0]") || gl.getUniformLocation(program, "iRatios");

    startTime = performance.now();
    render();
  }

  function render() {
    if (!gl) return;
    const now = performance.now();
    const elapsed = (now - startTime) / 1000;

    gl.viewport(0, 0, canvasEl.width, canvasEl.height);
    
    gl.uniform1f(uniforms.iTime, elapsed);
    gl.uniform1f(uniforms.iRandom, randomOffset);
    gl.uniform2f(uniforms.iResolution, canvasEl.width, canvasEl.height);
    gl.uniform1f(uniforms.iCoverSize, coverSize * RENDER_SCALE);

    const activePalette = colors.length > 0 ? colors : DEFAULT_PALETTE;
    const floatColors = new Float32Array(8 * 3);
    const floatRatios = new Float32Array(8);
    
    let totalRatio = 0.0;
    for (let i = 0; i < 8; i++) {
      const item = activePalette[i];
      if (item && item.color) {
        totalRatio += parseFloat(item.ratio) || 0.0;
      }
    }
    if (totalRatio === 0.0) totalRatio = 1.0;

    for (let i = 0; i < 8; i++) {
      const item = activePalette[i];
      if (item && item.color) {
        const hex = parseInt(item.color.replace("#", ""), 16);
        floatColors[i * 3]     = ((hex >> 16) & 0xFF) / 255.0;
        floatColors[i * 3 + 1] = ((hex >> 8) & 0xFF) / 255.0;
        floatColors[i * 3 + 2] = (hex & 0xFF) / 255.0;
        floatRatios[i] = (parseFloat(item.ratio) || 0.0) / totalRatio;
      } else {
        floatColors[i * 3]     = 0.0;
        floatColors[i * 3 + 1] = 0.0;
        floatColors[i * 3 + 2] = 0.0;
        floatRatios[i] = 0.0;
      }
    }
    
    gl.uniform3fv(uniforms.iColors, floatColors);
    gl.uniform1fv(uniforms.iRatios, floatRatios);

    gl.drawArrays(gl.TRIANGLES, 0, 6);
    animationFrame = requestAnimationFrame(render);
  }

  function handleResize() {
    if (canvasEl) {
      canvasEl.width = Math.max(1, Math.floor(window.innerWidth * RENDER_SCALE));
      canvasEl.height = Math.max(1, Math.floor(window.innerHeight * RENDER_SCALE));
    }
  }

  $effect(() => {
    if (colors || coverSize) {
      handleResize();
    }
  });

  onMount(() => {
    handleResize();
    initGL();
    window.addEventListener("resize", handleResize);
  });

  onDestroy(() => {
    if (animationFrame) cancelAnimationFrame(animationFrame);
    window.removeEventListener("resize", handleResize);
  });
</script>

<canvas
  bind:this={canvasEl}
  style="
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    z-index: 0;
    pointer-events: none;
  "
></canvas>
