# A list of available rules and their signatures can be found here: https://buck2.build/docs/api/rules/

genrule(
    name = "hello_world",
    out = "out.txt",
    cmd = "echo BUILT BY BUCK2> $OUT",
)

rust_binary(
    name = "hello_rust",
    srcs = ["hello.rs"],
    crate_root = "hello.rs",
    edition = "2021",
    deps = [
        "kernel_uapi//:kernel_uapi"
    ]
)
