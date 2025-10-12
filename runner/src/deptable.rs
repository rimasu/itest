use std::collections::BTreeMap;
use std::fmt::{self, Debug, Display};

use crate::tasklist::TaskList;

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub enum Error {
    // Error when one or more declarations reference a dependency that is not declared
    UndeclaredDependency {
        dep_name: String,
        decls: Vec<(String, String)>,
    },

    // Error when two or more declarations have the same name
    DeclNameConflict {
        name: String,
        decls: Vec<String>,
    },
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::UndeclaredDependency { dep_name, decls } => {
                writeln!(
                    f,
                    "Undeclared component used in #[depends_on({})]",
                    dep_name
                )?;
                for (name, decl) in decls {
                    writeln!(f, "\tused by #[set_up({})] at {}", name, decl)?;
                }
                Ok(())
            }
            Error::DeclNameConflict { name, decls } => {
                writeln!(f, "Multiple components have same name #[set_up({})]", name)?;
                for decl in decls {
                    writeln!(f, "\tused at {}", decl)?;
                }
                Ok(())
            }
        }
    }
}

struct Decl<D> {
    pub name: String,
    decl: D,
}

#[derive(Default)]
struct UnresolvedUsage {
    is_declared_by: Vec<usize>,
    is_depended_on_by: Vec<usize>,
}

struct ResolvedUsage {
    decl: usize,
    is_dependend_on_by: Vec<usize>,
}

pub struct Builder<D> {
    decls: Vec<Decl<D>>,
    usages: BTreeMap<String, UnresolvedUsage>, // Use btree map to enforce deterministic iteration
}

impl<D: Display> Builder<D> {
    pub fn new() -> Self {
        Self {
            decls: Vec::new(),
            usages: BTreeMap::new(),
        }
    }
    pub fn declare_node(&mut self, decl: D, name: &str, deps: &[&str]) {
        let decl_idx = self.decls.len();
        self.decls.push(Decl {
            name: name.to_owned(),
            decl,
        });

        self.usage(name).is_declared_by.push(decl_idx);
        for d in deps {
            self.usage(d).is_depended_on_by.push(decl_idx);
        }
    }

    fn usage(&mut self, name: &str) -> &mut UnresolvedUsage {
        self.usages.entry(name.to_owned()).or_default()
    }

    pub fn build(self) -> Result<DepTable<D>, Vec<Error>> {
        let usages = resolve_usages(&self.decls, self.usages)?;
        // if every usage resolved - we should have exactly one usage per decl
        assert_eq!(usages.len(), self.decls.len());
        Ok(DepTable {
            decls: self.decls,
            usages,
        })
    }
}

fn resolve_usages<D: Display>(
    decls: &[Decl<D>],
    usages: BTreeMap<String, UnresolvedUsage>,
) -> Result<Vec<ResolvedUsage>, Vec<Error>> {
    let mut errs = Vec::new();
    let mut resolved = Vec::new();
    for (name, usage) in usages {
        match resolve_usage(decls, name, usage) {
            Ok(r) => resolved.push(r),
            Err(e) => errs.push(e),
        }
    }
    if errs.is_empty() {
        Ok(resolved)
    } else {
        Err(errs)
    }
}

fn resolve_usage<D: Display>(
    decls: &[Decl<D>],
    name: String,
    usage: UnresolvedUsage,
) -> Result<ResolvedUsage, Error> {
    if usage.is_declared_by.len() > 1 {
        Err(Error::DeclNameConflict {
            name,
            decls: usage
                .is_declared_by
                .into_iter()
                .map(|i| decls[i].decl.to_string())
                .collect(),
        })
    } else if usage.is_declared_by.is_empty() {
        Err(Error::UndeclaredDependency {
            dep_name: name,
            decls: usage
                .is_depended_on_by
                .into_iter()
                .map(|i| (decls[i].name.to_owned(), decls[i].decl.to_string()))
                .collect(),
        })
    } else {
        Ok(ResolvedUsage {
            decl: usage.is_declared_by[0], // we know there will be exactly one,
            is_dependend_on_by: usage.is_depended_on_by,
        })
    }
}

pub struct DepTable<D> {
    decls: Vec<Decl<D>>,
    usages: Vec<ResolvedUsage>,
}

impl<D: Display> DepTable<D> {
    pub fn name(&self, id: usize) -> &str {
        &self.decls[id].name
    }

    pub fn decl(&self, id: usize) -> &D {
        &self.decls[id].decl
    }

    pub fn make_task_list(&self) -> TaskList {
        let mut deps = Vec::with_capacity(self.usages.len());
        for _ in &self.usages {
            deps.push(vec![])
        }
        for usage in &self.usages {
            for unblocked in &usage.is_dependend_on_by {
                deps[*unblocked].push(usage.decl);
            }
        }
        TaskList::new(&deps)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn detect_undeclared_deps_and_name_conflicts() {
        let mut bld = Builder::new();
        bld.declare_node("1".to_owned(), "a", &["c"]);
        bld.declare_node("2".to_owned(), "a", &["c"]);
        bld.declare_node("3".to_owned(), "b", &["d"]);
        bld.declare_node("4".to_owned(), "e", &["d"]);

        let errs = bld.build().err().unwrap();
        assert_eq!(
            vec![
                Error::DeclNameConflict {
                    name: "a".to_owned(),
                    decls: vec!["1".to_owned(), "2".to_owned()],
                },
                Error::UndeclaredDependency {
                    dep_name: "c".to_owned(),
                    decls: vec![
                        ("a".to_owned(), "1".to_owned()),
                        ("a".to_owned(), "2".to_owned()),
                    ]
                },
                Error::UndeclaredDependency {
                    dep_name: "d".to_owned(),
                    decls: vec![
                        ("b".to_owned(), "3".to_owned()),
                        ("e".to_owned(), "4".to_owned())
                    ]
                }
            ],
            errs
        );
    }

    #[test]
    fn can_convert_error_to_string() {
        let errs = vec![
            Error::DeclNameConflict {
                name: "a".to_owned(),
                decls: vec!["1".to_owned(), "2".to_owned()],
            },
            Error::UndeclaredDependency {
                dep_name: "c".to_owned(),
                decls: vec![
                    ("a".to_owned(), "1".to_owned()),
                    ("a".to_owned(), "2".to_owned()),
                ],
            },
            Error::UndeclaredDependency {
                dep_name: "d".to_owned(),
                decls: vec![
                    ("b".to_owned(), "3".to_owned()),
                    ("e".to_owned(), "4".to_owned()),
                ],
            },
        ];

        let str = errs
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join("\n");

        assert_eq!(
            "Multiple components have same name #[set_up(a)]\n\tused at 1\n\tused at 2\n\nUndeclared component used in #[depends_on(c)]\n\tused by #[set_up(a)] at 1\n\tused by #[set_up(a)] at 2\n\nUndeclared component used in #[depends_on(d)]\n\tused by #[set_up(b)] at 3\n\tused by #[set_up(e)] at 4\n",
            str
        )
    }
}
