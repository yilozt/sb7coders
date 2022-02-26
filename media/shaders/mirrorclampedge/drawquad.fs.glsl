#version 300 es
precision mediump float;

uniform bool is_mirror;
uniform sampler2D tex;

layout (location = 0) out vec4 color;

in vec2 uv;

void main(void)
{
    if (is_mirror)
    {
        color = texture(tex, ((uv * 0.5) * vec2(1.5, 1.0) + 0.5));
    }
    else
    {
        color = texture(tex, uv * vec2(1.5, 1.0));
    }
}
