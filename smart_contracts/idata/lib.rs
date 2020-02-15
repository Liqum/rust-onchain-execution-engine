#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod idata {
    use ink_core::env::call::*;
    use ink_core::env::EnvError;
    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_core::storage;
    use ink_prelude::vec::Vec;
    //iflow
    const GET_INTERPRETER: [u8; 4] = [0x54, 0xBC, 0xAE, 0x12];
    //interpreter
    const EXECUTE_ELEMENTS: [u8; 4] = [0xB8, 0x66, 0x1E, 0xE4];

    #[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
    #[derive(scale::Encode, scale::Decode)]
    pub enum Errors {
        EnviromentError,
        Other,
    }

    impl From<EnvError> for Errors {
        fn from(_: EnvError) -> Self {
            Errors::EnviromentError
        }
    }

    #[ink(storage)]
    struct Idata {
        tokens_on_edges: storage::Value<u128>,
        started_activities: storage::Value<u128>,
        idata_parent: storage::Value<AccountId>,
        iflow_node: storage::Value<AccountId>,
        index_in_parent: storage::Value<u128>,
        children: storage::HashMap<u128, Vec<AccountId>>,
        instance_count: storage::HashMap<u128, u128>,
    }

    impl Idata {
        /// Initializes the value to the initial value.
        #[ink(constructor)]
        fn new(&mut self) {
            self.tokens_on_edges.set(0);
            self.started_activities.set(0);
            self.idata_parent.set(AccountId::default());
            self.iflow_node.set(AccountId::default());
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
        fn set_parent(&mut self, parent: AccountId, child_flow: AccountId, element_index: u128) {
            self.index_in_parent.set(element_index);
            self.idata_parent.set(parent);
            self.iflow_node.set(child_flow);
        }

        #[ink(message)]
        fn add_child(&mut self, element_index: u128, child: AccountId) {
            self.children
                .mutate_with(&element_index, |children| children.push(child));
            self.instance_count
                .mutate_with(&element_index, |count| *count += 1);
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
        fn get_instance_count(&self, element_index: u128) -> u128 {
            *self.instance_count.get(&element_index).unwrap_or(&0)
        }

        #[ink(message)]
        fn decrease_instance_count(&mut self, element_index: u128) -> u128 {
            self.instance_count
                .mutate_with(&element_index, |count| *count -= 1);
            self.get_instance_count(element_index)
        }

        #[ink(message)]
        fn set_instance_count(&mut self, element_index: u128, instance_count: u128) {
            self.instance_count.insert(element_index, instance_count);
        }

        #[ink(message)]
        fn get_index_in_parent(&self) -> u128 {
            *self.index_in_parent
        }

        #[ink(message)]
        fn get_child_process_instance(&self, element_index: u128) -> Vec<AccountId> {
            self.children
                .get(&element_index)
                .unwrap_or(&Vec::default())
                .clone()
        }

        #[ink(message)]
        fn get_child_flow_instance(&self) -> AccountId {
            *self.iflow_node
        }

        #[ink(message)]
        fn get_parent(&self) -> AccountId {
            *self.idata_parent
        }

        #[ink(message)]
        fn continue_execution(&self, element_index: u128) -> Result<(), Errors> {
            let get_interpreter_selector = Selector::from(GET_INTERPRETER);
            let execute_elements_selector = Selector::from(EXECUTE_ELEMENTS);
            let interpreter = CallParams::<EnvTypes, AccountId>::eval(
                self.get_child_flow_instance(),
                get_interpreter_selector,
            )
            .fire()?;
            CallParams::<EnvTypes, Result<(), Errors>>::eval(
                self.get_child_flow_instance(),
                execute_elements_selector,
            )
            .push_arg::<AccountId>(&self.env().caller())
            .push_arg::<u128>(&element_index)
            .fire()?
        }
    }
}
