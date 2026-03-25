#version 300 es
precision highp float;

uniform vec2 iResolution;
uniform float iIntensity;

out vec4 fragColor;

// The exact hash function used in your other shaders
float hash(vec2 p) {
    return fract(sin(dot(p, vec2(12.9898, 78.233))) * 43758.5453);
}

void main() {
    // We use gl_FragCoord to ensure the grain is screen-space stable
    float noise = (hash(gl_FragCoord.xy) - 0.0) * iIntensity;
    
    // We output grain as a modification to a neutral alpha
    // Using an additive/subtractive approach in the blend
    fragColor = vec4(vec3(0.1 + noise), 1.0);
}
