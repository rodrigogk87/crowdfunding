#![no_std]

#[derive(TopEncode, TopDecode, TypeAbi, PartialEq, Clone, Copy)]
pub enum Status {
    FundingPeriod,
    Successful,
    Failed,
}

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait Crowdfunding {
    #[view(getTarget)]
    #[storage_mapper("target")]
    fn target(&self) -> SingleValueMapper<BigUint>;
  
    #[view(getDeadline)]
    #[storage_mapper("deadline")]
    fn deadline(&self) -> SingleValueMapper<u64>;
  
    #[view(getDeposit)]
    #[storage_mapper("deposit")]
    fn deposit(&self, donor: &ManagedAddress) -> SingleValueMapper<BigUint>;
  
    #[init]
    fn init(&self, target: BigUint, deadline: u64) {
        self.target().set(&target);
        self.deadline().set(&deadline);
    }

    #[endpoint]
    #[payable("EGLD")]
    fn fund(&self) {
        let current_time = self.blockchain().get_block_timstamp();
        require!(current_time < self.deadline().get(), "cannot fund after deadline");

        let payment = self.call_value().egld_value();
        let caller = self.blockchain().get_caller();
        self.deposit(&caller).update(|deposit| *deposit += payment);
    }

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
  
    #[view(getCurrentFunds)]
    fn get_current_funds(&self) -> BigUint {
        self.blockchain().get_sc_balance(&TokenIdentifier::egld(), 0)
    }

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