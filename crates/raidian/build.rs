fn main() {
    println!("cargo:rerun-if-changed=proto");
    let mut config = prost_build::Config::new();
    config
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile_protos(
            &[
                "proto/auth.proto",
                "proto/repository.proto",
                "proto/issue.proto",
                "proto/merge_request.proto",
                "proto/pipeline.proto",
                "proto/member.proto",
                "proto/collaboration.proto",
                "proto/dashboard.proto",
            ],
            &["proto"],
        )
        .expect("failed to compile protobuf definitions");
}
