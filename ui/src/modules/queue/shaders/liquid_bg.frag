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

const float SPEED = 0.03; // Adjusted for rotational feel
const float GRAIN_AMOUNT = 0.02;
const float N = 4.0; 

vec3 hexToRgb(int hex) {
    float r = float((hex >> 16) & 0xFF) / 255.0;
    float g = float((hex >> 8) & 0xFF) / 255.0;
    float b = float(hex & 0xFF) / 255.0;
    return vec3(r, g, b);
}

float hash(vec2 p) {
    p = fract(p * vec2(123.34, 456.21));
    p += dot(p, p + 45.32);
    return fract(p.x * p.y);
}

float noise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    f = f * f * (3.0 - 2.0 * f);
    float a = hash(i);
    float b = hash(i + vec2(1.0, 0.0));
    float c = hash(i + vec2(0.0, 1.0));
    float d = hash(i + vec2(1.0, 1.0));
    return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

// Rotation matrix to "spin" the noise layers
mat2 rot(float a) {
    float s = sin(a);
    float c = cos(a);
    return mat2(c, -s, s, c);
}

float fbm(vec2 p) {
    float v = 0.0;
    float a = 0.5;
    float t = (iTime + iRandom) * SPEED;
    for (int i = 0; i < 4; i++) {
        // Rotate each octave in a different direction to prevent drifting
        p *= rot(t * 0.2 * float(i + 1));
        v += a * noise(p);
        p *= 2.0;
        a *= 0.5;
    }
    return v;
}

void main() {
    vec2 uv = gl_FragCoord.xy / iResolution.xy;
    float aspect = iResolution.x / iResolution.y;
    vec2 p = uv - 0.5; // Center for rotation
    p.x *= aspect;
    
    float t = (iTime + iRandom) * SPEED;
    
    // Create a "pulse" rather than a drift
    float zoom = 1.0 + sin(t * 0.5) * 0.1;
    p *= zoom;

    // Use Domain Warping to create fluid motion that stays roughly in place
    vec2 movement = vec2(sin(t * 0.7), cos(t * 0.8)) * 0.2;
    float val = fbm(p * 1.5 + movement);
    
    // Second pass of noise to break up the symmetry
    val = (val + fbm(p * 1.2 - movement)) * 0.5;

    float totalWeight = 0.0;
    for(int i = 0; i < 16; i++) {
        if (i >= iCount) break;
        totalWeight += 1.0 / (float(i) + N);
    }

    vec3 finalColor = vec3(0.0);
    float softness = 0.02; 
    float cumulative = 0.1;

    for(int i = 0; i < 16; i++) {
        if (i >= iCount) break;
        
        float weight = (1.0 / (float(i) + N)) / totalWeight;
        float nextCumulative = cumulative + weight;
        
        float weightMask = smoothstep(cumulative - softness, cumulative + softness, val) - 
                           smoothstep(nextCumulative - softness, nextCumulative + softness, val);
        
        finalColor += hexToRgb(iColors[i]) * max(0.0, weightMask);
        cumulative = nextCumulative;
    }

    float noiseFloor = (hash(uv + iTime) - 0.5) * GRAIN_AMOUNT;
    finalColor += noiseFloor;
    
    fragColor = vec4(finalColor, 1.0);
}
