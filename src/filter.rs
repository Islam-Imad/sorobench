use solang_parser::pt;

/// Short-circuit a walk: if the sub-walk found a feature, return it.
macro_rules! check {
    ($e:expr) => {
        if let Some(r) = $e {
            return Some(r);
        }
    };
}

/// Why a test was filtered — the capability class of the offending feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    /// EVM-only builtin, gated out of Soroban at sema (`builtin.rs` `target:`).
    SemaGated,
    /// solang offers it (`target: []`) but Soroban has no such concept.
    NoSemantics,
    /// An EVM-only construct / builtin with no Soroban mapping.
    Construct,
}

impl Category {
    pub fn note(self) -> &'static str {
        match self {
            Category::SemaGated => "EVM-only builtin, no Soroban primitive",
            Category::NoSemantics => "offered by solang but no Soroban concept",
            Category::Construct => "EVM-only construct, no Soroban mapping",
        }
    }
}

/// The feature that caused a test to be filtered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilterReason {
    pub feature: &'static str,
    pub category: Category,
}

impl std::fmt::Display for FilterReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} — {}", self.feature, self.category.note())
    }
}

fn reason(feature: &'static str, category: Category) -> Option<FilterReason> {
    Some(FilterReason { feature, category })
}

const CALL_BUILTINS: &[(&str, Category)] = &[
    ("selfdestruct", Category::SemaGated),
    ("ecrecover", Category::SemaGated),
    ("blockhash", Category::SemaGated),
    ("gasleft", Category::SemaGated),
    ("ripemd160", Category::Construct),
];

pub fn filter_source(src: &str) -> Option<FilterReason> {
    let (unit, _comments) = solang_parser::parse(src, 0).ok()?;
    for part in &unit.0 {
        check!(walk_source_unit_part(part));
    }
    None
}

fn walk_source_unit_part(part: &pt::SourceUnitPart) -> Option<FilterReason> {
    use pt::SourceUnitPart as P;
    match part {
        P::ContractDefinition(c) => {
            for cp in &c.parts {
                check!(walk_contract_part(cp));
            }
        }
        P::FunctionDefinition(f) => check!(walk_function(f)),
        P::VariableDefinition(v) => {
            if let Some(e) = &v.initializer {
                check!(walk_expr(e));
            }
        }
        _ => {}
    }
    None
}

fn walk_contract_part(cp: &pt::ContractPart) -> Option<FilterReason> {
    use pt::ContractPart as CP;
    match cp {
        CP::FunctionDefinition(f) => check!(walk_function(f)),
        CP::VariableDefinition(v) => {
            if let Some(e) = &v.initializer {
                check!(walk_expr(e));
            }
        }
        _ => {}
    }
    None
}

fn walk_function(f: &pt::FunctionDefinition) -> Option<FilterReason> {
    // Base-constructor / modifier invocation args, e.g. `Base(tx.origin)`.
    for attr in &f.attributes {
        if let pt::FunctionAttribute::BaseOrModifier(_, base) = attr {
            if let Some(args) = &base.args {
                for e in args {
                    check!(walk_expr(e));
                }
            }
        }
    }
    if let Some(body) = &f.body {
        check!(walk_statement(body));
    }
    None
}

fn walk_statement(s: &pt::Statement) -> Option<FilterReason> {
    use pt::Statement as S;
    match s {
        S::Block { statements, .. } => {
            for st in statements {
                check!(walk_statement(st));
            }
        }
        // Any inline assembly block is EVM-only (no Soroban opcode model).
        S::Assembly { .. } => return reason("assembly", Category::Construct),
        S::Args(_, named) => {
            for na in named {
                check!(walk_expr(&na.expr));
            }
        }
        S::If(_, cond, then, els) => {
            check!(walk_expr(cond));
            check!(walk_statement(then));
            if let Some(e) = els {
                check!(walk_statement(e));
            }
        }
        S::While(_, cond, body) => {
            check!(walk_expr(cond));
            check!(walk_statement(body));
        }
        S::Expression(_, e) => check!(walk_expr(e)),
        S::VariableDefinition(_, _decl, init) => {
            if let Some(e) = init {
                check!(walk_expr(e));
            }
        }
        S::For(_, init, cond, incr, body) => {
            if let Some(s) = init {
                check!(walk_statement(s));
            }
            if let Some(e) = cond {
                check!(walk_expr(e));
            }
            if let Some(e) = incr {
                check!(walk_expr(e));
            }
            if let Some(s) = body {
                check!(walk_statement(s));
            }
        }
        S::DoWhile(_, body, cond) => {
            check!(walk_statement(body));
            check!(walk_expr(cond));
        }
        S::Return(_, e) => {
            if let Some(e) = e {
                check!(walk_expr(e));
            }
        }
        S::Revert(_, _, exprs) => {
            for e in exprs {
                check!(walk_expr(e));
            }
        }
        S::RevertNamedArgs(_, _, named) => {
            for na in named {
                check!(walk_expr(&na.expr));
            }
        }
        S::Emit(_, e) => check!(walk_expr(e)),
        S::Try(_, e, ok, catches) => {
            check!(walk_expr(e));
            if let Some((_, body)) = ok {
                check!(walk_statement(body));
            }
            for c in catches {
                let body = match c {
                    pt::CatchClause::Simple(_, _, b) | pt::CatchClause::Named(_, _, _, b) => b,
                };
                check!(walk_statement(body));
            }
        }
        S::Continue(_) | S::Break(_) | S::Error(_) => {}
    }
    None
}

