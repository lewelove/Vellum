<script>
  import { onMount, onDestroy } from "svelte";

  let { colors = [] } = $props();

  let canvasEl;
  let gl;
  let program;
  let animationFrame;
  let startTime;

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
    uniform vec2 iResolution;
    uniform int iColors[8];

    out vec4 fragColor;

    const float SPEED = 0.3;
    const float BLUR_INTENSITY = 0.6;
    const float GRAIN_AMOUNT = 0.06;

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
        uv.x *= iResolution.x / iResolution.y;
        
        vec3 totalColor = vec3(0.0);
        float totalWeight = 0.0;
        float t = iTime * SPEED;
        
        for (int i = 0; i < 8; i++) {
            vec3 color = hexToRgb(iColors[i]);
            float seed = float(i) * 1.618;
            
            vec2 pos = vec2(
                0.5 + 0.4 * sin(t + seed),
                0.5 + 0.4 * cos(t * 0.8 + seed)
            );
            
            pos.x *= iResolution.x / iResolution.y;
            float dist = distance(uv, pos);
            float weight = 1.0 / (pow(dist, 1.8) + 0.01 + (1.0 - BLUR_INTENSITY));
            
            totalColor += color * weight;
            totalWeight += weight;
        }
        
        vec3 finalColor = totalColor / totalWeight;
        float luminance = dot(finalColor, vec3(0.2126, 0.7152, 0.0722));
        vec3 grey = vec3(luminance);
        finalColor = mix(grey, finalColor, 1.4);

        float noise = (random(uv + iTime) - 0.5) * GRAIN_AMOUNT;
        finalColor += noise;

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
    // CRITICAL: Set alpha to false to stabilize the compositor layer
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

    const vertices = new Float32Array([
      -1, -1, 
       1, -1, 
      -1,  1, 
      -1,  1, 
       1, -1, 
       1,  1
    ]);

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

    const timeLoc = gl.getUniformLocation(program, "iTime");
    gl.uniform1f(timeLoc, elapsed);

    const resLoc = gl.getUniformLocation(program, "iResolution");
    gl.uniform2f(resLoc, canvasEl.width, canvasEl.height);

    const colorLoc = gl.getUniformLocation(program, "iColors");
    const intColors = new Int32Array(8);
    for (let i = 0; i < 8; i++) {
      const hex = colors[i] || "#000000";
      intColors[i] = parseInt(hex.replace("#", ""), 16);
    }
    gl.uniform1iv(colorLoc, intColors);

    gl.drawArrays(gl.TRIANGLES, 0, 6);
    animationFrame = requestAnimationFrame(render);
  }

  function handleResize() {
    if (canvasEl) {
      // CRITICAL: Scale canvas by devicePixelRatio to match screen physical pixels
      const dpr = window.devicePixelRatio || 1;
      canvasEl.width = window.innerWidth * dpr;
      canvasEl.height = window.innerHeight * dpr;
    }
  }

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
    z-index: -1;
    pointer-events: none;
    /* Prevent the browser from trying to smooth the canvas element itself */
    image-rendering: pixelated;
  "
></canvas>
