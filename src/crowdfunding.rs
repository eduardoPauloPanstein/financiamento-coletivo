#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

pub mod crowdfunding_proxy;

#[multiversx_sc::contract]
pub trait Crowdfunding {
    #[init]
    fn init(&self, target: BigUint) {
        self.target().set(&target);
    }

    #[upgrade]
    fn upgrade(&self) {}

    #[view]
    #[storage_mapper("target")]
    fn target(&self) -> SingleValueMapper<BigUint>;
}
