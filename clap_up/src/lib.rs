use cargo_up::{
    ra_hir::{Adt, AsAssocItem, AssocItemContainer, Function, Module, Name, Semantics},
    ra_ide_db::RootDatabase,
    ra_syntax::ast::{self, AstNode},
    upgrader, Upgrader, Visitor,
};

#[upgrader(minimum = "2.33.0", peers = ["structopt"])]
#[derive(Default)]
pub struct Clap;

// #[up("3.0.0-beta.1")]
// replace_dep!("structopt", "clap", features = ["derive"]);

// #[up("3.0.0-beta.1")]
// rename_struct_methods!(clap::args::arg, Arg, [
//     help => about,
//     from_usage => from,
//     set => setting,
//     unset => unset_setting,
// ]);

// #[up("3.0.0-beta.1")]
// rename_struct_methods!(clap::app, App, [
//     from_yaml => from,
//     arg_from_usage => arg,
//     help => override_help,
//     usage => override_usage,
//     template => help_template,
//     get_matches_safe => try_get_matches,
//     get_matches_from_safe => try_get_matches_from,
//     get_matches_from_safe_borrow => try_get_matches_from_mut,
// ]);

// #[up("3.0.0-rc.0")]
// rename_struct_methods!(clap::build::arg, Arg, with_name => new);

impl Visitor for Clap {
    fn visit_method_call_expr(
        &mut self,
        method_call_expr: ast::MethodCallExpr,
        semantics: &Semantics<RootDatabase>,
    ) {
        if let Some(name_ref) = method_call_expr.name_ref() {
            if name_ref.text() != "help" {
                return;
            }

            let f = semantics.resolve_method_call(&method_call_expr).unwrap();

            if let Some(name) = get_struct_name(&f, semantics.db) {
                let mod_name = full_name(&f.module(semantics.db), semantics.db);

                if format!("{}", name) == "Arg" && mod_name == "clap::args::arg" {
                    self.replace(
                        method_call_expr.name_ref().unwrap().syntax().text_range(),
                        "about".to_string(),
                    );
                }
            }
        }
    }
}

fn get_struct_name(f: &Function, db: &RootDatabase) -> Option<Name> {
    let assoc_item = f.clone().as_assoc_item(db)?;

    if let AssocItemContainer::ImplDef(impl_def) = assoc_item.container(db) {
        if let Adt::Struct(s) = impl_def.target_ty(db).as_adt()? {
            return Some(s.name(db));
        }
    }

    None
}

fn full_name(m: &Module, db: &RootDatabase) -> String {
    let mut ret = vec![];
    let mut module = *m;

    loop {
        if let Some(name) = module.name(db) {
            ret.push(format!("{}", name));

            if let Some(p) = module.parent(db) {
                module = p;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    if let Some(name) = m.krate().display_name(db) {
        ret.push(format!("{}", name));
    }

    ret.reverse();
    ret.join("::")
}
