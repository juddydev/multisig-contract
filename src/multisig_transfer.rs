// src/multisig-transfer.rs

#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod multisig_transfer {
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;

    #[ink(storage)]
    pub struct MultisigTransfer {
        signatories: Vec<AccountId>, // List of signatories
        threshold: u32,              // Minimum number of approvals needed
        proposals: Mapping<u32, Proposal>, // Proposals by ID
        next_proposal_id: u32,       // Unique proposal ID tracker
    }

    #[derive(scale::Encode, scale::Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, Debug))]
    pub struct Proposal {
        proposer: AccountId,    // The account that proposed the transfer
        to: AccountId,          // The recipient address
        amount: Balance,        // Amount to transfer
        approvals: Vec<AccountId>, // List of accounts that approved the proposal
        executed: bool,         // Whether the proposal has been executed
    }

    impl MultisigTransfer {
        #[ink(constructor)]
        pub fn new(signatories: Vec<AccountId>, threshold: u32) -> Self {
            assert!(threshold > 0 && threshold <= signatories.len() as u32, "Invalid threshold");
            Self {
                signatories,
                threshold,
                proposals: Mapping::new(),
                next_proposal_id: 0,
            }
        }

        #[ink(message)]
        pub fn propose_transfer(&mut self, to: AccountId, amount: Balance) -> u32 {
            let caller = self.env().caller();
            assert!(self.is_signatory(&caller), "Only signatories can propose transfers");

            let proposal_id = self.next_proposal_id;
            self.proposals.insert(proposal_id, &Proposal {
                proposer: caller,
                to,
                amount,
                approvals: vec![caller],
                executed: false,
            });
            self.next_proposal_id += 1;
            proposal_id
        }

        #[ink(message)]
        pub fn approve_proposal(&mut self, proposal_id: u32) {
            let caller = self.env().caller();
            assert!(self.is_signatory(&caller), "Only signatories can approve proposals");

            let mut proposal = self.get_proposal(proposal_id);
            assert!(!proposal.executed, "Proposal already executed");
            assert!(!proposal.approvals.contains(&caller), "You have already approved this proposal");

            proposal.approvals.push(caller);
            self.proposals.insert(proposal_id, &proposal);
        }

        #[ink(message)]
        pub fn execute_proposal(&mut self, proposal_id: u32) {
            let mut proposal = self.get_proposal(proposal_id);
            assert!(self.has_sufficient_approvals(&proposal), "Not enough approvals");
            assert!(!proposal.executed, "Proposal already executed");

            // Transfer the funds
            let result = self.env().transfer(proposal.to, proposal.amount);
            assert!(result.is_ok(), "Transfer failed");

            // Mark proposal as executed
            proposal.executed = true;
            self.proposals.insert(proposal_id, &proposal);
        }

        #[ink(message)]
        pub fn get_proposal(&self, proposal_id: u32) -> Proposal {
            self.proposals.get(proposal_id).expect("Proposal does not exist")
        }

        fn is_signatory(&self, account: &AccountId) -> bool {
            self.signatories.contains(account)
        }

        fn has_sufficient_approvals(&self, proposal: &Proposal) -> bool {
            proposal.approvals.len() as u32 >= self.threshold
        }
    }
}
