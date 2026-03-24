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

const float SPEED = 0.04;
const float GRAIN_AMOUNT = 0.03;

vec3 hexToRgb(int hex) {
    float r = float((hex >> 16) & 0xFF) / 255.0;
    float g = float((hex >> 8) & 0xFF) / 255.0;
    float b = float(hex & 0xFF) / 255.0;
    return vec3(r, g, b);
}

vec2 hash2(vec2 p) {
    p = vec2(dot(p,vec2(127.1,311.7)), dot(p,vec2(269.5,183.3)));
    return fract(sin(p)*43758.5453123);
}

// Inigo Quilez's Smooth Voronoi
float smoothVoronoi( in vec2 x, float falloff ) {
    vec2 p = floor( x );
    vec2 f = fract( x );
    
    float res = 0.0;
    for( int j=-1; j<=1; j++ ) {
        for( int i=-1; i<=1; i++ ) {
            vec2 b = vec2( i, j );
            vec2 r = vec2( b ) - f + hash2( p + b );
            float d = dot( r, r );
            res += exp( -falloff * d );
        }
    }
    return -(1.0/falloff) * log(res);
}

void main() {
    vec2 uv = gl_FragCoord.xy / iResolution.xy;
    float aspect = iResolution.x / iResolution.y;
    vec2 p = (uv - 0.5) * 3.0; // Scale of the cells
    p.x *= aspect;
    
    float t = (iTime + iRandom) * SPEED;

    // We warp the coordinates just slightly to give the cells a fluid wobble
    vec2 wobble = vec2(sin(p.y * 1.5 + t), cos(p.x * 1.5 + t)) * 0.3;
    
    // Evaluate smooth voronoi. Subtracting time makes the cells "fall" inward/outward
    float val = smoothVoronoi(p + wobble - vec2(0.0, t*0.5), 12.0);

    // Smooth Voronoi output is roughly [0.2, 0.9]. Normalize to [0, 1]
    val = clamp((val - 0.2) / 0.7, 0.0, 1.0);
    
    // Flatten distribution for the ratio mapping
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
        
        float currentSoftness = min(0.05, weight * 0.45); 
        
        float startMask = (i == 0) ? 1.0 : smoothstep(cumulative - currentSoftness, cumulative + currentSoftness, val);
        float endMask = (i == iCount - 1) ? 0.0 : smoothstep(nextCumulative - currentSoftness, nextCumulative + currentSoftness, val);
        float weightMask = startMask - endMask;
        
        finalColor += hexToRgb(iColors[i]) * max(0.0, weightMask);
        cumulative = nextCumulative;
    }
    
    float grain = (fract(sin(dot(uv, vec2(12.9898, 78.233))) * 43758.5453) - 0.5) * GRAIN_AMOUNT;
    fragColor = vec4(finalColor + grain, 1.0);
}