fn walk_expr(e: &pt::Expression) -> Option<FilterReason> {
    use pt::Expression as E;
    match e {
        E::MemberAccess(_, recv, member) => {
            check!(match_member(recv, member));
            check!(walk_expr(recv));
        }
        E::FunctionCall(_, callee, args) => {
            check!(match_call(callee));
            check!(walk_expr(callee));
            for a in args {
                check!(walk_expr(a));
            }
        }
        E::FunctionCallBlock(_, callee, block) => {
            check!(match_call(callee));
            check!(walk_expr(callee));
            check!(walk_statement(block));
        }
        E::NamedFunctionCall(_, callee, args) => {
            check!(match_call(callee));
            check!(walk_expr(callee));
            for na in args {
                check!(walk_expr(&na.expr));
            }
        }
        E::ConditionalOperator(_, a, b, c) => {
            check!(walk_expr(a));
            check!(walk_expr(b));
            check!(walk_expr(c));
        }
        E::ArraySubscript(_, a, b) => {
            check!(walk_expr(a));
            if let Some(b) = b {
                check!(walk_expr(b));
            }
        }
        E::ArraySlice(_, a, b, c) => {
            check!(walk_expr(a));
            if let Some(b) = b {
                check!(walk_expr(b));
            }
            if let Some(c) = c {
                check!(walk_expr(c));
            }
        }
        E::ArrayLiteral(_, xs) => {
            for x in xs {
                check!(walk_expr(x));
            }
        }
        // Everything else (unary / binary / assign ops, `new`, parenthesis,
        // literals, `Variable`, `Type`, `List`) has at most two sub-expression
        // components, which `components()` exposes without us re-enumerating the
        // whole ~60-variant enum.
        _ => {
            let (l, r) = e.components();
            if let Some(l) = l {
                check!(walk_expr(l));
            }
            if let Some(r) = r {
                check!(walk_expr(r));
            }
        }
    }
    None
}

/// A bare call `name(...)` to a filtered EVM builtin.
fn match_call(callee: &pt::Expression) -> Option<FilterReason> {
    if let pt::Expression::Variable(id) = callee.strip_parentheses() {
        for (name, category) in CALL_BUILTINS {
            if id.name == *name {
                return reason(name, *category);
            }
        }
    }
    None
}

