use super::*;
use crate::cache::resolvers::{DummyResolver, SimpleResolver};
use crate::error::ImportError;
use crate::label::Label;
use crate::parser::{grammar, lexer};
use crate::term::make as mk_term;
use crate::term::{BinaryOp, StrChunk, UnaryOp};
use crate::transform::import_resolution::resolve_imports;
use crate::{mk_app, mk_fun};
use codespan::Files;

/// Evaluate a term without import support.
fn eval_no_import(t: RichTerm) -> Result<Term, EvalError> {
    eval(t, &Environment::new(), &mut DummyResolver {}).map(Term::from)
}

fn parse(s: &str) -> Option<RichTerm> {
    let id = Files::new().add("<test>", String::from(s));

    grammar::TermParser::new()
        .parse_term(id, lexer::Lexer::new(&s))
        .map(RichTerm::without_pos)
        .map_err(|err| println!("{:?}", err))
        .ok()
}

#[test]
fn identity_over_values() {
    let num = Term::Num(45.3);
    assert_eq!(Ok(num.clone()), eval_no_import(num.into()));

    let boolean = Term::Bool(true);
    assert_eq!(Ok(boolean.clone()), eval_no_import(boolean.into()));

    let lambda = mk_fun!("x", mk_app!(mk_term::var("x"), mk_term::var("x")));
    assert_eq!(Ok(lambda.as_ref().clone()), eval_no_import(lambda.into()));
}

#[test]
fn blame_panics() {
    let label = Label::dummy();
    if let Err(EvalError::BlameError(l, ..)) =
        eval_no_import(mk_term::op1(UnaryOp::Blame(), Term::Lbl(label.clone())))
    {
        assert_eq!(l, label);
    } else {
        panic!("This evaluation should've returned a BlameError!");
    }
}

#[test]
#[should_panic]
fn lone_var_panics() {
    eval_no_import(mk_term::var("unbound")).unwrap();
}

#[test]
fn only_fun_are_applicable() {
    eval_no_import(mk_app!(Term::Bool(true), Term::Num(45.))).unwrap_err();
}

#[test]
fn simple_app() {
    let t = mk_app!(mk_term::id(), Term::Num(5.0));
    assert_eq!(Ok(Term::Num(5.0)), eval_no_import(t));
}

#[test]
fn simple_let() {
    let t = mk_term::let_in("x", Term::Num(5.0), mk_term::var("x"));
    assert_eq!(Ok(Term::Num(5.0)), eval_no_import(t));
}

#[test]
fn simple_ite() {
    let t = mk_term::if_then_else(Term::Bool(true), Term::Num(5.0), Term::Bool(false));
    assert_eq!(Ok(Term::Num(5.0)), eval_no_import(t));
}

#[test]
fn simple_plus() {
    let t = mk_term::op2(BinaryOp::Plus(), Term::Num(5.0), Term::Num(7.5));
    assert_eq!(Ok(Term::Num(12.5)), eval_no_import(t));
}

#[test]
fn asking_for_various_types() {
    let num = mk_term::op1(UnaryOp::IsNum(), Term::Num(45.3));
    assert_eq!(Ok(Term::Bool(true)), eval_no_import(num));

    let boolean = mk_term::op1(UnaryOp::IsBool(), Term::Bool(true));
    assert_eq!(Ok(Term::Bool(true)), eval_no_import(boolean));

    let lambda = mk_term::op1(
        UnaryOp::IsFun(),
        mk_fun!("x", mk_app!(mk_term::var("x"), mk_term::var("x"))),
    );
    assert_eq!(Ok(Term::Bool(true)), eval_no_import(lambda));
}

fn mk_default(t: RichTerm) -> Term {
    use crate::term::MergePriority;

    let mut meta = MetaValue::from(t);
    meta.priority = MergePriority::Default;
    Term::MetaValue(meta)
}

fn mk_docstring<S>(t: RichTerm, s: S) -> Term
where
    S: Into<String>,
{
    let mut meta = MetaValue::from(t);
    meta.doc.replace(s.into());
    Term::MetaValue(meta)
}

#[test]
fn enriched_terms_unwrapping() {
    let t =
        mk_default(mk_default(mk_docstring(Term::Bool(false).into(), "a").into()).into()).into();
    assert_eq!(Ok(Term::Bool(false)), eval_no_import(t));
}

#[test]
fn merge_enriched_default() {
    let t = mk_term::op2(
        BinaryOp::Merge(),
        Term::Num(1.0),
        mk_default(Term::Num(2.0).into()),
    );
    assert_eq!(Ok(Term::Num(1.0)), eval_no_import(t));
}

#[test]
fn merge_incompatible_defaults() {
    let t = mk_term::op2(
        BinaryOp::Merge(),
        mk_default(Term::Num(1.0).into()),
        mk_default(Term::Num(2.0).into()),
    );

    eval_no_import(t).unwrap_err();
}

