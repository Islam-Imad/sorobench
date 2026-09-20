use crate::decoder::SorobanType;
use solang::sema::ast::{Namespace, StructType, Type};

fn round_width(bits: u16) -> u16 {
    match bits {
        0..=32 => 32,
        33..=64 => 64,
        65..=128 => 128,
        _ => 256,
    }
}

#[derive(Debug, Clone)]
pub struct MappedType {
    pub soroban: SorobanType,
    pub abi: String,
}

fn map_soroban(ns: &Namespace, ty: &Type) -> Option<SorobanType> {
    Some(match ty {
        Type::Bool => SorobanType::Bool,
        Type::Uint(n) => match round_width(*n) {
            32 => SorobanType::U32,
            64 => SorobanType::U64,
            128 => SorobanType::U128,
            _ => SorobanType::U256,
        },
        Type::Int(n) => match round_width(*n) {
            32 => SorobanType::I32,
            64 => SorobanType::I64,
            128 => SorobanType::I128,
            _ => SorobanType::I256,
        },
        Type::Bytes(_) | Type::DynamicBytes => SorobanType::Bytes,
        Type::String => SorobanType::Str,
        Type::Enum(_) => SorobanType::U32,
        Type::Address(_) | Type::Contract(_) => SorobanType::Address,
        Type::Array(elem, dims) => {
            let mut st = map_soroban(ns, elem)?;
            for _ in dims {
                st = SorobanType::Vec(Box::new(st));
            }
            st
        }
        Type::Struct(StructType::UserDefined(n)) => {
            let decl = &ns.structs[*n];
            let mut fields = Vec::with_capacity(decl.fields.len());
            for f in &decl.fields {
                let name = f.id.as_ref()?.name.clone();
                fields.push((name, map_soroban(ns, &f.ty)?));
            }
            SorobanType::Struct(fields)
        }
        _ => return None,
    })
}

// solang `Type` to (`SorobanType`, ABI string)
pub fn map_type(ns: &Namespace, ty: &Type) -> Option<MappedType> {
    let soroban = map_soroban(ns, ty)?;
    Some(MappedType {
        soroban,
        abi: ty.to_signature_string(false, ns),
    })
}

// A resolved function ready to invoke:
pub struct ResolvedFn {
    pub export_name: String,
    pub params: Vec<MappedType>,
    pub returns: Vec<MappedType>,
}

pub fn resolve_overloads(
    ns: &Namespace,
    contract_no: usize,
    bare_name: &str,
) -> Result<Vec<ResolvedFn>, String> {
    let contract = &ns.contracts[contract_no];
    let matches: Vec<usize> = contract
        .functions
        .iter()
        .copied()
        .filter(|&fno| ns.functions[fno].id.name == bare_name)
        .collect();

    if matches.is_empty() {
        return Err(format!("no function `{bare_name}`"));
    }

    let mut resolved = Vec::new();
    let mut last_err = None;
    for fno in matches {
        match build_resolved(ns, contract_no, fno) {
            Ok(r) => resolved.push(r),
            Err(e) => last_err = Some(e),
        }
    }
    if resolved.is_empty() {
        return Err(last_err.unwrap_or_else(|| format!("no mappable `{bare_name}`")));
    }
    Ok(resolved)
}

fn build_resolved(ns: &Namespace, contract_no: usize, fno: usize) -> Result<ResolvedFn, String> {
    let f = &ns.functions[fno];

    let export_name = if f.mangled_name_contracts.contains(&contract_no) {
        f.mangled_name.clone()
    } else {
        f.id.name.clone()
    };

    let map_all = |ps: &[solang::sema::ast::Parameter<Type>], what: &str| {
        ps.iter()
            .map(|p| {
                map_type(ns, &p.ty).ok_or_else(|| format!("unsupported {what} type {:?}", p.ty))
            })
            .collect::<Result<Vec<_>, _>>()
    };
    let params = map_all(&f.params, "param")?;
    let returns = map_all(&f.returns, "return")?;

    Ok(ResolvedFn {
        export_name,
        params,
        returns,
    })
}

pub fn resolve_constructor(ns: &Namespace, contract_no: usize) -> Result<Vec<MappedType>, String> {
    let contract = &ns.contracts[contract_no];
    let ctor = contract
        .functions
        .iter()
        .copied()
        .find(|&fno| ns.functions[fno].is_constructor());
    match ctor {
        None => Ok(Vec::new()),
        Some(fno) => ns.functions[fno]
            .params
            .iter()
            .map(|p| {
                map_type(ns, &p.ty)
                    .ok_or_else(|| format!("unsupported constructor param type {:?}", p.ty))
            })
            .collect(),
    }
}
