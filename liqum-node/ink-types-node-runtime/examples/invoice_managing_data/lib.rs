#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract(version = "0.1.0", env = NodeRuntimeTypes)]
mod invoice_managing_data {
    use ink_core::env;
    use ink_prelude::*;
    use ink_types_node_runtime::{calls as runtime_calls, NodeRuntimeTypes};
    use ink_core::storage;

    #[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
    #[derive(scale::Encode, scale::Decode)]
    pub enum Errors {
        CheckInError,
        CheckOutError,
    }

    #[ink(storage)]
    struct InvoiceManagingData {
        /// Stores a single `bool` value on the storage.
        accepted: storage::Value<bool>,
        /// Strores respective pallet idata instance id
        idata_instance_id: storage::Value<u64>
    }

    impl InvoiceManagingData {
        #[ink(constructor)]
        fn new(&mut self, idata_instance_id: u64) {
            self.accepted.set(false);
            self.idata_instance_id.set(idata_instance_id);
        }

        #[ink(message)]
        fn execute_script(&mut self, element_index: u128) -> u128 {
            if element_index == 5 {
                if *self.accepted {
                    return 32;
                } else {
                    return 64;
                }
            } else {
                return 0;
            }
        }

        #[ink(message)]
        fn check_in1(&self, element_index: u128) -> Result<(), Errors> {
            if 132 & (1 << element_index) != 0 {
                self.continue_execution(*self.idata_instance_id.get(), element_index);
                Ok(())
            } else {
                Err(Errors::CheckInError)
            }
        }

        #[ink(message)]
        fn check_in2(&mut self, element_index: u128, i1: bool) -> Result<(), Errors> {
            if 8 & (1 << element_index) != 0 {
                self.accepted.set(i1);
                self.continue_execution(*self.idata_instance_id.get(), element_index);
                Ok(())
            } else {
                Err(Errors::CheckInError)
            }
        }

        /// Dispatches a `continue_execution` call to the BpmnInterpreter srml module
        #[ink(message)]
        fn continue_execution(&self, instance_id: u64, element_index: u128) {
            // create the BpmnInterpreter::continue_execution Call
            let continue_execution_call = runtime_calls::continue_execution(instance_id, element_index);
            // dispatch the call to the runtime
            let result = self.env().invoke_runtime(&continue_execution_call);

            // report result to console
            // NOTE: println should only be used on a development chain)
            env::println(&format!("continue_execution invoke_runtime result {:?}", result));
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use sp_keyring::AccountKeyring;

        #[test]
        fn dispatches_continue_execution_call() {
            let calls = InvoiceManagingData::new(0);
            //assert_eq!(calls.env().dispatched_calls().into_iter().count(), 0);
            calls.continue_execution(0, 0);
            //assert_eq!(calls.env().dispatched_calls().into_iter().count(), 1);
        }
    }
}
