fn main() {
    let path = "dev_app/renderer.octo";
    octo::process_file(&path).unwrap();
/*
    let path_vert = "src/vert.glsl";
    octo::process_glsl_debug(&path_vert, octo::Shader::Vertex);

    let path_frag = "src/frag.glsl";
    octo::process_glsl_debug(&path_frag, octo::Shader::Fragment);
    */
}
