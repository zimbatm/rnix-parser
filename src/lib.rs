macro_rules! magic {
    (start $init:expr; lookup $fn:ident; $($ident:ident)*) => {
        magic!(def start $init; $($ident)*);
        pub fn $fn(val: SyntaxKind) -> Option<&'static str> {
            match val {
                // Depending on stringify! is bad, but how else can I
                // get the identifier name?
                $($ident => Some(stringify!($ident)),)*
                _ => None
            }
        }
    };
    (def start $init:expr; $ident:ident $($remaining:tt)*) => {
        pub const $ident: SyntaxKind = SyntaxKind($init);
        magic!(def start $init+1; $($remaining)*);
    };
    (def start $init:expr;) => {};
}

pub mod parser;
pub mod tokenizer;
pub mod types;
pub mod value;

pub use self::{
    parser::{AST, nodes},
    value::Value as NixValue
};

pub use rowan::{
    TextUnit, TextRange, SmolStr,
    SyntaxElement, SyntaxNode, SyntaxToken, SyntaxKind, TreeArc,
    TokenAtOffset, WalkEvent, SyntaxNodeChildren, SyntaxElementChildren,
};

use self::tokenizer::Tokenizer;

/// A convenience function for first tokenizing and then parsing given input
pub fn parse(input: &str) -> AST {
    parser::parse(Tokenizer::new(input))
}

#[cfg(test)]
mod tests {
    use super::{NixValue, parse, types::*};

    fn test(code: &str) {
        parse(code).as_result().expect("parsing error");
    }

    #[test]
    fn all() {
        test(include_str!("../test_data/dynamic-attrs.nix"));
        //test(include_str!("../test_data/error.nix"));
        test(include_str!("../test_data/ifs.nix"));
        test(include_str!("../test_data/inherit.nix"));
        test(include_str!("../test_data/interpolation.nix"));
        test(include_str!("../test_data/isset.nix"));
        test(include_str!("../test_data/lambdas.nix"));
        test(include_str!("../test_data/lists.nix"));
        test(include_str!("../test_data/math.nix"));
        test(include_str!("../test_data/meta.nix"));
        test(include_str!("../test_data/pattern.nix"));
        test(include_str!("../test_data/simple-set.nix"));
        test(include_str!("../test_data/with-import-let-in.nix"));
    }
    #[test]
    fn interpolation() {
        let ast = parse(include_str!("../test_data/interpolation.nix"));

        let let_in = LetIn::cast(ast.root().inner()).unwrap();
        let set = Set::cast(let_in.body()).unwrap();
        let entry = set.entries().nth(1).unwrap();
        let value = Interpol::cast(entry.value()).unwrap();

        match &*value.parts() {
            &[
                InterpolPart::Literal(ref s1),
                InterpolPart::Ast(_),
                InterpolPart::Literal(ref s2),
                InterpolPart::Ast(_),
                InterpolPart::Literal(ref s3)
            ]
            if s1 == "The set\'s x value is: "
                && s2 == "\n\nThis line shall have no indention\n  This line shall be indented by 2\n\n\n"
                && s3 == "\n" => (),
            parts => panic!("did not match: {:#?}", parts)
        }
    }
    #[test]
    fn inherit() {
        let ast = parse(include_str!("../test_data/inherit.nix"));

        let let_in = LetIn::cast(ast.root().inner()).unwrap();
        let set = Set::cast(let_in.body()).unwrap();
        let inherit = set.inherits().nth(1).unwrap();

        let from = Ident::cast(inherit.from().unwrap().inner()).unwrap();
        assert_eq!(from.as_str(), "set");
        let mut children = inherit.idents();
        assert_eq!(children.next().unwrap().as_str(), "z");
        assert_eq!(children.next().unwrap().as_str(), "a");
        assert!(children.next().is_none());
    }
    #[test]
    fn math() {
        let ast = parse(include_str!("../test_data/math.nix"));
        let root = Operation::cast(ast.root().inner()).unwrap();
        let operation = Operation::cast(root.value1()).unwrap();

        assert_eq!(root.operator(), OpKind::Add);
        assert_eq!(operation.operator(), OpKind::Add);

        let value = Value::cast(operation.value1()).unwrap();
        assert_eq!(value.to_value(), Ok(NixValue::Integer(1)));
    }
    // #[test]
    // fn remove_pattern_entry() {
    //     let ast = parse("{\n  /* 1 */ a,\n  /* 2 */ b,\n  /* 3 */ c\n}:\na").as_result().unwrap();
    //     let lambda = Lambda::cast(ast.root().inner()).unwrap();
    //     let pattern = Pattern::cast(lambda.arg()).unwrap();

    //     let only_b = pattern.filter_entries(|entry| entry.name().as_str() == "b");
    //     assert_eq!(only_b.node().to_string(), "{\n  /* 2 */ b\n  }:\na");

    //     let without_b = pattern.filter_entries(|entry| entry.name().as_str() != "b");
    //     assert_eq!(without_b.node().to_string(), "{\n  /* 1 */ a,\n  /* 3 */ c\n}:\na");
    // }
    // #[test]
    // fn remove_set_entry() {
    //     let ast = parse("{\n  /* 1 */ a = 3;\n  /* 2 */ b = 2;\n  /* 3 */ c = 3;\n}").as_result().unwrap();
    //     let set = Set::cast(ast.root().inner()).unwrap();

    //     let only_b = set.filter_entries(|entry| {
    //         let key = entry.key();
    //         let mut path = key.path();
    //         Ident::cast(path.next().unwrap()).unwrap().as_str() == "b"
    //     });
    //     assert_eq!(only_b.node().to_string(), "{\n  /* 2 */ b = 2;\n  }");

    //     let without_b = set.filter_entries(|entry| {
    //         let key = entry.key();
    //         let mut path = key.path();
    //         Ident::cast(path.next().unwrap()).unwrap().as_str() != "b"
    //     });
    //     assert_eq!(without_b.node().to_string(), "{\n  /* 1 */ a = 3;\n  /* 3 */ c = 3;\n}");
    // }
}
