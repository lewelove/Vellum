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

    const float SPEED = 0.10;
    const float SATURATION = 1.2;
    const float GRAIN_AMOUNT = 0.02;
    const float BLEND_SOFTNESS = 0.12;

    vec3 hexToRgb(int hex) {
        float r = float((hex >> 16) & 0xFF) / 255.0;
        float g = float((hex >> 8) & 0xFF) / 255.0;
        float b = float(hex & 0xFF) / 255.0;
        return vec3(r, g, b);
    }

    float random(vec2 st) {
        return fract(sin(dot(st.xy, vec2(12.9898,78.233))) * 43758.5453123);
    }

    void main() {
        vec2 uv = gl_FragCoord.xy / iResolution.xy;
        float aspect = iResolution.x / iResolution.y;
        uv.x *= aspect;
        
        float t = (iTime + iRandom) * SPEED;
        
        vec2 center = vec2(0.5 * aspect, 0.5);
        vec2 diff = uv - center;
        float dBox = max(abs(diff.x), abs(diff.y));
        float cRad = (iCoverSize / iResolution.y) * 0.5;
        
        float repel = smoothstep(cRad + 0.4, cRad - 0.2, dBox);
        vec2 normDiff = diff / (length(diff) + 0.0001);
        vec2 p = uv + normDiff * repel * 0.35;
        
        p *= 2.2;
        
        for(float i = 1.0; i < 5.0; i++) {
            vec2 newp = p;
            newp.x += 0.6 / i * cos(i * 1.8 * p.y + t * 1.5);
            newp.y += 0.6 / i * sin(i * 1.2 * p.x - t * 1.3);
            p = newp;
        }

        float noiseVal = sin(p.x * 0.45 + p.y * 0.35 + t * 0.5) * 0.5 + 0.5;

        float totalRatio = 0.0;
        for(int i = 0; i < 8; i++) {
            totalRatio += iRatios[i];
        }

        vec3 finalColor = vec3(0.0);
        float currentStart = 0.0;
        float totalWeight = 0.0;

        for (int i = 0; i < 8; i++) {
            if (iRatios[i] <= 0.0) continue;
            
            float normRatio = iRatios[i] / totalRatio;
            float currentEnd = currentStart + normRatio;
            
            float weight = smoothstep(currentStart - BLEND_SOFTNESS, currentStart + BLEND_SOFTNESS, noiseVal) * 
                           (1.0 - smoothstep(currentEnd - BLEND_SOFTNESS, currentEnd + BLEND_SOFTNESS, noiseVal));
            
            finalColor += hexToRgb(iColors[i]) * weight;
            totalWeight += weight;
            
            currentStart = currentEnd;
        }

        finalColor /= (totalWeight + 0.00001);

        float luminance = dot(finalColor, vec3(0.2126, 0.7152, 0.0722));
        finalColor = mix(vec3(luminance), finalColor, SATURATION);

        float noise = (random(gl_FragCoord.xy / iResolution.xy + iTime) - 0.5) * GRAIN_AMOUNT;
        finalColor += noise;

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
        floatRatios[i] = item.ratio || 0.1;
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
