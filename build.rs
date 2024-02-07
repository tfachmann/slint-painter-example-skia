use slint_build::CompilerConfiguration;

fn main() {
    let config = CompilerConfiguration::new();
    slint_build::compile_with_config(
        "ui/appwindow.slint",
        config.with_style("fluent-dark".to_string()),
    )
    .unwrap();
}

