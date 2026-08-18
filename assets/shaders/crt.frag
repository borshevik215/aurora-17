#version 100

precision lowp float;

varying lowp vec2 uv;

uniform sampler2D screen_texture;
uniform float time;

void main() {
    vec2 p = uv * 2.0 - 1.0;

    float r2 = dot(p, p);

    p *= 1.0 + 0.055 * r2;

    vec2 warped =
        p * 0.5 + 0.5;

    vec3 color =
        texture2D(
            screen_texture,
            warped
        ).rgb;

    float scan =
        0.975
        + 0.025
        * sin(
            warped.y * 900.0
        );

    color *= scan;

    float shimmer =
        1.0
        + 0.009
        * sin(time * 5.7);

    color *= shimmer;

    color.r *= 0.88;
    color.g *= 1.02;
    color.b *= 0.88;

    float vignette =
        1.0
        - smoothstep(
            0.35,
            1.10,
            r2
        );

    color *=
        0.80
        + 0.20 * vignette;

    gl_FragColor =
        vec4(color, 1.0);
}
