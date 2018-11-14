#version 420 core

in layout (location = 0) vec2 vp;
in layout (location = 1) vec2 vt;
out vec2 st;


void main() {
    st = vt;
    gl_Position = vec4(vp, 0.0, 1.0);
}
