#version 330

in vec2 vp;
uniform mat4 view, proj;
out vec2 st;


void main() {
    st = 0.5 * (vp + 1.0);
    gl_Position = proj * view * vec4 (vp, 0.0, 1.0);
}
