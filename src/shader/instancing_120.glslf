#version 330
uniform sampler2DArray tex;
in vec2 v_tex_coords;
in float v_is_selected;
flat in uint v_tex_id;
out vec4 f_color;

void main() {
    f_color = texture(tex, vec3(v_tex_coords, float(v_tex_id))) * vec4(1.0 + v_is_selected, 1.0, 1.0, 1.0);
}
