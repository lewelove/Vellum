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

const float SPEED = 0.02;
const float GRAIN_AMOUNT = 0.03;

vec3 hexToRgb(int hex) {
    float r = float((hex >> 16) & 0xFF) / 255.0;
    float g = float((hex >> 8) & 0xFF) / 255.0;
    float b = float(hex & 0xFF) / 255.0;
    return vec3(r, g, b);
}

// 2D Rotation Matrix
mat2 rot(float a) {
    float s = sin(a);
    float c = cos(a);
    return mat2(c, -s, s, c);
}

void main() {
    vec2 uv = gl_FragCoord.xy / iResolution.xy;
    float aspect = iResolution.x / iResolution.y;
    vec2 p = (uv - 0.5) * 2.5; 
    p.x *= aspect;
    
    float t = (iTime + iRandom) * SPEED;

    float val = 0.0;
    float amp = 1.0;
    float freq = 1.5;
    
    // Accumulate rotating sine waves
    for(int i = 0; i < 6; i++) {
        // Slowly rotate the space over time, offset by the iteration count
        p *= rot(t * 0.3 + float(i) * 1.1);
        
        // Add a sine wave along the X axis, distorted slightly by the Y axis
        val += sin(p.x * freq + sin(p.y * freq * 0.5)) * amp;
        
        freq *= 1.4; // Increase frequency (closer waves)
        amp *= 0.6;  // Decrease amplitude (less impact)
    }

    // Since we summed sines, output is roughly [-2.5, 2.5]. 
    // Shift to [0, 1] range.
    val = clamp(val * 0.2 + 0.5, 0.0, 1.0);
    
    // Sine wave combinations naturally cluster around the center (0.5), 
    // so we apply the trig expansion to force them to the edges to respect your ratios.
    val = 0.5 - 0.5 * cos(3.14159265 * val);
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
        
        float currentSoftness = min(0.06, weight * 0.45); 
        
        float startMask = (i == 0) ? 1.0 : smoothstep(cumulative - currentSoftness, cumulative + currentSoftness, val);
        float endMask = (i == iCount - 1) ? 0.0 : smoothstep(nextCumulative - currentSoftness, nextCumulative + currentSoftness, val);
        float weightMask = startMask - endMask;
        
        finalColor += hexToRgb(iColors[i]) * max(0.0, weightMask);
        cumulative = nextCumulative;
    }
    
    float grain = (fract(sin(dot(uv, vec2(12.9898, 78.233))) * 43758.5453) - 0.5) * GRAIN_AMOUNT;
    fragColor = vec4(finalColor + grain, 1.0);
}
