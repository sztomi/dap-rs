use quote::{format_ident, quote};
use rustfmt_wrapper::rustfmt;
use std::{
  collections::{HashMap, HashSet},
  env,
  fs::File,
  io::Write,
  path::{Path, PathBuf},
};

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
  let resp_to_body_ty = HashMap::from([("Initialize".to_string(), "Capabilities")]);

  // These responses are skipped because the schema for Source (which they contain) is
  // self-referential and we can't load that the way we're doing it now (by manually dereferencing
  // the $ref's in the schema).
  // This is something that jsonschema should explicitly develop support for.
  let skipped_responses: HashSet<_> = ["LoadedSources", "Scopes"]
    .iter()
    .map(|s| s.to_string())
    .collect();

  for item in ast.items {
    if let syn::Item::Enum(e) = item {
      if e.ident != "ResponseBody" {
        continue;
      }
      for variant in e.variants {
        if skipped_responses.contains(&variant.ident.to_string()) {
          continue;
        }
        let test_name = format_ident!(
          "validate_{}_response",
          variant.ident.to_string().to_lowercase()
        );

        let mut init_part = quote! {};
        if let syn::Fields::Unnamed(fields) = variant.fields.clone() {
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

        let test_name = format_ident!(
          "validate_fake_{}_response",
          variant.ident.to_string().to_lowercase()
        );
        let resp_ty = if resp_to_body_ty.contains_key(&variant.ident.to_string()) {
          format_ident!(
            "{}",
            resp_to_body_ty.get(&variant.ident.to_string()).unwrap()
          )
        } else {
          format_ident!("{}Response", variant.ident)
        };

        let mut init_part = quote! {};
        let mut create_body = quote! {};
        if let syn::Fields::Unnamed(fields) = variant.fields {
          for field in fields.unnamed {
            if let syn::Type::Path(_) = field.ty {
              init_part = quote! { (body) };
              create_body = quote! {
                let rng = &mut StdRng::from_seed(RNG_SEED);
                let body: #resp_ty = Faker.fake_with_rng(rng);
              };
              break;
            }
          }
        }

        let test_fn = quote! {
          #[test]
          fn #test_name() {
            let schema = get_schema(#schema_ident);
            let compiled = JSONSchema::options()
              .with_document("/".to_string(), schema.clone())
              .compile(&schema)
              .unwrap();
            #create_body
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