#[test]
fn imports() {
    let mut resolver = SimpleResolver::new();
    resolver.add_source(String::from("two"), String::from("1 + 1"));
    resolver.add_source(String::from("lib"), String::from("{f = true}"));
    resolver.add_source(String::from("bad"), String::from("^$*/.23ab 0°@"));
    resolver.add_source(
        String::from("nested"),
        String::from("let x = import \"two\" in x + 1"),
    );
    resolver.add_source(
        String::from("cycle"),
        String::from("let x = import \"cycle_b\" in {a = 1, b = x.a}"),
    );
    resolver.add_source(
        String::from("cycle_b"),
        String::from("let x = import \"cycle\" in {a = x.a}"),
    );

    fn mk_import<R>(
        var: &str,
        import: &str,
        body: RichTerm,
        resolver: &mut R,
    ) -> Result<RichTerm, ImportError>
    where
        R: ImportResolver,
    {
        resolve_imports(
            mk_term::let_in(var, mk_term::import(import), body),
            resolver,
        )
        .map(|(t, _)| t)
    }

    // let x = import "does_not_exist" in x
    match mk_import("x", "does_not_exist", mk_term::var("x"), &mut resolver).unwrap_err() {
        ImportError::IOError(_, _, _) => (),
        _ => assert!(false),
    };

    // let x = import "bad" in x
    match mk_import("x", "bad", mk_term::var("x"), &mut resolver).unwrap_err() {
        ImportError::ParseErrors(_, _) => (),
        _ => assert!(false),
    };

    // let x = import "two" in x
    assert_eq!(
        eval(
            mk_import("x", "two", mk_term::var("x"), &mut resolver).unwrap(),
            &Environment::new(),
            &mut resolver
        )
        .map(Term::from)
        .unwrap(),
        Term::Num(2.0)
    );

    // let x = import "lib" in x.f
    assert_eq!(
        eval(
            mk_import(
                "x",
                "lib",
                mk_term::op1(UnaryOp::StaticAccess(Ident::from("f")), mk_term::var("x")),
                &mut resolver,
            )
            .unwrap(),
            &Environment::new(),
            &mut resolver
        )
        .map(Term::from)
        .unwrap(),
        Term::Bool(true)
    );
}

#[test]
fn interpolation_simple() {
    let mut chunks = vec![
        StrChunk::Literal(String::from("Hello")),
        StrChunk::expr(
            mk_term::op2(
                BinaryOp::StrConcat(),
                mk_term::string(", "),
                mk_term::string("World!"),
            )
            .into(),
        ),
        StrChunk::Literal(String::from(" How")),
        StrChunk::expr(mk_term::if_then_else(
            Term::Bool(true),
            mk_term::string(" are"),
            mk_term::string(" is"),
        )),
        StrChunk::Literal(String::from(" you?")),
    ];
    chunks.reverse();

    let t: RichTerm = Term::StrChunks(chunks).into();
    assert_eq!(
        eval_no_import(t),
        Ok(Term::Str(String::from("Hello, World! How are you?")))
    );
}

#[test]
fn interpolation_nested() {
    let mut inner_chunks = vec![
        StrChunk::Literal(String::from(" How")),
        StrChunk::expr(
            Term::Op2(
                BinaryOp::StrConcat(),
                mk_term::string(" ar"),
                mk_term::string("e"),
            )
            .into(),
        ),
        StrChunk::expr(mk_term::if_then_else(
            Term::Bool(true),
            mk_term::string(" you"),
            mk_term::string(" me"),
        )),
    ];
    inner_chunks.reverse();

    let mut chunks = vec![
        StrChunk::Literal(String::from("Hello, World!")),
        StrChunk::expr(Term::StrChunks(inner_chunks).into()),
        StrChunk::Literal(String::from("?")),
    ];
    chunks.reverse();

    let t: RichTerm = Term::StrChunks(chunks).into();
    assert_eq!(
        eval_no_import(t),
        Ok(Term::Str(String::from("Hello, World! How are you?")))
    );
}

#[test]
fn global_env() {
    let mut global_env = Environment::new();
    let mut resolver = DummyResolver {};
    global_env.insert(
        Ident::from("g"),
        Thunk::new(
            Closure::atomic_closure(Term::Num(1.0).into()),
            IdentKind::Let,
        ),
    );

    let t = mk_term::let_in("x", Term::Num(2.0), mk_term::var("x"));
    assert_eq!(
        eval(t, &global_env, &mut resolver).map(Term::from),
        Ok(Term::Num(2.0))
    );

    let t = mk_term::let_in("x", Term::Num(2.0), mk_term::var("g"));
    assert_eq!(
        eval(t, &global_env, &mut resolver).map(Term::from),
        Ok(Term::Num(1.0))
    );

    // Shadowing of global environment
    let t = mk_term::let_in("g", Term::Num(2.0), mk_term::var("g"));
    assert_eq!(
        eval(t, &global_env, &mut resolver).map(Term::from),
        Ok(Term::Num(2.0))
    );
}

fn mk_env(bindings: Vec<(&str, RichTerm)>) -> Environment {
    bindings
        .into_iter()
        .map(|(id, t)| {
            (
                id.into(),
                Thunk::new(Closure::atomic_closure(t), IdentKind::Let),
            )
        })
        .collect()
}

#[test]
fn substitution() {
    let global_env = mk_env(vec![
        ("glob1", Term::Num(1.0).into()),
        ("glob2", parse("\"Glob2\"").unwrap()),
        ("glob3", Term::Bool(false).into()),
    ]);
    let env = mk_env(vec![
        ("loc1", Term::Bool(true).into()),
        ("loc2", parse("if glob3 then glob1 else glob2").unwrap()),
    ]);

    let t = parse("let x = 1 in if loc1 then 1 + loc2 else glob3").unwrap();
    assert_eq!(
        subst(t, &global_env, &env),
        parse("let x = 1 in if true then 1 + (if false then 1 else \"Glob2\") else false").unwrap()
    );

    let t = parse("switch {`x => [1, glob1], `y => loc2, `z => {id = true, other = glob3}} loc1")
        .unwrap();
    assert_eq!(
        subst(t, &global_env, &env),
        parse("switch {`x => [1, 1], `y => (if false then 1 else \"Glob2\"), `z => {id = true, other = false}} true").unwrap()
    );
}
