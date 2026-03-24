#version 300 es
precision highp float;
precision highp int;

uniform float iTime;
uniform float iRandom;
uniform vec2 iResolution;
uniform float iCoverSize;
uniform int iColors[16];
uniform float iRatios[16];
uniform int iCount;

out vec4 fragColor;

// Slowed down to match the new, larger scale
const float SPEED = 0.005;
const float GRAIN_AMOUNT = 0.03;

vec3 hexToRgb(int hex) {
    float r = float((hex >> 16) & 0xFF) / 255.0;
    float g = float((hex >> 8) & 0xFF) / 255.0;
    float b = float(hex & 0xFF) / 255.0;
    return vec3(r, g, b);
}

vec2 hash( vec2 p ) {
    p = vec2(dot(p,vec2(127.1,311.7)), dot(p,vec2(269.5,183.3)));
    return -1.0 + 2.0*fract(sin(p)*43758.5453123);
}

float noise( in vec2 p ) {
    const float K1 = 0.366025404; 
    const float K2 = 0.211324865; 
    vec2 i = floor(p + (p.x+p.y)*K1);
    vec2 a = p - i + (i.x+i.y)*K2;
    vec2 o = (a.x>a.y) ? vec2(1.0,0.0) : vec2(0.0,1.0);
    vec2 b = a - o + K2;
    vec2 c = a - 1.0 + 2.0*K2;
    vec3 h = max(0.5-vec3(dot(a,a), dot(b,b), dot(c,c) ), 0.0);
    vec3 n = h*h*h*h*vec3( dot(a,hash(i+0.0)), dot(b,hash(i+o)), dot(c,hash(i+1.0)));
    return dot(n, vec3(70.0));
}

float fbm(vec2 p) {
    float f = 0.0;
    float amp = 0.5;
    for(int i=0; i<4; i++) {
        f += amp * noise(p);
        p *= 2.0;
        amp *= 0.5;
    }
    return f;
}

void main() {
    vec2 uv = gl_FragCoord.xy / iResolution.xy;
    float aspect = iResolution.x / iResolution.y;
    
    // Zoomed in coordinate space for macroscopic, less dense waves
    vec2 p = uv * 1.2; 
    p.x *= aspect;
    
    float t = (iTime + iRandom) * SPEED;

    // Relaxed Domain Warping (Multipliers reduced from 3.0 to 1.2/1.5)
    vec2 q = vec2(fbm(p + vec2(0.0,0.0) + t), 
                  fbm(p + vec2(5.2,1.3) - t * 0.6));
    
    vec2 r = vec2(fbm(p + 1.2 * q + vec2(1.7,9.2) + t * 0.9), 
                  fbm(p + 1.2 * q + vec2(8.3,2.8) - t * 0.8));
    
    float val = fbm(p + 1.5 * r);

    // FBM output is ~[-0.95, 0.95]. Shift to [0, 1] without hard clamping.
    val = val * 0.5 + 0.5;
    val = clamp(val, 0.0, 1.0);
    
    // Single-pass trig expansion.
    // Preserves your ratio mapping while preventing steep/sharp gradient cliffs.
    val = 0.5 - 0.5 * cos(3.14159265 * val);

    float totalWeight = 0.0;
    for(int i = 0; i < 16; i++) {
        if (i >= iCount) break;
        totalWeight += iRatios[i];
    }
    if (totalWeight <= 0.0) totalWeight = 1.0; 

    vec3 finalColor = vec3(0.0);
    float cumulative = 0.00;

    for(int i = 0; i < 16; i++) {
        if (i >= iCount) break;
        
        float weight = iRatios[i] / totalWeight;
        float nextCumulative = cumulative + weight;
        
        // Massive increase in maximum softness (0.08 -> 0.35) for blurry transitions
        float currentSoftness = min(0.8, weight * 0.8); 
        
        float startMask = (i == 0) ? 1.0 : smoothstep(cumulative - currentSoftness, cumulative + currentSoftness, val);
        float endMask = (i == iCount - 1) ? 0.0 : smoothstep(nextCumulative - currentSoftness, nextCumulative + currentSoftness, val);
        float weightMask = startMask - endMask;
        
        finalColor += hexToRgb(iColors[i]) * max(0.0, weightMask);
        cumulative = nextCumulative;
    }
    
    float grain = (fract(sin(dot(uv, vec2(12.9898, 78.233))) * 43758.5453) - 0.5) * GRAIN_AMOUNT;
    fragColor = vec4(finalColor + grain, 1.0);
}
