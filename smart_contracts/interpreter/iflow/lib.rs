#![feature(proc_macro_hygiene)]
#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod iflow {
    use ink_core::storage::{self, Flush};
    use ink_prelude::vec::Vec;

    #[ink(storage)]
    struct Iflow {
        start_event: storage::Value<u128>,
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
        parent_references: storage::HashMap<u128, AccountId>,
        // Subprocess Index => number of instances
        instance_count: storage::HashMap<u128, u128>,
    }

    impl Iflow {
        //Default values
        #[ink(constructor)]
        fn new(&mut self) {
            self.start_event.set(0);
            self.factory.set(AccountId::default());
            self.interpreter.set(AccountId::default());
            self.events.set(Vec::new());
        }

        #[ink(message)]
        fn get_pre_condition(&self, element_index: u128) -> u128 {
            self.cond_table
                .get(&element_index)
                .map_or(0, |cond| cond[0])
        }

        #[ink(message)]
        fn get_post_condition(&self, element_index: u128) -> u128 {
            self.cond_table
                .get(&element_index)
                .map_or(0, |cond| cond[1])
        }

        #[ink(message)]
        fn get_type_info(&self, element_index: u128) -> u128 {
            self.cond_table
                .get(&element_index)
                .map_or(0, |cond| cond[2])
        }

        #[ink(message)]
        fn get_first_element(&self) -> u128 {
            *self.start_event
        }

        #[ink(message)]
        fn get_element_info(&self, element_index: u128) -> ([u128; 3], Vec<u128>) {
            (
                *self.cond_table.get(&element_index).unwrap_or(&[0; 3]),
                self.next_elem
                    .get(&element_index)
                    .unwrap_or(&Vec::default())
                    .clone(),
            )
        }

        #[ink(message)]
        fn get_ady_elements(&self, element_index: u128) -> Vec<u128> {
            self.next_elem
                .get(&element_index)
                .unwrap_or(&Vec::default())
                .clone()
        }

        #[ink(message)]
        fn get_subprocess_list(&self) -> Vec<u128> {
            self.subprocesses.clone()
        }

        #[ink(message)]
        fn get_instance_count(&self, element_index: u128) -> u128 {
            *self.instance_count.get(&element_index).unwrap_or(&0)
        }

        #[ink(message)]
        fn get_event_code(&self, element_index: u128) -> [u8; 32] {
            *self.event_code.get(&element_index).unwrap_or(&[0; 32])
        }

        #[ink(message)]
        fn get_event_list(&self) -> Vec<u128> {
            self.events.clone()
        }

        #[ink(message)]
        fn get_attached_to(&self, element_index: u128) -> u128 {
            *self.attached_to.get(&element_index).unwrap_or(&0)
        }

        #[ink(message)]
        fn get_subprocess_instance(&self, element_index: u128) -> AccountId {
            *self
                .parent_references
                .get(&element_index)
                .unwrap_or(&AccountId::default())
        }

        #[ink(message)]
        fn get_factory_instance(&self) -> AccountId {
            *self.factory
        }

        #[ink(message)]
        fn set_factory_instance(&mut self, _factory: AccountId) {
            self.factory.set(_factory)
        }

        #[ink(message)]
        fn get_interpreter_instance(&self) -> AccountId {
            *self.interpreter
        }

        #[ink(message)]
        fn set_interpreter_instance(&mut self, _inerpreter: AccountId) {
            self.interpreter.set(_inerpreter)
        }

        #[ink(message)]
        fn set_element(
            &mut self,
            element_index: u128,
            pre_condition: u128,
            post_condition: u128,
            type_info: u128,
            event_code: [u8; 32],
            _next_elem: Vec<u128>,
        ) {
            let _type_info = self.get_type_info(element_index);
            match _type_info {
                0 => {
                    if type_info & 4 == 4 {
                        //Should be fixed
                        self.events.push(element_index);
                        if type_info & 36 == 36 {
                            self.start_event.set(element_index);
                        }
                        self.event_code.insert(element_index, event_code);
                    } else if type_info & 33 == 33 {
                        self.subprocesses.push(element_index);
                    }
                }
                _ => {
                    //"Should be equal!"
                    if type_info != _type_info {
                        return;
                    }
                }
            }
            self.cond_table
                .insert(element_index, [pre_condition, post_condition, type_info]);
            self.next_elem.insert(element_index, _next_elem);
        }

        #[ink(message)]
        fn link_sub_process(
            &mut self,
            parent_index: u128,
            child_flow_inst: AccountId,
            attached_events: Vec<u128>,
            count_instances: u128,
        ) {
            //BITs (0, 5) Veryfing the subprocess to link is already in the data structure
            if self.get_type_info(parent_index) & 33 != 33 {
                return;
            }
            self.parent_references.insert(parent_index, child_flow_inst);
            for attached_event in attached_events.iter() {
                if self.get_type_info(parent_index) & 4 == 4 {
                    self.attached_to.insert(*attached_event, parent_index);
                }
            }
            self.instance_count.insert(parent_index, count_instances);
        }
    }
}
