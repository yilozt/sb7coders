#version 460 core

layout (location = 0) in vec4 position;

uniform mat4 proj_mat;
uniform mat4 mv_mat;

out vec4 vs_color;

void main() {
    gl_Position = proj_mat * mv_mat * position;
    vs_color = position * 2.0 + vec4(0.5, 0.5, 0.5, 0.0);
}