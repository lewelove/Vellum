<script>
  import { onMount, onDestroy } from "svelte";

  let { colors = [
  ], coverSize = 0 } = $props();

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
    uniform float iCoverSize;
    uniform int iColors[8];
    uniform float iRatios[8];

    out vec4 fragColor;

    const float SPEED = 0.08;
    const float SATURATION = 1.0;
    const float GRAIN_AMOUNT = 0.02;

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
        
        float t = iTime * SPEED;
        
        vec2 center = vec2(0.5 * aspect, 0.5);
        vec2 diff = uv - center;
        float dBox = max(abs(diff.x), abs(diff.y));
        float cRad = (iCoverSize / iResolution.y) * 0.5;
        
        float repel = smoothstep(cRad + 0.3, cRad - 0.1, dBox);
        vec2 normDiff = diff / (length(diff) + 0.0001);
        vec2 p = uv + normDiff * repel * 0.25;
        
        p *= 3.0;
        
        for(float i = 1.0; i < 5.0; i++) {
            vec2 newp = p;
            newp.x += 0.5 / i * cos(i * 2.5 * p.y + t * 1.2);
            newp.y += 0.5 / i * sin(i * 1.5 * p.x - t * 1.1);
            p = newp;
        }
        
        vec3 totalColor = vec3(0.0);
        float totalWeight = 0.0;
        
        for (int i = 0; i < 8; i++) {
            if (iRatios[i] <= 0.0) continue;
            
            vec3 color = hexToRgb(iColors[i]);
            float ratio = iRatios[i];
            
            float seed = float(i) * 7.312;
            
            vec2 dir = vec2(cos(seed), sin(seed));
            float proj = dot(p, dir);
            
            float wave = sin(proj * 1.5 + seed + t);
            wave = wave * 0.5 + 0.5; 
            
            float sharp = mix(60.0, 1.0, pow(ratio, 0.3));
            
            float weight = pow(wave, sharp) * mix(0.5, 1.5, ratio);
            
            totalColor += color * weight;
            totalWeight += weight;
        }
        
        vec3 finalColor = totalColor / (totalWeight + 0.0001);

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

    const vertices = new Float32Array([
      -1,
      -1,
      1,
      -1,
      -1,
      1,
      -1,
      1,
      1,
      -1,
      1,
      1
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
    
    const coverSizeLoc = gl.getUniformLocation(program, "iCoverSize");
    const dpr = window.devicePixelRatio || 1;
    gl.uniform1f(coverSizeLoc, coverSize * dpr);

    const colorLoc = gl.getUniformLocation(program, "iColors");
    const ratioLoc = gl.getUniformLocation(program, "iRatios");
    
    const intColors = new Int32Array(8);
    const floatRatios = new Float32Array(8);
    
    for (let i = 0; i < 8; i++) {
      const item = colors[i];
      const hex = (item && item.color) ? item.color : "#000000";
      const ratio = (item && item.ratio) ? item.ratio : 0.0;
      
      intColors[i] = parseInt(hex.replace("#", ""), 16);
      floatRatios[i] = ratio;
    }
    
    gl.uniform1iv(colorLoc, intColors);
    gl.uniform1fv(ratioLoc, floatRatios);

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
    image-rendering: auto;
  "
></canvas>
