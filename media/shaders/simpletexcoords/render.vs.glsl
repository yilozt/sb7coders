#version 300 es

precision highp float;

uniform mat4 mv_matrix;
uniform mat4 proj_matrix;

layout (location = 0) in vec4 position;
layout (location = 4) in vec2 tc_in;

out vec2 tc;

void main(void)
{
    vec4 pos_vs = mv_matrix * position;

    tc = tc_in;

    gl_Position = proj_matrix * pos_vs;
}
