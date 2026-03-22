<script>
  import { onMount, onDestroy } from "svelte";
  
  import vertexShaderSource from "./shaders/quad.vert?raw";
  import fragmentShaderSource from "./shaders/simplex.frag?raw";

  let { colors = [], coverSize = 0, visible = false } = $props();

  let canvasEl;
  let gl;
  let program;
  let animationFrame;
  
  let totalTime = 0;
  let lastFrameTime = 0;
  let isTabVisible = $state(true);
  let randomOffset = Math.random() * 1000.0;

  const intColors = new Int32Array(16);
  let activeColorCount = 0;
  const DEFAULT_PALETTE = ["#1a1a1a", "#242424", "#0d1117", "#161b22"];

  let shouldRender = $derived(visible && isTabVisible);

  $effect(() => {
    const palette = (colors && colors.length > 0) ? colors : DEFAULT_PALETTE;
    activeColorCount = Math.min(palette.length, 16);
    for (let i = 0; i < 16; i++) {
      if (i < activeColorCount) {
        intColors[i] = parseInt(palette[i].replace("#", ""), 16);
      } else {
        intColors[i] = 0;
      }
    }
  });

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
      premultipliedAlpha: false,
      preserveDrawingBuffer: false
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

    lastFrameTime = performance.now();
    startLoop();
  }

  function startLoop() {
    if (animationFrame) cancelAnimationFrame(animationFrame);
    lastFrameTime = performance.now();
    render();
  }

  function render() {
    if (!gl) return;

    if (!shouldRender) {
      animationFrame = requestAnimationFrame(render);
      return;
    }

    const now = performance.now();
    const delta = (now - lastFrameTime) / 1000;
    lastFrameTime = now;
    totalTime += delta;

    gl.viewport(0, 0, canvasEl.width, canvasEl.height);
    gl.useProgram(program);

    gl.uniform1f(gl.getUniformLocation(program, "iTime"), totalTime);
    gl.uniform1f(gl.getUniformLocation(program, "iRandom"), randomOffset);
    gl.uniform2f(gl.getUniformLocation(program, "iResolution"), canvasEl.width, canvasEl.height);
    
    const dpr = window.devicePixelRatio || 1;
    gl.uniform1f(gl.getUniformLocation(program, "iCoverSize"), coverSize * dpr);

    gl.uniform1iv(gl.getUniformLocation(program, "iColors"), intColors);
    gl.uniform1i(gl.getUniformLocation(program, "iCount"), activeColorCount);

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

  function handleVisibilityChange() {
    isTabVisible = !document.hidden;
    if (isTabVisible) {
      lastFrameTime = performance.now();
    }
  }

  $effect(() => {
    if (colors || coverSize) {
      handleResize();
    }
  });

  $effect(() => {
    if (shouldRender) {
      lastFrameTime = performance.now();
    }
  });

  onMount(() => {
    handleResize();
    initGL();
    window.addEventListener("resize", handleResize);
    document.addEventListener("visibilitychange", handleVisibilityChange);
  });

  onDestroy(() => {
    if (animationFrame) cancelAnimationFrame(animationFrame);
    window.removeEventListener("resize", handleResize);
    document.removeEventListener("visibilitychange", handleVisibilityChange);
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
