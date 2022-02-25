#version 300 es

precision highp float;

uniform sampler2D tex_object;

in vec2 tc;

out vec4 color;

void main(void)
{
    color = texture(tex_object, tc * vec2(5.0, 2.0));
}
