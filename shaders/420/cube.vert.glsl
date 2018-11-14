#version 420 core

in vec3 vp;
uniform mat4 view, proj;
out vec3 texcoords;


void main() {
	texcoords = vp;
	gl_Position = proj * view * vec4 (vp, 1.0);
}
