use std::{
    env,
    fs::{self, DirEntry},
    path::{Path, PathBuf},
};

use quote::quote;
use wasm_bindgen_cli_support::Bindgen;

const PATH_TO_CLIENT: &str = "../client";
const CLIENT_BIN_ENV_VAR: &str = "CARGO_BIN_FILE_CLIENT_client";

pub fn main() {
    let client_assets = Path::new("../client/assets");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let client_wasm_path = std::env::var(CLIENT_BIN_ENV_VAR).unwrap();

    // Based on https://github.com/DioxusLabs/dioxus/blob/da4794d7937ae9d554a1be7e2d7b5dfe6645e80d/packages/cli/src/builder.rs
    let client_outdir: PathBuf = Path::new(&out_dir).join("client");
    let bindgen_outdir = client_outdir.join("assets").join("dioxus");

    let mut bindgen_builder = Bindgen::new();
    bindgen_builder
        .input_path(client_wasm_path)
        .web(true)
        .unwrap()
        .debug(true)
        .demangle(true)
        .keep_debug(true)
        .remove_name_section(false)
        .remove_producers_section(false)
        .out_name("client")
        .generate(&bindgen_outdir)
        .unwrap();

    // FIXME optimize client wasm? (minimize + optimize?)

    println!(
        "cargo:rerun-if-env-changed={}",
        client_assets.to_str().unwrap()
    );
    let client_files_values =
        uri_paths_and_absolute_paths_of_contents(&[&client_assets, &client_outdir])
            .iter()
            .map(|(uri_path, absolute_path)| {
                let mime_type = match absolute_path.extension().unwrap().to_str().unwrap() {
                    "html" => quote! {Some("text/html; charset=utf-8")},
                    "css" => quote! {Some("text/css; charset=utf-8")},
                    "js" => quote! {Some("application/javascript; charset=utf-8")},
                    "wasm" => quote! {None},
                    extension => unimplemented!("{}", extension),
                };

                let uri_path = uri_path.to_str().unwrap();
                let uri_path = match uri_path {
                    "/server_index.html" => "/",
                    uri_path => uri_path,
                };

                let absolute_path = absolute_path.to_str().unwrap();

                quote! {
                    (#uri_path, #mime_type, include_bytes!(#absolute_path).as_slice())
                }
            })
            .collect::<Vec<_>>();

    let client_const = quote! {
        static CLIENT_FILES: &'static [(&'static str, Option<&'static str>, &'static [u8])] = &[
            #(#client_files_values),*
        ];
    };

    fs::write(
        Path::new(&out_dir).join("client.rs"),
        format!("{}", client_const),
    )
    .unwrap();

    // relative_paths_of_contents(&bindgen_path)
    // panic!("{:?}", Path::new(&out_dir).join("client.rs"))
}

fn uri_paths_and_absolute_paths_of_contents(root_paths: &[&Path]) -> Vec<(PathBuf, PathBuf)> {
    fn recurse(parent_uri_path: &Path, entry: DirEntry, ret: &mut Vec<(PathBuf, PathBuf)>) {
        let uri_path = parent_uri_path.join(entry.file_name());
        if entry.metadata().unwrap().is_file() {
            ret.push((uri_path, entry.path().canonicalize().unwrap()))
        } else {
            for entry in fs::read_dir(entry.path()).unwrap() {
                recurse(&uri_path, entry.unwrap(), ret);
            }
        }
    }

    let mut ret = vec![];
    for path in root_paths {
        for entry in fs::read_dir(&path).unwrap() {
            recurse(Path::new("/"), entry.unwrap(), &mut ret);
        }
    }
    ret
}
