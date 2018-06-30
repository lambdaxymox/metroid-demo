#version 410

in vec2 st;
uniform sampler2D tex;
uniform vec4 text_colour;
out vec4 frag_color;


void main () {
    frag_color =  text_colour * texture(tex, st);
}

