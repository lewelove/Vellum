<script>
  import { onMount, onDestroy } from "svelte";
  import vertexShaderSource from "./shaders/quad.vert?raw";
  import fragmentShaderSource from "./shaders/grain.frag?raw";

  let { intensity = 0.00 } = $props();

  let canvasEl;
  let gl;
  let program;

  function initGL() {
    gl = canvasEl.getContext("webgl2", { alpha: false });
    if (!gl) return;

    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, vertexShaderSource);
    gl.compileShader(vs);

    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, fragmentShaderSource);
    gl.compileShader(fs);

    program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);

    const vertices = new Float32Array([-1,-1, 1,-1, -1,1, -1,1, 1,-1, 1,1]);
    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(gl.ARRAY_BUFFER, vertices, gl.STATIC_DRAW);

    const posLoc = gl.getAttribLocation(program, "position");
    gl.enableVertexAttribArray(posLoc);
    gl.vertexAttribPointer(posLoc, 2, gl.FLOAT, false, 0, 0);

    render();
  }

  function render() {
    if (!gl) return;
    gl.viewport(0, 0, canvasEl.width, canvasEl.height);
    gl.useProgram(program);
    gl.uniform2f(gl.getUniformLocation(program, "iResolution"), canvasEl.width, canvasEl.height);
    gl.uniform1f(gl.getUniformLocation(program, "iIntensity"), intensity);
    gl.drawArrays(gl.TRIANGLES, 0, 6);
  }

  function handleResize() {
    if (canvasEl) {
      const dpr = window.devicePixelRatio || 1;
      canvasEl.width = canvasEl.clientWidth * dpr;
      canvasEl.height = canvasEl.clientHeight * dpr;
      render();
    }
  }

  onMount(() => {
    initGL();
    window.addEventListener("resize", handleResize);
    handleResize();
  });

  onDestroy(() => {
    window.removeEventListener("resize", handleResize);
  });
</script>

<canvas bind:this={canvasEl} class="grain-overlay"></canvas>

<style>
  .grain-overlay {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
    z-index: 1;
    /* mix-blend-mode: overlay; */
    opacity: 0.0;
    image-rendering: pixelated;
  }
</style>
