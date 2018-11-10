#version 330

in vec3 texcoords;
uniform samplerCube cube_texture;
out vec4 frag_color;


void main () {
	frag_color = texture (cube_texture, texcoords);
}
