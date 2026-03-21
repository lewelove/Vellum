<script>
  import { onMount, onDestroy } from "svelte";
  
  import vertexShaderSource from "./shaders/quad.vert?raw";
  // import fragmentShaderSource from "./shaders/liquid_bg.frag?raw";
  import fragmentShaderSource from "./shaders/simplex.frag?raw";
  // import fragmentShaderSource from "./shaders/aurora.frag?raw";
  // import fragmentShaderSource from "../../shaders/deep_liquid.frag?raw"; 
  // import fragmentShaderSource from "../../shaders/radial_drift.frag?raw";

  let { colors = [], coverSize = 0 } = $props();

  let canvasEl;
  let gl;
  let program;
  let animationFrame;
  let startTime;
  let randomOffset = Math.random() * 1000.0;

  const DEFAULT_PALETTE = ["#1a1a1a", "#242424", "#0d1117", "#161b22"];

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

    const activePalette = (colors && colors.length > 0) ? colors : DEFAULT_PALETTE;
    const intColors = new Int32Array(16);
    
    const count = Math.min(activePalette.length, 16);
    for (let i = 0; i < 16; i++) {
      if (i < count) {
        intColors[i] = parseInt(activePalette[i].replace("#", ""), 16);
      } else {
        intColors[i] = 0;
      }
    }
    
    gl.uniform1iv(gl.getUniformLocation(program, "iColors"), intColors);
    gl.uniform1i(gl.getUniformLocation(program, "iCount"), count);

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
