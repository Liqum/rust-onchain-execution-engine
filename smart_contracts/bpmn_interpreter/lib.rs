#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod bpmn_interpreter {
    use ink_core::env::call::*;
    use ink_core::env::EnvError;
    use ink_core::storage::{self, Flush};
    use ink_prelude::vec::Vec;
    use lazy_static::lazy_static;

    lazy_static! {
        // ifactory
        static ref NEW_INST: Selector = Selector::from([0xE8, 0xF9, 0xD4, 0xF7]);
        // iflow
        static ref GET_FACTORY_INST: Selector = Selector::from([0x17, 0x8E, 0x4F, 0x8E]);
        static ref GET_SUB_PROC_INST: Selector = Selector::from([0xF0,0x84,0x42,0x4F]);
        static ref GET_FIRST_ELEMENT: Selector = Selector::from([0x64,0x2F,0x2F,0xF9]);
        static ref GET_POST_COND: Selector = Selector::from([0xE3,0xCF,0xCB,0x39]);
        static ref GET_ADY_ELEMENTS: Selector = Selector::from([0xE5,0x71,0xE7,0x01]);
        static ref GET_INSTANCE_COUNT: Selector = Selector::from([0xCB,0x60,0x17,0xBC]);
        static ref GET_TYPE_INFO: Selector = Selector::from([0x64,0x2F,0x2F,0xF9]);
        static ref GET_ELEMENT_INFO: Selector = Selector::from([0xED,0x21,0x9E,0x57]);
        static ref GET_ATTACHED_TO: Selector = Selector::from([0xD7,0x96,0xA9,0x47]);
        static ref GET_EVENT_LIST: Selector = Selector::from([0x39,0x1B,0x79,0x46]);
        static ref GET_PRE_COND: Selector = Selector::from([0x7A,0x0F,0x0A,0xC8]);
        static ref GET_EVENT_CODE: Selector = Selector::from([0x9E,0x8F,0x22,0xC3]);
        static ref GET_SUB_PROC_LIST: Selector = Selector::from([0xEF,0xA0,0x8B,0x35]);
        // idata
        static ref SET_PARENT: Selector = Selector::from([0x09, 0x86, 0x1D,0xD5]);
        static ref ADD_CHILD: Selector = Selector::from([0x77,0x01,0x66,0x39]);
        static ref GET_CFLOW_INST: Selector = Selector::from([0xC8,0x87,0xD1,0xC3]);
        static ref GET_MARKING: Selector = Selector::from([0xE1,0x7D,0x66,0x77]);
        static ref GET_STARTED_ACTIVITIES: Selector = Selector::from([0xDC,0x98,0x88,0x81]);
        static ref GET_PARENT: Selector = Selector::from([0xC8,0x79,0x9A,0x47]);
        static ref GET_INDEX_IN_PARENT: Selector = Selector::from([0x8F,0x31,0x80,0xBB]);
        static ref DECREASE_INSTANCE_COUNT: Selector = Selector::from([0x2B,0x87,0x57,0x63]);
        static ref SET_ACTIVITY_MARKING: Selector = Selector::from([0x91,0x0A,0x14,0xC2]);
        static ref SET_MARKING: Selector = Selector::from([0x5A,0x1D,0x86,0x60]);
        static ref GET_CHILD_PROC_INST: Selector = Selector::from([0x9D,0x70,0x2C,0x35]);
        static ref SET_INSTANCE_COUNT: Selector = Selector::from([0x9B,0x70,0x40,0x9A]);
        // Data & scripts
        static ref EXECUTE_SCRIPT: Selector = Selector::from([0xAC,0x52,0xC8,0xD3]);
    }

    #[ink(storage)]
    struct BpmnInterpreter {}

    #[ink(event)]
    struct MessageSent {
        #[ink(topic)]
        ev_code: [u8; 32],
    }

    #[ink(event)]
    struct NewCaseCreated {
        #[ink(topic)]
        p_case: AccountId,
    }

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

    impl BpmnInterpreter {
        #[ink(constructor)]
        fn new(&mut self) {}

        /// Instantiation of Root-Process
        #[ink(message)]
        fn create_root_instance(&self, c_flow: AccountId) -> Result<(), Errors> {
            let factory =
                CallParams::<EnvTypes, AccountId>::eval(c_flow, *GET_FACTORY_INST).fire()?;
            if factory == AccountId::default() {
                Err(Errors::Other)
            } else {
                let p_case = CallParams::<EnvTypes, AccountId>::eval(factory, *NEW_INST).fire()?;
                CallParams::<EnvTypes, ()>::invoke(p_case, *SET_PARENT)
                    .push_arg::<AccountId>(&AccountId::default())
                    .push_arg::<AccountId>(&c_flow)
                    .push_arg::<u128>(&0)
                    .fire()?;
                self.env().emit_event(NewCaseCreated { p_case });
                self.execution_required(c_flow, p_case)?;
                Ok(())
            }
        }

        fn create_instance(&self, e_ind: u128, p_case: AccountId) -> Result<AccountId, Errors> {
            let p_flow = CallParams::<EnvTypes, AccountId>::eval(p_case, *GET_CFLOW_INST).fire()?;
            let c_flow = CallParams::<EnvTypes, AccountId>::eval(p_flow, *GET_SUB_PROC_INST)
                .push_arg::<u128>(&e_ind)
                .fire()?;
            let factory =
                CallParams::<EnvTypes, AccountId>::eval(c_flow, *GET_FACTORY_INST).fire()?;
            if factory == AccountId::default() {
                Err(Errors::Other)
            } else {
                let c_case = CallParams::<EnvTypes, AccountId>::eval(factory, *NEW_INST).fire()?;
                CallParams::<EnvTypes, ()>::invoke(p_case, *SET_PARENT)
                    .push_arg::<AccountId>(&p_case)
                    .push_arg::<AccountId>(&c_flow)
                    .push_arg::<u128>(&e_ind)
                    .fire()?;
                CallParams::<EnvTypes, ()>::invoke(p_case, *ADD_CHILD)
                    .push_arg::<u128>(&e_ind)
                    .push_arg::<AccountId>(&c_case)
                    .fire()?;
                self.execution_required(c_flow, c_case)?;
                Ok(c_case)
            }
        }

        fn execution_required(&self, c_flow: AccountId, p_case: AccountId) -> Result<(), Errors> {
            let e_first = CallParams::<EnvTypes, u128>::eval(c_flow, *GET_FIRST_ELEMENT).fire()?;
            let post_cond = CallParams::<EnvTypes, u128>::eval(c_flow, *GET_POST_COND)
                .push_arg::<u128>(&e_first)
                .fire()?;
            CallParams::<EnvTypes, ()>::invoke(p_case, *SET_MARKING)
                .push_arg::<u128>(&post_cond)
                .fire()?;
            let next: Vec<u128> =
                CallParams::<EnvTypes, Vec<u128>>::eval(c_flow, *GET_ADY_ELEMENTS)
                    .push_arg::<u128>(&e_first)
                    .fire()?;
            if next.len() != 0 {
                self.execute_elements(p_case, next[0])
            } else {
                Ok(())
            }
        }

        fn throw_event(
            &self,
            p_case: AccountId,
            ev_code: [u8; 32],
            ev_info: u128,
        ) -> Result<(), Errors> {
            // This function only receive THROW EVENTS (throw event verification made in function executeElement)
            let mut p_state: [u128; 2] = [0; 2];
            p_state[0] = CallParams::<EnvTypes, u128>::eval(p_case, *GET_MARKING).fire()?;
            p_state[1] =
                CallParams::<EnvTypes, u128>::eval(p_case, *GET_STARTED_ACTIVITIES).fire()?;
            match ev_info {
                ev_info if ev_info & 4096 == 4096 => {
                    // Message (BIT 15), to publish a Message in the Ethereum Event Log
                    self.env().emit_event(MessageSent { ev_code });
                }
                ev_info if ev_info & 5632 == 5632 => {
                    // 9- End, 10- Default, 12- Message
                    // If there are not tokens to consume nor started activities in any subprocess
                    if p_state[0] | p_state[1] == 0 {
                        // Sub-process ended, thus continue execution on parent
                        self.try_catch_event(p_case, ev_code, ev_info, true)?;
                    }
                }
                ev_info => {
                    if ev_info & 2048 == 2048 {
                        // Terminate Event (BIT 11), only END EVENT from standard,
                        // Terminate the execution in the current Sub-process and each children
                        self.kill_process(p_case)?;
                    }
                    // Continue the execution on parent
                    self.try_catch_event(p_case, ev_code, ev_info, p_state[0] | p_state[1] == 0)?;
                }
            }
            Ok(())
        }

        fn try_catch_event(
            &self,
            p_case: AccountId,
            ev_code: [u8; 32],
            ev_info: u128,
            is_inst_compl: bool,
        ) -> Result<(), Errors> {
            let mut catch_case =
                CallParams::<EnvTypes, AccountId>::eval(p_case, *GET_PARENT).fire()?;
            let mut p_case = p_case;
            if catch_case == AccountId::default() {
                // No Parent exist, root node
                if ev_info & 8192 == 8192 {
                    // Error event (BIT 13), only END EVENT from standard, in the root process.
                    self.kill_process(p_case)?;
                }
                return Ok(());
            }
            let c_flow =
                CallParams::<EnvTypes, AccountId>::eval(catch_case, *GET_CFLOW_INST).fire()?;

            let mut p_state: [u128; 2] = [0; 2];
            p_state[0] = CallParams::<EnvTypes, u128>::eval(catch_case, *GET_MARKING).fire()?;
            p_state[1] =
                CallParams::<EnvTypes, u128>::eval(catch_case, *GET_STARTED_ACTIVITIES).fire()?;

            let sub_proc_ind =
                CallParams::<EnvTypes, u128>::eval(p_case, *GET_INDEX_IN_PARENT).fire()?;
            let run_inst_count = if is_inst_compl {
                CallParams::<EnvTypes, u128>::eval(catch_case, *DECREASE_INSTANCE_COUNT)
                    .push_arg(&sub_proc_ind)
                    .fire()?
            } else {
                CallParams::<EnvTypes, u128>::eval(catch_case, *GET_INSTANCE_COUNT)
                    .push_arg(&sub_proc_ind)
                    .fire()?
            };

            if run_inst_count == 0 {
                // Update the corresponding sub-process, call activity as completed
                CallParams::<EnvTypes, ()>::invoke(catch_case, *SET_ACTIVITY_MARKING)
                    .push_arg(&(p_state[1] & !(1 << 1 << sub_proc_ind)))
                    .fire()?
            }

            let sub_proc_info = CallParams::<EnvTypes, u128>::eval(c_flow, *GET_INSTANCE_COUNT)
                .push_arg(&sub_proc_ind)
                .fire()?;

            if ev_info & 7168 != 0 {
                // If receiving 10- Default, 11- Terminate or 12- Message
                if run_inst_count == 0 && sub_proc_info & 4096 != 4096 {
                    // No Instances of the sub-process propagating the event and The sub-process isn't an event-sub-process (BIT 12)
                    let post_c = CallParams::<EnvTypes, u128>::eval(c_flow, *GET_POST_COND)
                        .push_arg::<u128>(&sub_proc_ind)
                        .fire()?;
                    CallParams::<EnvTypes, ()>::invoke(catch_case, *SET_MARKING)
                        .push_arg(&(p_state[0] & !post_c))
                        .fire()?;
                    let first_ady_element =
                        CallParams::<EnvTypes, Vec<u128>>::eval(c_flow, *GET_ADY_ELEMENTS)
                            .push_arg::<u128>(&sub_proc_info)
                            .fire()?[0];
                    self.execute_elements(catch_case, first_ady_element)?;
                } else if sub_proc_info & 128 == 128 {
                    // Multi-Instance Sequential (BIT 7), with pending instances to be started.
                    self.create_instance(sub_proc_ind, p_case)?;
                }
            } else {
                // Signal, Error or Escalation

                // Signals are only handled from the Root-Process by Broadcast, thus the propagation must reach the Root-Process.
                if ev_info & 32768 == 32768 {
                    // Propagating the Signal to the Root-Process
                    while catch_case != AccountId::default() {
                        p_case = catch_case;
                        catch_case =
                            CallParams::<EnvTypes, AccountId>::eval(p_case, *GET_PARENT).fire()?;
                    }
                    self.broadcast_signal(p_case)?;
                    return Ok(());
                }

                let events =
                    CallParams::<EnvTypes, Vec<u128>>::eval(c_flow, *GET_EVENT_LIST).fire()?;
                // The event can be catched only once, unless it is a signal where a broadcast must happen.
                // Precondition: Event-subprocess must appear before boundary events on the event list.
                for event in events {
                    let event_code =
                        CallParams::<EnvTypes, [u8; 32]>::eval(c_flow, *GET_EVENT_CODE)
                            .push_arg::<u128>(&event)
                            .fire()?;
                    if event_code == ev_code {
                        // Verifiying there is a match with the throw-cath events.
                        let catch_ev_info =
                            CallParams::<EnvTypes, u128>::eval(c_flow, *GET_TYPE_INFO)
                                .push_arg::<u128>(&event)
                                .fire()?;
                        let attached_to =
                            CallParams::<EnvTypes, u128>::eval(c_flow, *GET_ATTACHED_TO)
                                .push_arg::<u128>(&event)
                                .fire()?;
                        if catch_ev_info & 6 == 6 {
                            // Start event-sub-process (BIT 6)
                            if catch_ev_info & 16 == 16 {
                                // Interrupting (BIT 4 must be 1, 0 if non-interrupting)
                                // Before starting the event subprocess, the parent is killed
                                self.kill_process(catch_case)?;
                            }
                            // Starting event sub-process
                            self.create_instance(attached_to, p_case)?;
                            // Marking the event-sub-process as started
                            CallParams::<EnvTypes, ()>::invoke(catch_case, *SET_ACTIVITY_MARKING)
                                .push_arg::<u128>(&(p_state[1] | (1 << attached_to)))
                                .fire()?;
                            return Ok(());
                        } else if catch_ev_info & 256 == 256 && attached_to == sub_proc_ind {
                            // Boundary (BIT 6) of the subproces propagating the event
                            if catch_ev_info & 16 == 16 {
                                // Interrupting (BIT 4 must be 1, 0 if non-interrupting)
                                self.kill_process(p_case)?;
                            }
                            // The subprocess propagating the event must be interrupted
                            let post_c = CallParams::<EnvTypes, u128>::eval(c_flow, *GET_POST_COND)
                                .push_arg::<u128>(&event)
                                .fire()?;
                            let first_ady_element =
                                CallParams::<EnvTypes, Vec<u128>>::eval(c_flow, *GET_ADY_ELEMENTS)
                                    .push_arg::<u128>(&event)
                                    .fire()?[0];
                            // Update the marking with the output of the boundary event
                            CallParams::<EnvTypes, ()>::invoke(catch_case, *SET_MARKING)
                                .push_arg::<u128>(&(p_state[0] & !post_c))
                                .fire()?;
                            // Continue the execution of possible internal elements
                            self.execute_elements(catch_case, first_ady_element)?;
                            return Ok(());
                        }
                    }
                }
                // If the event was not caught the propagation continues to the parent unless it's the root process
                self.throw_event(catch_case, ev_code, ev_info)?;
            }
            Ok(())
        }

        fn kill_process(&self, p_case: AccountId) -> Result<(), Errors> {
            let started_activities =
                CallParams::<EnvTypes, u128>::eval(p_case, *GET_STARTED_ACTIVITIES).fire()?;
            CallParams::<EnvTypes, ()>::invoke(p_case, *SET_MARKING)
                .push_arg::<u128>(&0)
                .fire()?;
            CallParams::<EnvTypes, ()>::invoke(p_case, *SET_ACTIVITY_MARKING)
                .push_arg::<u128>(&0)
                .fire()?;
            let c_flow_inst =
                CallParams::<EnvTypes, AccountId>::eval(p_case, *GET_CFLOW_INST).fire()?;
            let children =
                CallParams::<EnvTypes, Vec<u128>>::eval(c_flow_inst, *GET_SUB_PROC_LIST).fire()?;
            for child in children {
                if started_activities & (1 << child) != 0 {
                    let child_proc_inst =
                        CallParams::<EnvTypes, Vec<AccountId>>::eval(p_case, *GET_CHILD_PROC_INST)
                            .push_arg::<u128>(&child)
                            .fire()?;
                    self.kill_processes(child_proc_inst)?;
                }
            }
            Ok(())
        }

        fn kill_processes(&self, p_cases: Vec<AccountId>) -> Result<(), Errors> {
            for p_case in p_cases {
                self.kill_process(p_case)?
            }
            Ok(())
        }

        fn broadcast_signal(&self, p_case: AccountId) -> Result<(), Errors> {
            let c_flow = CallParams::<EnvTypes, AccountId>::eval(p_case, *GET_CFLOW_INST).fire()?;
            let events = CallParams::<EnvTypes, Vec<u128>>::eval(c_flow, *GET_EVENT_LIST).fire()?;
            // let mut p_state: [u128; 2] = [0; 2];
            // p_state[1] =
            //     CallParams::<EnvTypes, u128>::eval(p_case, *GET_STARTED_ACTIVITIES).fire()?;
            for event in events {
                let ev_info = CallParams::<EnvTypes, u128>::eval(c_flow, *GET_TYPE_INFO)
                    .push_arg::<u128>(&event)
                    .fire()?;

                if ev_info & 32780 == 32772 {
                    // Event Catch Signal (BITs 2, 3 [0-catch, 1-throw], 15)
                    let catch_ev_info = CallParams::<EnvTypes, u128>::eval(c_flow, *GET_TYPE_INFO)
                        .push_arg::<u128>(&event)
                        .fire()?;
                    let attached_to = CallParams::<EnvTypes, u128>::eval(c_flow, *GET_ATTACHED_TO)
                        .push_arg::<u128>(&event)
                        .fire()?;

                    if catch_ev_info & 6 == 6 {
                        // Start event-sub-process (BIT 6)
                        if catch_ev_info & 16 == 16 {
                            // Interrupting (BIT 4 must be 1, 0 if non-interrupting)
                            // Before starting the event subprocess, the current process-instance is killed
                            self.kill_process(p_case)?;
                        }
                        self.create_instance(attached_to, p_case)?;
                        CallParams::<EnvTypes, ()>::invoke(p_case, *SET_ACTIVITY_MARKING)
                            .push_arg::<u128>(&(1 << attached_to))
                            .fire()?;
                    } else if catch_ev_info & 256 == 256 {
                        // Boundary (BIT 6) of the subproces propagating the event
                        if catch_ev_info & 16 == 16 {
                            // Interrupting (BIT 4 must be 1, 0 if non-interrupting)
                            // The subprocess propagating the event must be interrupted
                            let child_proc_inst = CallParams::<EnvTypes, Vec<AccountId>>::eval(
                                p_case,
                                *GET_CHILD_PROC_INST,
                            )
                            .push_arg::<u128>(&attached_to)
                            .fire()?;
                            self.kill_processes(child_proc_inst)?;
                        }
                        let marking =
                            CallParams::<EnvTypes, u128>::eval(p_case, *GET_MARKING).fire()?;
                        let post_c = CallParams::<EnvTypes, u128>::eval(c_flow, *GET_POST_COND)
                            .push_arg::<u128>(&event)
                            .fire()?;
                        // Update the marking with the output of the boundary event
                        CallParams::<EnvTypes, ()>::invoke(p_case, *SET_MARKING)
                            .push_arg::<u128>(&(marking & !post_c))
                            .fire()?;
                        let first_ady_element =
                            CallParams::<EnvTypes, Vec<u128>>::eval(c_flow, *GET_ADY_ELEMENTS)
                                .push_arg::<u128>(&event)
                                .fire()?[0];
                        // Continue the execution of possible internal elements
                        self.execute_elements(p_case, first_ady_element)?;
                    } else if ev_info & 160 == 160 {
                        // Start (not Event Subprocess) OR Intermediate Event
                        let marking =
                            CallParams::<EnvTypes, u128>::eval(p_case, *GET_MARKING).fire()?;
                        let post_c = CallParams::<EnvTypes, u128>::eval(c_flow, *GET_POST_COND)
                            .push_arg::<u128>(&event)
                            .fire()?;
                        let pre_c = CallParams::<EnvTypes, u128>::eval(c_flow, *GET_PRE_COND)
                            .push_arg::<u128>(&event)
                            .fire()?;
                        CallParams::<EnvTypes, ()>::invoke(p_case, *SET_MARKING)
                            .push_arg::<u128>(&(marking & !pre_c | post_c))
                            .fire()?;
                        let first_ady_element =
                            CallParams::<EnvTypes, Vec<u128>>::eval(c_flow, *GET_ADY_ELEMENTS)
                                .push_arg::<u128>(&event)
                                .fire()?[0];
                        self.execute_elements(p_case, first_ady_element)?;
                    }
                }
            }
            let c_flow_inst =
                CallParams::<EnvTypes, AccountId>::eval(p_case, *GET_CFLOW_INST).fire()?;
            let children =
                CallParams::<EnvTypes, Vec<u128>>::eval(c_flow_inst, *GET_SUB_PROC_LIST).fire()?;
            let started_activities =
                CallParams::<EnvTypes, u128>::eval(p_case, *GET_STARTED_ACTIVITIES).fire()?;
            for child in children {
                if started_activities & (1 << child) != 0 {
                    let child_proc_inst =
                        CallParams::<EnvTypes, Vec<AccountId>>::eval(p_case, *GET_CHILD_PROC_INST)
                            .push_arg::<u128>(&child)
                            .fire()?;
                    self.broadcast_signals(child_proc_inst)?;
                }
            }
            Ok(())
        }

        fn broadcast_signals(&self, p_cases: Vec<AccountId>) -> Result<(), Errors> {
            for p_case in p_cases {
                self.broadcast_signal(p_case)?
            }
            Ok(())
        }

        #[ink(message)]
        fn execute_elements(&self, p_case: AccountId, e_ind: u128) -> Result<(), Errors> {
            let mut e_ind = e_ind;
            let c_flow = CallParams::<EnvTypes, AccountId>::eval(p_case, *GET_CFLOW_INST).fire()?;
            
            // 0- tokensOnEdges
            // 1- startedActivities
            let mut p_state: [u128; 2] = [0; 2];
            p_state[0] = CallParams::<EnvTypes, u128>::eval(p_case, *GET_MARKING).fire()?;
            p_state[1] =
                CallParams::<EnvTypes, u128>::eval(p_case, *GET_STARTED_ACTIVITIES).fire()?;
            
                // Execution queue and pointers to the first & last element (i.e. basic circular queue implementation)
            let mut queue: [u128; 100] = [0; 100];
            let mut i: usize = 0;
            let mut count: usize = 0;
            queue[count] = e_ind;
            count +=1;
            while i < count {
                e_ind = queue[i];
                i += 1;
                let ([pre_c, post_c, type_info], next) =
                    CallParams::<EnvTypes, ([u128; 3], Vec<u128>)>::eval(
                        c_flow,
                        *GET_ELEMENT_INFO,
                    )
                    .push_arg::<u128>(&e_ind)
                    .fire()?;

                // Verifying Preconditions (i.e. Is the element enabled?)
                match type_info {
                    type_info if type_info & 42 == 42 => {
                        // else if (AND Join)
                        if p_state[0] & pre_c != pre_c {
                            continue;
                        }
                        p_state[0] &= !pre_c;
                    }
                    type_info if type_info & 74 == 74 => {
                        // else if (OR Join)
                        ///// OR Join Implementation //////
                    }
                    type_info
                        if (type_info & 1 == 1
                            || (type_info & 4 == 4 && type_info & 640 != 0)
                            || type_info & 2 == 2) =>
                    {
                        // If (Activity || Intermediate/End Event || Gateway != AND/OR Join)
                        if p_state[0] & pre_c == 0 {
                            continue;
                        }
                        // Removing tokens from input arcs
                        p_state[0] &= !pre_c;
                    }
                    _ => {
                        continue;
                    }
                }

                // Executing current element (If enabled)
                match type_info {
                    type_info if type_info & 65 == 65 => {
                        // (0- Activity, 6- Parallel Multi-Instance)
                        let c_inst =
                            CallParams::<EnvTypes, u128>::eval(c_flow, *GET_INSTANCE_COUNT)
                                .push_arg::<u128>(&e_ind)
                                .fire()?;
                        for _ in 0..c_inst {
                            self.create_instance(e_ind, p_case)?;
                        }
                        p_state[1] |= 1 << e_ind;
                    }
                    type_info
                        if (type_info & 129 == 129
                            || (type_info & 1 == 1
                                && type_info & 48 != 0
                                && type_info & 4096 == 0)) =>
                    {
                        // If (0- Activity, 7- Sequential Multi-Instance) ||
                        // Sub-process(0- Activity, 5- Sub-process) or Call-Activity(0- Activity, 4- Call-Activity)
                        // but NOT Event Sub-process(12- Event Subprocess)
                        let instance = self.create_instance(e_ind, p_case)?;
                        let instance_count =
                            CallParams::<EnvTypes, u128>::eval(c_flow, *GET_INSTANCE_COUNT)
                                .push_arg::<u128>(&e_ind)
                                .fire()?;
                        CallParams::<EnvTypes, ()>::invoke(instance, *SET_INSTANCE_COUNT)
                            .push_arg::<u128>(&e_ind)
                            .push_arg::<u128>(&instance_count)
                            .fire()?;
                        p_state[1] |= 1 << e_ind;
                    }
                    type_info
                        if (type_info & 4105 == 4105
                            || (type_info & 10 == 2 && type_info & 80 != 0)) =>
                    {
                        // (0- Activity, 3- Task, 12- Script) ||
                        // Exclusive(XOR) Split (1- Gateway, 3- Split(0), 4- Exclusive) ||
                        // Inclusive(OR) Split (1- Gateway, 3- Split(0), 6- Inclusive)
                        CallParams::<EnvTypes, u128>::eval(p_case, *EXECUTE_SCRIPT)
                            .push_arg::<u128>(&e_ind)
                            .fire()?;
                        p_state[0] |= CallParams::<EnvTypes, u128>::eval(p_case, *EXECUTE_SCRIPT)
                            .push_arg::<u128>(&e_ind)
                            .fire()?;
                    }
                    type_info
                        if ((type_info & 9 == 9 && type_info & 27657 != 0)
                            || type_info & 2 == 2) =>
                    {
                        // If (User(11), Service(13), Receive(14) or Default(10) Task || Gateways(1) not XOR/OR Split)
                        // The execution of User/Service/Receive is triggered off-chain,
                        // Thus the starting point would be the data contract which executes any script/data-update related to the task.
                        p_state[0] |= post_c;
                    }
                    type_info if type_info & 12 == 12 => {
                        // If (2- Event, 3- Throw(1))
                        CallParams::<EnvTypes, ()>::invoke(p_case, *SET_MARKING)
                            .push_arg::<u128>(&p_state[0])
                            .fire()?;
                        CallParams::<EnvTypes, ()>::invoke(p_case, *SET_ACTIVITY_MARKING)
                            .push_arg::<u128>(&p_state[1])
                            .fire()?;
                        let event_code =
                            CallParams::<EnvTypes, [u8; 32]>::eval(c_flow, *GET_EVENT_CODE)
                                .push_arg::<u128>(&e_ind)
                                .fire()?;
                        self.throw_event(p_case, event_code, type_info)?;
                        let marking =
                            CallParams::<EnvTypes, u128>::eval(p_case, *GET_MARKING).fire()?;
                        let started_activities =
                            CallParams::<EnvTypes, u128>::eval(p_case, *GET_STARTED_ACTIVITIES)
                                .fire()?;
                        if marking | started_activities == 0 {
                            // By throwing the event, a kill was performed so the current instance was terminated
                            return Ok(());
                        }
                        p_state[0] = marking;
                        p_state[1] = started_activities;
                        if type_info & 128 == 128 {
                            // If Intermediate event (BIT 7)
                            p_state[0] |= post_c;
                        }
                    }
                    _ => (),
                }

                // Adding the possible candidates to be executed to the queue.
                // The enablement of the element is checked at the moment it gets out of the queue.
                for next_elem in next {
                    queue[count] = next_elem;
                    count = (count + 1) % 100;
                }
            }

            // Updating the state (storage) after the execution of each internal element.
            CallParams::<EnvTypes, ()>::invoke(p_case, *SET_MARKING)
                .push_arg::<u128>(&p_state[0])
                .fire()?;
            CallParams::<EnvTypes, ()>::invoke(p_case, *SET_ACTIVITY_MARKING)
                .push_arg::<u128>(&p_state[1])
                .fire()?;
            Ok(())
        }
    }
}
