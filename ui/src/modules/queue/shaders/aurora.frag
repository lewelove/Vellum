#version 300 es
precision highp float;
precision highp int;

uniform float iTime;
uniform float iRandom;
uniform vec2 iResolution;
uniform float iCoverSize;
uniform int iColors[16];
uniform int iCount;

out vec4 fragColor;

const float SPEED = 0.03;
const float N = 4.0;

vec3 hexToRgb(int hex) {
    float r = float((hex >> 16) & 0xFF) / 255.0;
    float g = float((hex >> 8) & 0xFF) / 255.0;
    float b = float(hex & 0xFF) / 255.0;
    return vec3(r, g, b);
}

void main() {
    vec2 uv = gl_FragCoord.xy / iResolution.xy;
    float t = (iTime + iRandom) * SPEED;

    // Create vertical ribbons with horizontal swaying
    float warp = sin(uv.y * 2.0 + t) * 0.5;
    float val = sin(uv.x * 3.0 + warp + t * 2.0) * 0.5 + 0.5;
    
    // Add a layer of secondary noise
    val = mix(val, fract(val * 2.0 + uv.y), 0.1);

    float totalWeight = 0.0;
    for(int i = 0; i < 16; i++) {
        if (i >= iCount) break;
        totalWeight += 1.0 / (float(i) + N);
    }

    vec3 finalColor = vec3(0.0);
    float softness = 0.15; 
    float cumulative = 0.0;

    for(int i = 0; i < 16; i++) {
        if (i >= iCount) break;
        float weight = (1.0 / (float(i) + N)) / totalWeight;
        float nextCumulative = cumulative + weight;
        float weightMask = smoothstep(cumulative - softness, cumulative + softness, val) - 
                           smoothstep(nextCumulative - softness, nextCumulative + softness, val);
        finalColor += hexToRgb(iColors[i]) * max(0.0, weightMask);
        cumulative = nextCumulative;
    }

    fragColor = vec4(finalColor, 1.0);
}
