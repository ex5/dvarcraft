#version 330
in uint id;
in vec2 i_position;
in vec2 a_Translate;
in uint i_tex_id;
in float is_selected;
out vec2 v_tex_coords;
out float v_is_selected;
flat out uint v_tex_id;
uniform mat4 matrix;
void main() {
    gl_Position = matrix * vec4(i_position + a_Translate, 0.0, 1.0);
    if (gl_VertexID % 4 == 0) {
        v_tex_coords = vec2(0.0, 1.0);
    } else if (gl_VertexID % 4 == 1) {
        v_tex_coords = vec2(1.0, 1.0);
    } else if (gl_VertexID % 4 == 2) {
        v_tex_coords = vec2(0.0, 0.0);
    } else {
        v_tex_coords = vec2(1.0, 0.0);
    }
    v_tex_id = i_tex_id;
    v_is_selected = is_selected;
}
