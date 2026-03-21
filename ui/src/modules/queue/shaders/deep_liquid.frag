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

const float SPEED = 0.02;
const float N = 4.0;

vec3 hexToRgb(int hex) {
    float r = float((hex >> 16) & 0xFF) / 255.0;
    float g = float((hex >> 8) & 0xFF) / 255.0;
    float b = float(hex & 0xFF) / 255.0;
    return vec3(r, g, b);
}

float hash(vec2 p) { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453123); }

float noise(vec2 p) {
    vec2 i = floor(p); vec2 f = fract(p);
    f = f*f*(3.0-2.0*f);
    return mix(mix(hash(i), hash(i+vec2(1,0)), f.x), mix(hash(i+vec2(0,1)), hash(i+vec2(1,1)), f.x), f.y);
}

float fbm(vec2 p) {
    float v = 0.0; float a = 0.5;
    for (int i = 0; i < 3; i++) { v += a * noise(p); p *= 2.0; a *= 0.5; }
    return v;
}

void main() {
    vec2 uv = gl_FragCoord.xy / iResolution.xy;
    float aspect = iResolution.x / iResolution.y;
    uv.x *= aspect;
    float t = (iTime + iRandom) * SPEED;

    // Domain Warping: Noise affecting Noise
    vec2 q = vec2(fbm(uv + t), fbm(uv + vec2(5.2, 1.3) + t));
    vec2 r = vec2(fbm(uv + 4.0*q + vec2(1.7, 9.2) + 0.15*t), fbm(uv + 4.0*q + vec2(8.3, 2.8) + 0.126*t));
    float val = fbm(uv + 4.0*r);

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
        float weightMask = smoothstep(cumulative - 0.1, cumulative + 0.1, val) - 
                           smoothstep(nextCumulative - 0.1, nextCumulative + 0.1, val);
        finalColor += hexToRgb(iColors[i]) * max(0.0, weightMask);
        cumulative = nextCumulative;
    }

    fragColor = vec4(finalColor, 1.0);
}
