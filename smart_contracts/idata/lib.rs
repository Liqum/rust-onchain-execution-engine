#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod idata {
    use ink_core::env::call::*;
    use ink_core::env::EnvError;
    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_core::storage;
    use ink_prelude::vec::Vec;

    const GET_INTERPRETER: [u8; 4] = [0xB2, 0x77, 0xA3, 0x80];
    
    #[ink(storage)]
    struct Idata {
        tokens_on_edges: storage::Value<u128>,
        started_activities: storage::Value<u128>,
        idata_parent: storage::Value<AccountId>,
        iflow_node: storage::Value<AccountId>,
        index_in_parent: storage::Value<u128>,
        children: storage::HashMap<u128, Vec<AccountId>>,
        inst_count: storage::HashMap<u128, u128>,
    }

    impl Idata {
        /// Initializes the value to the initial value.
        #[ink(constructor)]
        fn new(&mut self) {
            self.tokens_on_edges.set(0);
            self.started_activities.set(0);
            self.idata_parent.set(AccountId::from([0x0; 32]));
            self.iflow_node.set(AccountId::from([0x0; 32]));
            self.index_in_parent.set(0);
        }

        #[ink(message)]
        fn set_activity_marking(&mut self, n_marking: u128) {
            self.started_activities.set(n_marking);
        }

        #[ink(message)]
        fn set_marking(&mut self, n_marking: u128) {
            self.tokens_on_edges.set(n_marking);
        }

        #[ink(message)]
        fn set_parent(&mut self, parent: AccountId, c_flow: AccountId, e_ind: u128) {
            self.index_in_parent.set(e_ind);
            self.idata_parent.set(parent);
            self.iflow_node.set(c_flow);
        }

        #[ink(message)]
        fn add_child(&mut self, e_ind: u128, child: AccountId) {
            self.children
                .mutate_with(&e_ind, |children| children.push(child));
            self.inst_count.mutate_with(&e_ind, |count| *count += 1);
        }

        /// Returns the current state.
        #[ink(message)]
        fn get_marking(&self) -> u128 {
            *self.tokens_on_edges
        }

        #[ink(message)]
        fn get_started_activities(&self) -> u128 {
            *self.started_activities
        }

        #[ink(message)]
        fn get_instance_count(&self, e_ind: u128) -> u128 {
            *self.inst_count.get(&e_ind).unwrap_or(&0)
        }

        #[ink(message)]
        fn decrease_instance_count(&mut self, e_ind: u128) {
            self.inst_count.mutate_with(&e_ind, |count| *count -= 1);
        }

        #[ink(message)]
        fn set_instance_count(&mut self, e_ind: u128, inst_c: u128) {
            self.inst_count.insert(e_ind, inst_c);
        }

        #[ink(message)]
        fn get_index_in_parent(&self) -> u128 {
            *self.index_in_parent
        }

        #[ink(message)]
        fn get_child_proc_inst(&self, e_ind: u128) -> Vec<AccountId> {
            self.children.get(&e_ind).unwrap_or(&Vec::default()).clone()
        }

        #[ink(message)]
        fn get_cflow_inst(&self) -> AccountId {
            *self.iflow_node
        }

        #[ink(message)]
        fn get_parent(&self) -> AccountId {
            *self.idata_parent
        }

        #[ink(message)]
        fn continue_execution(&self, e_ind: u128) -> AccountId {
            let selector = Selector::from(GET_INTERPRETER);
            CallParams::<EnvTypes, AccountId>::eval(self.get_cflow_inst(), selector)
                .fire()
                .unwrap_or(AccountId::from([0x0; 32]))
        }
    }
}
