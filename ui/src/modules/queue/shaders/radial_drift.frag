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

const float SPEED = 0.05;
const float N = 4.0;

vec3 hexToRgb(int hex) {
    float r = float((hex >> 16) & 0xFF) / 255.0;
    float g = float((hex >> 8) & 0xFF) / 255.0;
    float b = float(hex & 0xFF) / 255.0;
    return vec3(r, g, b);
}

float hash(vec2 p) { return fract(sin(dot(p, vec2(12.9898, 78.233))) * 43758.5453); }

float noise(vec2 p) {
    vec2 i = floor(p); vec2 f = fract(p);
    f = f*f*(3.0-2.0*f);
    return mix(mix(hash(i), hash(i+vec2(1,0)), f.x), mix(hash(i+vec2(0,1)), hash(i+vec2(1,1)), f.x), f.y);
}

void main() {
    vec2 uv = gl_FragCoord.xy / iResolution.xy;
    vec2 center = vec2(0.5);
    vec2 dir = uv - center;
    float dist = length(dir);
    float angle = atan(dir.y, dir.x);

    float t = (iTime + iRandom) * SPEED;

    // Use polar coordinates for noise
    float val = noise(vec2(dist * 2.0 - t, angle * 1.5)) * 0.5 +
                noise(vec2(dist * 1.0 - t * 0.5, angle * 3.0)) * 0.5;

    float totalWeight = 0.0;
    for(int i = 0; i < 16; i++) {
        if (i >= iCount) break;
        totalWeight += 1.0 / (float(i) + N);
    }

    vec3 finalColor = vec3(0.0);
    float cumulative = 0.0;
    for(int i = 0; i < 16; i++) {
        if (i >= iCount) break;
        float weight = (1.0 / (float(i) + N)) / totalWeight;
        float nextCumulative = cumulative + weight;
        float weightMask = smoothstep(cumulative - 0.12, cumulative + 0.12, val) - 
                           smoothstep(nextCumulative - 0.12, nextCumulative + 0.12, val);
        finalColor += hexToRgb(iColors[i]) * max(0.0, weightMask);
        cumulative = nextCumulative;
    }

    fragColor = vec4(finalColor, 1.0);
}