/// A member access: the EVM globals (`msg.value`, `tx.origin`, `block.*`) and
/// the low-level `.delegatecall` / `.staticcall` (receiver-agnostic).
fn match_member(recv: &pt::Expression, member: &pt::Identifier) -> Option<FilterReason> {
    match member.name.as_str() {
        "delegatecall" => return reason(".delegatecall", Category::Construct),
        "staticcall" => return reason(".staticcall", Category::Construct),
        _ => {}
    }
    let recv_name = match recv.strip_parentheses() {
        pt::Expression::Variable(id) => Some(id.name.as_str()),
        _ => None,
    };
    match (recv_name, member.name.as_str()) {
        (Some("msg"), "value") => reason("msg.value", Category::NoSemantics),
        (Some("tx"), "origin") => reason("tx.origin", Category::SemaGated),
        (Some("tx"), "gasprice") => reason("tx.gasprice", Category::NoSemantics),
        (Some("block"), "coinbase") => reason("block.coinbase", Category::SemaGated),
        (Some("block"), "difficulty") => reason("block.difficulty", Category::SemaGated),
        (Some("block"), "prevrandao") => reason("block.prevrandao", Category::SemaGated),
        (Some("block"), "gaslimit") => reason("block.gaslimit", Category::SemaGated),
        (Some("block"), "basefee") => reason("block.basefee", Category::SemaGated),
        (Some("block"), "chainid") => reason("block.chainid", Category::SemaGated),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feat(src: &str) -> Option<&'static str> {
        filter_source(src).map(|r| r.feature)
    }

    #[test]
    fn detects_call_builtins() {
        assert_eq!(
            feat("contract C{function f(address a)public{selfdestruct(payable(a));}}"),
            Some("selfdestruct")
        );
        assert_eq!(
            feat("contract C{function f(bytes32 h,uint8 v,bytes32 r,bytes32 s)public pure returns(address){return ecrecover(h,v,r,s);}}"),
            Some("ecrecover")
        );
        assert_eq!(
            feat("contract C{function f()public view returns(bytes32){return blockhash(1);}}"),
            Some("blockhash")
        );
        assert_eq!(
            feat("contract C{function f()public view returns(uint){return gasleft();}}"),
            Some("gasleft")
        );
        assert_eq!(
            feat("contract C{function f(bytes memory b)public pure returns(bytes20){return ripemd160(b);}}"),
            Some("ripemd160")
        );
    }

    #[test]
    fn detects_global_members() {
        assert_eq!(
            feat("contract C{function f()public payable returns(uint){return msg.value;}}"),
            Some("msg.value")
        );
        assert_eq!(
            feat("contract C{function f()public view returns(address){return tx.origin;}}"),
            Some("tx.origin")
        );
        assert_eq!(
            feat("contract C{function f()public view returns(uint){return tx.gasprice;}}"),
            Some("tx.gasprice")
        );
        for (src_member, want) in [
            ("coinbase", "block.coinbase"),
            ("prevrandao", "block.prevrandao"),
            ("gaslimit", "block.gaslimit"),
            ("basefee", "block.basefee"),
            ("chainid", "block.chainid"),
        ] {
            let src = format!(
                "contract C{{function f()public view returns(uint){{return uint(block.{});}}}}",
                src_member
            );
            assert_eq!(feat(&src), Some(want), "block.{src_member}");
        }
    }

    #[test]
    fn detects_lowlevel_and_assembly() {
        assert_eq!(
            feat("contract C{function f(address a)public{a.delegatecall(\"\");}}"),
            Some(".delegatecall")
        );
        assert_eq!(
            feat("contract C{function f(address a)public view{a.staticcall(\"\");}}"),
            Some(".staticcall")
        );
        assert_eq!(
            feat("contract C{function f()public{assembly{let x:=1}}}"),
            Some("assembly")
        );
    }

    #[test]
    fn detects_features_deep_in_the_tree() {
        // in a require() arg (nested call args)
        assert_eq!(
            feat("contract C{function f()public view{require(tx.origin != address(0));}}"),
            Some("tx.origin")
        );
        // in a ternary
        assert_eq!(
            feat("contract C{function f(bool b)public view returns(address){return b?tx.origin:msg.sender;}}"),
            Some("tx.origin")
        );
        // in a state-variable initializer
        assert_eq!(
            feat("contract C{address a = tx.origin;}"),
            Some("tx.origin")
        );
        // in a base-constructor argument
        assert_eq!(
            feat("contract B{constructor(address a){}} contract C is B{constructor() B(tx.origin){}}"),
            Some("tx.origin")
        );
        // inside a for loop body
        assert_eq!(
            feat("contract C{function f()public{for(uint i=0;i<1;i++){selfdestruct(payable(msg.sender));}}}"),
            Some("selfdestruct")
        );
    }

    #[test]
    fn portable_source_is_not_filtered() {
        assert_eq!(
            feat("contract C{function f(uint x)public pure returns(uint){return x+1;}}"),
            None
        );
        // block.number / block.timestamp / msg.sender are GAPS, not filtered.
        assert_eq!(
            feat("contract C{function f()public view returns(uint){return block.number+block.timestamp;}}"),
            None
        );
        assert_eq!(
            feat("contract C{function f()public view returns(address){return msg.sender;}}"),
            None
        );
    }

    #[test]
    fn ignores_comments_and_strings() {
        assert_eq!(feat("contract C{ // calls selfdestruct here\n function f()public pure returns(uint){return 1;} }"), None);
        assert_eq!(feat("contract C{ /* selfdestruct(x) */ function f()public pure returns(uint){return 1;} }"), None);
        assert_eq!(feat("contract C{function f()public pure returns(string memory){return \"tx.origin selfdestruct\";}}"), None);
    }

    #[test]
    fn does_not_overmatch_user_members() {
        // a struct field named `value` must not trip `msg.value`.
        assert_eq!(
            feat("contract C{struct S{uint value;} function f(S memory s)public pure returns(uint){return s.value;}}"),
            None
        );
        // a user `.transfer(` / `.send(` method is deferred to the ns pass.
        assert_eq!(
            feat("contract C{function f(address a)public{IToken(a).transfer(a,1);}}"),
            None
        );
        // a plain identifier `origin` (not `tx.origin`) is fine.
        assert_eq!(
            feat("contract C{function f()public pure returns(uint){uint origin=1;return origin;}}"),
            None
        );
    }

    #[test]
    fn unparseable_source_is_none() {
        assert_eq!(feat("this is not solidity {{{"), None);
    }
}
