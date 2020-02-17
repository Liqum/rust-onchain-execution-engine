#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod invoice_managing_data {
    use idata::{Errors, Idata};
    use ink_core::storage;
    use ink_prelude::vec::Vec;

    #[ink(storage)]
    struct InvoiceManagingData {
        /// Stores a single `bool` value on the storage.
        accepted: storage::Value<bool>,
        idata: storage::Value<Idata>,
    }

    impl InvoiceManagingData {
        #[ink(constructor)]
        fn new(&mut self, idata_hash: Hash) {
            self.accepted.set(false);
            let total_balance = self.env().balance();
            let idata = Idata::new()
                .endowment(total_balance / 4)
                .using_code(idata_hash)
                .instantiate()
                .expect("failed at instantiating the `Idata` contract");
            self.idata.set(idata)
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
                self.idata.continue_execution(element_index)
            } else {
                Err(Errors::CheckInError)
            }
        }

        #[ink(message)]
        fn check_in2(&mut self, element_index: u128, i1: bool) -> Result<(), Errors> {
            if 8 & (1 << element_index) != 0 {
                self.accepted.set(i1);
                self.idata.continue_execution(element_index)
            } else {
                Err(Errors::CheckInError)
            }
        }

        #[ink(message)]
        fn check_out(&self, element_index: u128) -> Result<(), Errors> {
            if 12 & (1 << element_index) == 0 {
                Ok(())
            } else {
                Err(Errors::CheckOutError)
            }
        }

        //IdataImpl
        #[ink(message)]
        fn set_activity_marking(&mut self, n_marking: u128) {
            self.idata.set_activity_marking(n_marking)
        }

        #[ink(message)]
        fn set_marking(&mut self, n_marking: u128) {
            self.idata.set_marking(n_marking);
        }

        #[ink(message)]
        fn set_parent(&mut self, parent: AccountId, child_flow: AccountId, element_index: u128) {
            self.idata.set_parent(parent, child_flow, element_index);
        }

        #[ink(message)]
        fn add_child(&mut self, element_index: u128, child: AccountId) {
            self.idata.add_child(element_index, child)
        }

        /// Returns the current state.
        #[ink(message)]
        fn get_marking(&self) -> u128 {
            self.idata.get_marking()
        }

        #[ink(message)]
        fn get_started_activities(&self) -> u128 {
            self.idata.get_started_activities()
        }

        #[ink(message)]
        fn get_instance_count(&self, element_index: u128) -> u128 {
            self.idata.get_instance_count(element_index)
        }

        #[ink(message)]
        fn decrease_instance_count(&mut self, element_index: u128) -> u128 {
            self.idata.decrease_instance_count(element_index)
        }

        #[ink(message)]
        fn set_instance_count(&mut self, element_index: u128, instance_count: u128) {
            self.idata.set_instance_count(element_index, instance_count)
        }

        #[ink(message)]
        fn get_index_in_parent(&self) -> u128 {
            self.idata.get_index_in_parent()
        }

        #[ink(message)]
        fn get_child_process_instance(&self, element_index: u128) -> Vec<AccountId> {
            self.idata.get_child_process_instance(element_index)
        }

        #[ink(message)]
        fn get_child_flow_instance(&self) -> AccountId {
            self.idata.get_child_flow_instance()
        }

        #[ink(message)]
        fn get_parent(&self) -> AccountId {
            self.idata.get_parent()
        }

        #[ink(message)]
        fn continue_execution(&self, element_index: u128) -> Result<(), Errors> {
            self.idata.continue_execution(element_index)
        }
    }
}
