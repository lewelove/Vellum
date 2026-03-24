<script>
  import { onMount, onDestroy } from "svelte";
  
  import vertexShaderSource from "./shaders/quad.vert?raw";
  import fragmentShaderSource from "./shaders/simplex.frag?raw";

  let { colors =[], coverSize = 0, visible = false, isPlaying = false } = $props();

  let canvasEl;
  let gl;
  let program;
  let animationFrame;
  
  let totalTime = 0;
  let lastFrameTime = 0;
  let isTabVisible = $state(true);
  let randomOffset = Math.random() * 1000.0;

  const intColors = new Int32Array(16);
  const floatRatios = new Float32Array(16);
  let activeColorCount = 0;
  const DEFAULT_PALETTE = ["#242424"];

  let needsRedraw = true;

  $effect(() => {
    const palette = (colors && colors.length > 0) ? colors : DEFAULT_PALETTE;
    activeColorCount = Math.min(palette.length, 16);
    
    // Check if the current palette schema provides ratios
    let hasRatios = false;
    for (let i = 0; i < activeColorCount; i++) {
      if (Array.isArray(palette[i]) && palette[i].length > 1) {
        hasRatios = true;
        break;
      }
    }

    for (let i = 0; i < 16; i++) {
      if (i < activeColorCount) {
        const c = palette[i];
        const hex = Array.isArray(c) ? c[0] : (c.hex || c);
        
        let ratio = 0.0;
        if (hasRatios) {
          ratio = Array.isArray(c) ? parseFloat(c[1]) : 0.0;
        } else {
          // Fallback uniform decay for legacy palettes
          ratio = 1.0 / (i + 1.0);
        }

        intColors[i] = parseInt(hex.replace("#", ""), 16);
        floatRatios[i] = ratio;
      } else {
        intColors[i] = 0;
        floatRatios[i] = 0.0;
      }
    }
    needsRedraw = true;
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

    // Discard looping calculations completely while inactive/hidden to avoid time skipping
    if (!visible || !isTabVisible) {
      animationFrame = requestAnimationFrame(render);
      return;
    }

    const now = performance.now();
    let timeAdvanced = false;

    if (isPlaying) {
      let delta = (now - lastFrameTime) / 1000;
      
      // Delta Clamping: If more than 100ms have passed between frames, the OS/Compositor
      // likely suspended the window (e.g., Hyprland unmapped the special workspace).
      // We clamp the delta to a standard ~60fps frame (0.016s) to prevent a massive jump.
      if (delta > 0.1) {
        delta = 0.016;
      }
      
      totalTime += delta;
      timeAdvanced = true;
    }
    lastFrameTime = now;

    // Avoid burning the GPU drawing identical frames if we're paused and not resizing/changing palette
    if (timeAdvanced || needsRedraw) {
      gl.viewport(0, 0, canvasEl.width, canvasEl.height);
      gl.useProgram(program);

      gl.uniform1f(gl.getUniformLocation(program, "iTime"), totalTime);
      gl.uniform1f(gl.getUniformLocation(program, "iRandom"), randomOffset);
      gl.uniform2f(gl.getUniformLocation(program, "iResolution"), canvasEl.width, canvasEl.height);
      
      const dpr = window.devicePixelRatio || 1;
      gl.uniform1f(gl.getUniformLocation(program, "iCoverSize"), coverSize * dpr);

      gl.uniform1iv(gl.getUniformLocation(program, "iColors"), intColors);
      gl.uniform1fv(gl.getUniformLocation(program, "iRatios"), floatRatios);
      gl.uniform1i(gl.getUniformLocation(program, "iCount"), activeColorCount);

      gl.drawArrays(gl.TRIANGLES, 0, 6);
      needsRedraw = false;
    }

    animationFrame = requestAnimationFrame(render);
  }

  function handleResize() {
    if (canvasEl) {
      const dpr = window.devicePixelRatio || 1;
      canvasEl.width = window.innerWidth * dpr;
      canvasEl.height = window.innerHeight * dpr;
      needsRedraw = true;
    }
  }

  function handleVisibilityChange() {
    isTabVisible = !document.hidden;
  }

  $effect(() => {
    if (colors || coverSize) {
      handleResize();
    }
  });

  // Re-synchronize lastFrameTime when visibility is restored so the time delta doesn't hitch.
  $effect(() => {
    if (visible && isTabVisible) {
      lastFrameTime = performance.now();
      needsRedraw = true;
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
