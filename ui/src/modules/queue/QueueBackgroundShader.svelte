<script>
  import { onMount, onDestroy } from "svelte";

  let { colors = [], coverSize = 0 } = $props();

  let canvasEl;
  let gl;
  let program;
  let animationFrame;
  let startTime;
  let randomOffset = Math.random() * 1000.0;

  const DEFAULT_PALETTE = [
    { color: "#1a1a1a", ratio: 0.4 },
    { color: "#242424", ratio: 0.3 },
    { color: "#0d1117", ratio: 0.2 },
    { color: "#161b22", ratio: 0.1 }
  ];

  const vertexShaderSource = `#version 300 es
    in vec2 position;
    void main() {
      gl_Position = vec4(position, 0.0, 1.0);
    }
  `;

  const fragmentShaderSource = `#version 300 es
    precision highp float;
    precision highp int;

    uniform float iTime;
    uniform float iRandom;
    uniform vec2 iResolution;
    uniform float iCoverSize;
    uniform int iColors[8];
    uniform float iRatios[8];

    out vec4 fragColor;

    const float SPEED = 0.04;
    const float GRAIN_AMOUNT = 0.02;

    vec3 hexToRgb(int hex) {
        float r = float((hex >> 16) & 0xFF) / 255.0;
        float g = float((hex >> 8) & 0xFF) / 255.0;
        float b = float(hex & 0xFF) / 255.0;
        return vec3(r, g, b);
    }

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
        for (int i = 0; i < 4; i++) {
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

        float totalRatio = 0.0;
        for(int i = 0; i < 8; i++) {
            totalRatio += iRatios[i];
        }

        vec3 finalColor = vec3(0.0);
        float cumulative = 0.0;
        float softness = 0.02; 

        for(int i = 0; i < 8; i++) {
            if (iRatios[i] <= 0.0) continue;
            
            float normRatio = iRatios[i] / totalRatio;
            float nextCumulative = cumulative + normRatio;
            
            float weight = smoothstep(cumulative - softness, cumulative + softness, val) - 
                           smoothstep(nextCumulative - softness, nextCumulative + softness, val);
            
            finalColor += hexToRgb(iColors[i]) * max(0.0, weight);
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
      gl.deleteShader(shader);
      return null;
    }
    return shader;
  }

  function initGL() {
    gl = canvasEl.getContext("webgl2", { 
      alpha: false, 
      antialias: true,
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

    startTime = performance.now();
    render();
  }

  function render() {
    if (!gl) return;
    const now = performance.now();
    const elapsed = (now - startTime) / 1000;

    gl.viewport(0, 0, canvasEl.width, canvasEl.height);
    gl.useProgram(program);

    gl.uniform1f(gl.getUniformLocation(program, "iTime"), elapsed);
    gl.uniform1f(gl.getUniformLocation(program, "iRandom"), randomOffset);
    gl.uniform2f(gl.getUniformLocation(program, "iResolution"), canvasEl.width, canvasEl.height);
    
    const dpr = window.devicePixelRatio || 1;
    gl.uniform1f(gl.getUniformLocation(program, "iCoverSize"), coverSize * dpr);

    const activePalette = colors.length > 0 ? colors : DEFAULT_PALETTE;
    const intColors = new Int32Array(8);
    const floatRatios = new Float32Array(8);
    
    for (let i = 0; i < 8; i++) {
      const item = activePalette[i];
      if (item && item.color) {
        intColors[i] = parseInt(item.color.replace("#", ""), 16);
        floatRatios[i] = parseFloat(item.ratio) || 0.0;
      } else {
        intColors[i] = 0;
        floatRatios[i] = 0.0;
      }
    }
    
    gl.uniform1iv(gl.getUniformLocation(program, "iColors"), intColors);
    gl.uniform1fv(gl.getUniformLocation(program, "iRatios"), floatRatios);

    gl.drawArrays(gl.TRIANGLES, 0, 6);
    animationFrame = requestAnimationFrame(render);
  }

  function handleResize() {
    if (canvasEl) {
      const dpr = window.devicePixelRatio || 1;
      canvasEl.width = window.innerWidth * dpr;
      canvasEl.height = window.innerHeight * dpr;
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
