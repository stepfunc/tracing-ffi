use oo_bindgen::model::*;
use std::env;
use std::io::Write;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let mut file = std::fs::File::create(
        Path::new(&env::var_os("OUT_DIR").unwrap()).join("implementation.rs"),
    )
    .unwrap();

    file.write_all(sfio_tracing_ffi::get_impl_file().as_bytes())
        .unwrap();

    let settings = LibrarySettings::create(
        "tracing_ffi",
        "tracing_ffi",
        ClassSettings::default(),
        IteratorSettings::default(),
        CollectionSettings::default(),
        FutureSettings::default(),
        InterfaceSettings::default(),
    )
    .unwrap();

    let info = LibraryInfo {
        description: "just a test".to_string(),
        project_url: "".to_string(),
        repository: "".to_string(),
        license_name: "".to_string(),
        license_description: vec![],
        license_path: Default::default(),
        developers: vec![],
        logo_png: &[],
    };

    let mut builder = oo_bindgen::model::LibraryBuilder::new(Version::new(0, 1, 0), info, settings);

    let error_type = builder
        .define_error_type(
            "init_error",
            "init_exception",
            ExceptionType::UncheckedException,
        )
        .unwrap()
        .add_error(
            "tracing_init_failed",
            "Unable to initialize tracing backend",
        )
        .unwrap()
        .doc("error type")
        .unwrap()
        .build()
        .unwrap();

    sfio_tracing_ffi::define(&mut builder, error_type).unwrap();

    let lib = builder.build().unwrap();

    oo_bindgen::backend::rust::generate_ffi(&lib).unwrap();
}
