#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec4 color;

layout(set = 0, binding = 0) uniform Camera {
    mat4 view_projection;
} camera;

layout(location = 0) out vec2 out_uv;
layout(location = 1) out vec4 out_color;

void main() {
    out_uv = uv;
    out_color = color;
    gl_Position = camera.view_projection * vec4(position, 0.0, 1.0);
}
