#version 460

in layout(location = 0) vec3 vertex_position;
in layout(location = 1) vec3 vertex_color;

uniform mat4 model, view, proj;
out vec3 color;


void main () {
    color = vertex_color;
    gl_Position = proj * view * model * vec4 (vertex_position, 1.0);
}
