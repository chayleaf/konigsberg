#[cfg(not(feature = "rebuild-reexports"))]
fn main() {}

#[cfg(feature = "rebuild-reexports")]
fn main() {
    use quote::ToTokens;
    let bindings = reqwest::blocking::get({
        #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
        {
            "https://raw.githubusercontent.com/Noxime/steamworks-rs/master/steamworks-sys/src/linux_bindings.rs"
        }
        #[cfg(target_os = "windows")]
        {
            "https://raw.githubusercontent.com/Noxime/steamworks-rs/master/steamworks-sys/src/windows_bindings.rs"
        }
        #[cfg(target_os = "macos")]
        {
            "https://raw.githubusercontent.com/Noxime/steamworks-rs/master/steamworks-sys/src/macos_bindings.rs"
        }
    }).and_then(|x| x.text()).expect("failed to fetch bindings from github");
    let mut generated = String::new();
    let code: syn::File = syn::parse_str(&bindings).expect("failed to parse bindings");
    for item in code.items {
        #[allow(clippy::single_match)]
        match item {
            syn::Item::ForeignMod(item) => {
                assert_eq!(item.attrs, []);
                assert_eq!(item.unsafety, None);
                assert_eq!(
                    item.abi.name.as_ref().map(|x| format!("{}", x.token())),
                    Some(format!("{}", proc_macro2::Literal::string("C")))
                );
                for it in item.items {
                    match it {
                        syn::ForeignItem::Fn(it) => {
                            let ident = format!("{}", it.sig.ident);
                            if &ident == "SteamInternal_FindOrCreateUserInterface" {
                                continue;
                            }
                            if !ident.contains("Steam") && !ident.starts_with('C') {
                                continue;
                            }
                            let mut link_name = None;
                            assert!(matches!(it.vis, syn::Visibility::Public(_)));
                            for attr in it.attrs {
                                match attr.meta {
                                    syn::Meta::NameValue(meta) => {
                                        let name = format!("{}", meta.path.get_ident().unwrap());
                                        if name == "doc" {
                                            continue;
                                        }
                                        assert_eq!(name, "link_name");
                                        link_name = Some(match meta.value {
                                            syn::Expr::Lit(syn::ExprLit {
                                                lit: syn::Lit::Str(lit),
                                                ..
                                            }) => lit.token(),
                                            _ => panic!(),
                                        });
                                    }
                                    _ => panic!(),
                                }
                            }
                            assert_eq!(it.sig.constness, None);
                            assert_eq!(it.sig.asyncness, None);
                            assert_eq!(it.sig.unsafety, None);
                            assert_eq!(it.sig.abi, None);
                            assert_eq!(it.sig.variadic, None);
                            assert_eq!(it.sig.generics.params.len(), 0);
                            let return_type = match it.sig.output {
                                syn::ReturnType::Type(_, typ) => Some(typ),
                                syn::ReturnType::Default => None,
                            };
                            let inputs = it.sig.inputs;
                            generated.push_str("reexport!(");
                            if let Some(link_name) = link_name {
                                generated.push_str(&format!("{}", link_name));
                                generated.push_str(", ");
                            }
                            generated.push_str("fn ");
                            generated.push_str(&ident);
                            generated.push('(');
                            let mut first = true;
                            for inp in inputs {
                                if first {
                                    first = false;
                                } else {
                                    generated.push_str(", ");
                                }
                                generated.push_str(&match inp {
                                    syn::FnArg::Typed(arg) => {
                                        assert!(arg.attrs.is_empty());
                                        format!("{}", arg.into_token_stream())
                                    }
                                    _ => panic!(),
                                });
                            }
                            generated.push(')');
                            if let Some(ret) = return_type {
                                generated.push_str(&format!(" -> {}", ret.into_token_stream()));
                            }
                            generated.push_str(");\n");
                        }
                        syn::ForeignItem::Static(..) => {}
                        _ => panic!(),
                    }
                }
            }
            _ => {}
        }
    }

    let mut out_dir = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    out_dir.push("reexports.rs");
    std::fs::write(out_dir, generated).expect("failed to generate reexports");
}
