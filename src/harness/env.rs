use soroban_sdk::testutils::Logs;
use soroban_sdk::{vec, Address, ConstructorArgs, Env, Symbol, Val};

pub enum Outcome {
    Returned(Val),
    Trapped,
}

impl Outcome {
    pub fn returned(&self) -> Option<Val> {
        match self {
            Outcome::Returned(v) => Some(*v),
            Outcome::Trapped => None,
        }
    }

    pub fn is_trap(&self) -> bool {
        matches!(self, Outcome::Trapped)
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
        match self
            .env
            .try_invoke_contract::<Val, Val>(addr, &func, args_soroban)
        {
            Ok(Ok(v)) => Outcome::Returned(v),
            _ => Outcome::Trapped,
        }
    }
}

impl Default for SorobanEnv {
    fn default() -> Self {
        Self::new()
    }
}
