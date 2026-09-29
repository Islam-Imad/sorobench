use soroban_sdk::testutils::Logs;
use soroban_sdk::{vec, Address, ConstructorArgs, Env, Symbol, Val};

pub enum Outcome {
    Returned(Val),
    /// The invocation failed. Carries the host error and any runtime-error
    /// log lines the contract emitted during the call.
    Trapped(String),
}

impl Outcome {
    pub fn returned(&self) -> Option<Val> {
        match self {
            Outcome::Returned(v) => Some(*v),
            Outcome::Trapped(_) => None,
        }
    }

    pub fn is_trap(&self) -> bool {
        matches!(self, Outcome::Trapped(_))
    }
}

// TODO: register accounts, related balances, events, etc.
pub struct SorobanEnv {
    env: Env,
    contracts: Vec<Address>,
}

impl SorobanEnv {
    pub fn new() -> Self {
        Self {
            env: Env::default(),
            contracts: Vec::new(),
        }
    }

    pub fn env(&self) -> &Env {
        &self.env
    }

    pub fn contracts(&self) -> &[Address] {
        &self.contracts
    }

    pub fn register_contract(&mut self, contract_wasm: &[u8]) -> Address {
        #[allow(deprecated)]
        let addr = self.env.register_contract_wasm(None, contract_wasm);
        self.contracts.push(addr.clone());
        addr
    }

    pub fn register_contract_with_args<A>(&mut self, contract_wasm: &[u8], args: A) -> Address
    where
        A: ConstructorArgs,
    {
        let addr = self.env.register(contract_wasm, args);
        self.contracts.push(addr.clone());
        addr
    }

    pub fn register_contract_with_arg_vals(
        &mut self,
        contract_wasm: &[u8],
        args: Vec<Val>,
    ) -> Address {
        let mut args_soroban = vec![&self.env];
        for arg in args {
            args_soroban.push_back(arg)
        }
        let addr = self.env.register(contract_wasm, args_soroban);
        self.contracts.push(addr.clone());
        addr
    }

    pub fn invoke_contract(&self, addr: &Address, function_name: &str, args: Vec<Val>) -> Val {
        let func = Symbol::new(&self.env, function_name);
        let mut args_soroban = vec![&self.env];
        for arg in args {
            args_soroban.push_back(arg)
        }
        // To avoid running out of fuel
        self.env.cost_estimate().budget().reset_unlimited();
        self.env.invoke_contract(addr, &func, args_soroban)
    }

    pub fn invoke_contract_expect_error(
        &self,
        addr: &Address,
        function_name: &str,
        args: Vec<Val>,
    ) -> Vec<String> {
        let func = Symbol::new(&self.env, function_name);
        let mut args_soroban = vec![&self.env];
        for arg in args {
            args_soroban.push_back(arg)
        }

        let _ = self
            .env
            .try_invoke_contract::<Val, Val>(addr, &func, args_soroban);

        self.env.logs().all()
    }

    pub fn try_invoke_contract(
        &self,
        addr: &Address,
        function_name: &str,
        args: Vec<Val>,
    ) -> Outcome {
        let func = Symbol::new(&self.env, function_name);
        let mut args_soroban = vec![&self.env];
        for arg in args {
            args_soroban.push_back(arg)
        }
        // To avoid running out of fuel
        self.env.cost_estimate().budget().reset_unlimited();
        let logs_before = self.env.logs().all().len();
        let reason = match self
            .env
            .try_invoke_contract::<Val, Val>(addr, &func, args_soroban)
        {
            Ok(Ok(v)) => return Outcome::Returned(v),
            Ok(Err(_)) => "return value conversion failed".to_string(),
            Err(Ok(err)) => format!("{err:?}"),
            Err(Err(_)) => "invoke error".to_string(),
        };
        let logs: Vec<String> = self
            .env
            .logs()
            .all()
            .into_iter()
            .skip(logs_before)
            .filter(|l| l.to_ascii_lowercase().contains("error"))
            .take(3)
            .map(|l| shorten(&log_data(&l), 300))
            .collect();
        if logs.is_empty() {
            Outcome::Trapped(reason)
        } else {
            Outcome::Trapped(format!("{reason}; log: {}", logs.join(" | ")))
        }
    }
}

/// The payload of a diagnostic-event log line: the text inside `data:"…"`,
/// e.g. `runtime_error: math overflow in test.sol:3:74-79`. Falls back to the
/// whole line.
fn log_data(line: &str) -> String {
    let Some(start) = line.find("data:\"") else {
        return line.to_string();
    };
    let body = &line[start + 6..];
    let body = body.strip_suffix('"').unwrap_or(body);
    body.replace("\\n", " ")
        .trim()
        .trim_end_matches(',')
        .trim()
        .to_string()
}

/// One line, at most `max` chars (char-boundary safe).
pub(crate) fn shorten(s: &str, max: usize) -> String {
    let one: String = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if one.chars().count() > max {
        let cut: String = one.chars().take(max).collect();
        format!("{cut}…")
    } else {
        one
    }
}

impl Default for SorobanEnv {
    fn default() -> Self {
        Self::new()
    }
}
