#version 460

in vec3 vp;
uniform vec3 in_color;
out vec3 color;


void main () {
    color = in_color;
    gl_Position = vec4 (vp, 1.0);
}
