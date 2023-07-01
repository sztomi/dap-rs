use std::{
  env,
  fs::File,
  io::Write,
  path::{Path, PathBuf},
};
use quote::{format_ident, quote};
use rustfmt_wrapper::rustfmt;

type DynResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn parse_file(path: &Path) -> DynResult<syn::File> {
  let content = std::fs::read_to_string(path)?;
  syn::parse_file(&content).map_err(|e| e.into())
}

fn main() -> DynResult<()> {
  let out_dir = env::var_os("OUT_DIR").unwrap();
  let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
  let dest_path = PathBuf::from(&out_dir).join("generated_tests.rs");
  let manifest_path = PathBuf::from(&manifest_dir);

  let mut f = File::create(dest_path)?;
  let dap_src = manifest_path.parent().unwrap().join("dap").join("src");

  let ast = parse_file(&dap_src.join("responses.rs"))?;

  for item in ast.items {
    if let syn::Item::Enum(e) = item {
      if e.ident != "ResponseBody" {
        continue;
      }
      for variant in e.variants {
        let test_name = format_ident!(
          "validate_{}_response",
          variant.ident.to_string().to_lowercase()
        );

        let mut init_part = quote! {};
        if let syn::Fields::Unnamed(fields) = variant.fields {
          for field in fields.unnamed {
            if let syn::Type::Path(ty) = field.ty {
              let fieldname = &ty.path.segments[0].ident;
              init_part = quote! { (#fieldname::default()) };
              break;
            }
          }
        }

        let ident = format_ident!("{}", variant.ident.to_string());
        let schema_ident = format!("{}Response", variant.ident);

        let test_fn = quote! {
          #[test]
          fn #test_name() {
            let schema = get_schema(#schema_ident);
            let compiled = JSONSchema::options()
              .with_document("/".to_string(), schema.clone())
              .compile(&schema)
              .unwrap();
            let resp = Response {
              request_seq: 1,
              success: true,
              message: None,
              body: Some(ResponseBody::#ident #init_part),
            };
            let instance = resp_to_value(&resp);
            let result = compiled.validate(&instance);
            if let Err(errors) = result {
              for error in errors {
                eprintln!("Validation error: {}", error);
                eprintln!("Instance path: {}", error.instance_path);
              }
            }
            assert!(compiled.is_valid(&instance));
          }
        };
        writeln!(f, "{}", rustfmt(test_fn).unwrap())?;
      }
    }
  }

  Ok(())
}
