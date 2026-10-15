#version 450

layout(location = 0) in vec2 in_position;
layout(location = 1) in vec2 in_uv;
layout(location = 2) in vec4 in_color;

layout(set = 0, binding = 0) uniform Camera {
    mat4 view_projection;
} camera;

layout(location = 0) out vec2 out_uv;
layout(location = 1) out vec4 out_color;

void main() {
    out_uv = in_uv;
    out_color = in_color;
    gl_Position = camera.view_projection * vec4(in_position, 0.0, 1.0);
}
