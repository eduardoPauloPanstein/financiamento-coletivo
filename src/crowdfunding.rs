#![no_std]

// Importações necessárias para o contrato
#[allow(unused_imports)]
use multiversx_sc::imports::*;
pub mod status; // Importa o módulo status, que contém o enum Status
use status::Status; // Usa o enum Status no contrato

pub mod crowdfunding_proxy; // Proxy para interagir com outros contratos

#[multiversx_sc::contract]
pub trait Crowdfunding {
    /// Inicializa o contrato de crowdfunding.
    ///
    /// # Parâmetros
    /// - `target`: Meta de arrecadação em EGLD (BigUint).
    /// - `deadline`: Prazo final para arrecadação, em timestamp Unix (u64).
    ///
    /// # Regras
    /// - A meta (`target`) deve ser maior que 0.
    /// - O prazo final (`deadline`) deve ser no futuro.
    #[init]
    fn init(&self, target: BigUint, deadline: u64) {
        require!(target > 0, "Target must be more than 0");
        self.target().set(target);

        require!(
            deadline > self.blockchain().get_block_timestamp(),
            "Deadline can't be in the past"
        );
        self.deadline().set(deadline);
    }

    /// Atualiza o contrato inteligente. Necessário para upgrades.
    #[upgrade]
    fn upgrade(&self) {}

    /// Retorna a meta de arrecadação.
    #[view(getTarget)]
    #[storage_mapper("target")]
    fn target(&self) -> SingleValueMapper<BigUint>;
    
    /// Retorna o prazo final para arrecadação.
    #[view(getDeadline)]
    #[storage_mapper("deadline")]
    fn deadline(&self) -> SingleValueMapper<u64>;
    
    /// Retorna o depósito de um doador específico.
    ///
    /// # Parâmetros
    /// - `donor`: Endereço do doador.
    #[view(getDeposit)]
    #[storage_mapper("deposit")]
    fn deposit(&self, donor: &ManagedAddress) -> SingleValueMapper<BigUint>;

    /// Permite que um usuário faça uma doação ao contrato.
    ///
    /// # Regras
    /// - Apenas permitido durante o período de arrecadação (`FundingPeriod`).
    /// - O valor doado é adicionado ao depósito do doador.
    #[endpoint]
    #[payable("EGLD")]
    fn fund(&self) {
        let payment = self.call_value().egld();
        require!(
            self.status() == Status::FundingPeriod,
            "cannot fund after deadline"
        );
    
        let caller = self.blockchain().get_caller();
        self.deposit(&caller)
            .update(|deposit| *deposit += &*payment);
    }

    /// Retorna o status atual do crowdfunding.
    ///
    /// # Possíveis Valores
    /// - `FundingPeriod`: Ainda no período de arrecadação.
    /// - `Successful`: Meta atingida.
    /// - `Failed`: Meta não atingida e prazo expirado.
    #[view]
    fn status(&self) -> Status {
        if self.blockchain().get_block_timestamp() <= self.deadline().get() {
            Status::FundingPeriod
        } else if self.get_current_funds() >= self.target().get() {
            Status::Successful
        } else {
            Status::Failed
        }
    }

    /// Retorna o saldo atual do contrato em EGLD.
    #[view(getCurrentFunds)]
    fn get_current_funds(&self) -> BigUint {
        self.blockchain().get_sc_balance(&EgldOrEsdtTokenIdentifier::egld(), 0)
    }

    /// Permite o saque dos fundos do contrato.
    ///
    /// # Regras
    /// - Se o status for `Successful`, apenas o dono do contrato pode sacar.
    /// - Se o status for `Failed`, os doadores podem recuperar seus depósitos.
    #[endpoint]
    fn claim(&self) {
        match self.status() {
            Status::FundingPeriod => sc_panic!("cannot claim before deadline"),
            Status::Successful => {
                let caller = self.blockchain().get_caller();
                require!(
                    caller == self.blockchain().get_owner_address(),
                    "only owner can claim successful funding"
                );

                let sc_balance = self.get_current_funds();
                self.send().direct_egld(&caller, &sc_balance);
            },
            Status::Failed => {
                let caller = self.blockchain().get_caller();
                let deposit = self.deposit(&caller).get();

                if deposit > 0u32 {
                    self.deposit(&caller).clear();
                    self.send().direct_egld(&caller, &deposit);
                }
            },
        }
    }   
}