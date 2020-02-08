#![feature(proc_macro_hygiene)]
#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod iflow {
    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_core::storage::{self, Flush};
    use ink_prelude::vec::Vec;

    #[ink(storage)]
    struct Iflow {
        start_evt: storage::Value<u128>,
        factory: storage::Value<AccountId>,
        interpreter: storage::Value<AccountId>,
        // elemIndex => [preC, postC, type]
        cond_table: storage::HashMap<u128, [u128; 3]>,
        // Element Index => List of elements that can be enabled with the completion of the key element
        next_elem: storage::HashMap<u128, Vec<u128>>,
        // List of Indexes of the subprocesses
        subprocesses: storage::Value<Vec<u128>>,
        // List of Event Indexes defined in the current Subprocess
        events: storage::Value<Vec<u128>>,
        // Event Index => Index of the element where event is attachedTo
        attached_to: storage::HashMap<u128, u128>,
        // Event Index => String representing the code to identify the event (for catching)
        event_code: storage::HashMap<u128, [u8; 32]>,
        // Subprocess Index => Child Subproces address
        p_references: storage::HashMap<u128, AccountId>,
        // Subprocess Index => number of instances
        instance_count: storage::HashMap<u128, u128>,
    }

    impl Iflow {
        //Default values
        #[ink(constructor)]
        fn new(&mut self) {
            self.start_evt.set(0);
            self.factory.set(AccountId::default());
            self.interpreter.set(AccountId::default());
            self.events.set(Vec::new());
        }

        #[ink(message)]
        fn get_precond(&self, e_ind: u128) -> u128 {
            self.cond_table.get(&e_ind).map_or(0, |cond| cond[0])
        }

        #[ink(message)]
        fn get_post_cond(&self, e_ind: u128) -> u128 {
            self.cond_table.get(&e_ind).map_or(0, |cond| cond[1])
        }

        #[ink(message)]
        fn get_type_info(&self, e_ind: u128) -> u128 {
            self.cond_table.get(&e_ind).map_or(0, |cond| cond[2])
        }

        #[ink(message)]
        fn get_first_element(&self) -> u128 {
            *self.start_evt
        }

        #[ink(message)]
        fn get_element_info(&self, e_ind: u128) -> ([u128; 3], Vec<u128>) {
            (
                *self.cond_table.get(&e_ind).unwrap_or(&[0; 3]),
                self.next_elem
                    .get(&e_ind)
                    .unwrap_or(&Vec::default())
                    .clone(),
            )
        }

        #[ink(message)]
        fn get_ady_elements(&self, e_ind: u128) -> Vec<u128> {
            self.next_elem
                .get(&e_ind)
                .unwrap_or(&Vec::default())
                .clone()
        }

        #[ink(message)]
        fn get_subprocess_list(&self) -> Vec<u128> {
            self.subprocesses.clone()
        }

        #[ink(message)]
        fn get_instance_count(&self, e_ind: u128) -> u128 {
            *self.instance_count.get(&e_ind).unwrap_or(&0)
        }

        #[ink(message)]
        fn get_event_code(&self, e_ind: u128) -> [u8; 32] {
            *self.event_code.get(&e_ind).unwrap_or(&[0; 32])
        }

        #[ink(message)]
        fn get_event_list(&self) -> Vec<u128> {
            self.events.clone()
        }

        #[ink(message)]
        fn get_attached_to(&self, e_ind: u128) -> u128 {
            *self.attached_to.get(&e_ind).unwrap_or(&0)
        }

        #[ink(message)]
        fn get_subproc_inst(&self, e_ind: u128) -> AccountId {
            *self
                .p_references
                .get(&e_ind)
                .unwrap_or(&AccountId::default())
        }

        #[ink(message)]
        fn get_factory_inst(&self) -> AccountId {
            *self.factory
        }

        #[ink(message)]
        fn set_factory_inst(&mut self, _factory: AccountId) {
            self.factory.set(_factory)
        }

        #[ink(message)]
        fn get_interpreter_inst(&self) -> AccountId {
            *self.interpreter
        }

        #[ink(message)]
        fn set_interpreter_inst(&mut self, _inerpreter: AccountId) {
            self.interpreter.set(_inerpreter)
        }

        #[ink(message)]
        fn set_element(
            &mut self,
            e_ind: u128,
            pre_c: u128,
            post_c: u128,
            type_info: u128,
            e_code: [u8; 32],
            _next_elem: Vec<u128>,
        ) {
            let _type_info = self.get_type_info(e_ind);
            match _type_info {
                0 => {
                    if type_info & 4 == 4 {
                        //Should be fixed
                        self.events.push(e_ind);
                        if type_info & 36 == 36 {
                            self.start_evt.set(e_ind);
                        }
                        self.event_code.insert(e_ind, e_code);
                    } else if type_info & 33 == 33 {
                        self.subprocesses.push(e_ind);
                    }
                }
                _ => {
                    //"Should be equal!"
                    if type_info != _type_info {
                        return;
                    }
                }
            }
            self.cond_table.insert(e_ind, [pre_c, post_c, type_info]);
            self.next_elem.insert(e_ind, _next_elem);
        }

        #[ink(message)]
        fn link_sub_process(
            &mut self,
            p_ind: u128,
            c_flow_inst: AccountId,
            attached_evts: Vec<u128>,
            count_instances: u128,
        ) {
            //BITs (0, 5) Veryfing the subprocess to link is already in the data structure
            if self.get_type_info(p_ind) & 33 != 33 {
                return;
            }
            self.p_references.insert(p_ind, c_flow_inst);
            for attached_evt in attached_evts.iter() {
                if self.get_type_info(p_ind) & 4 == 4 {
                    self.attached_to.insert(*attached_evt, p_ind);
                }
            }
            self.instance_count.insert(p_ind, count_instances);
        }
    }
}