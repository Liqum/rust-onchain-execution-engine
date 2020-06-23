#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Codec, Decode, Encode};
use frame_support::{
    decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure, Parameter,
};
use frame_system::{self as system, ensure_signed, RawOrigin};
use sp_runtime::traits::{CheckedAdd, MaybeSerialize, Member};
use sp_std::{collections::btree_map::BTreeMap, prelude::*};

mod errors;
use contracts::{CodeHash, ContractAddressFor};
use errors::*;

const ENDOWMENT: u32 = 1000;
const GAS: u32 = 500_000;

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Clone, PartialEq)]
pub struct Iflow<T: Trait> {
    start_evt: u128,
    /// elemIndex => [preC, postC, type]
    cond_table: BTreeMap<u128, [u128; 3]>,
    /// Element Index => List of elements that can be enabled with the completion of the key element
    next_elem: BTreeMap<u128, Vec<u128>>,
    /// List of Indexes of the subprocesses
    subprocesses: Vec<u128>,
    /// List of Event Indexes defined in the current Subprocess
    events: Vec<u128>,
    /// Event Index => Index of the element where event is attachedTo
    attached_to: BTreeMap<u128, u128>,

    event_code: BTreeMap<u128, [u8; 32]>,
    parent_references: BTreeMap<u128, T::InstanceId>,
    instance_count: BTreeMap<u128, u128>,
    factory: Ifactory<T>,
}

impl<T: Trait> Default for Iflow<T> {
    fn default() -> Self {
        Self {
            start_evt: 0,
            cond_table: BTreeMap::new(),
            next_elem: BTreeMap::new(),
            subprocesses: vec![],
            events: vec![],
            attached_to: BTreeMap::new(),
            event_code: BTreeMap::new(),
            parent_references: BTreeMap::new(),
            instance_count: BTreeMap::new(),
            factory: Ifactory::<T>::default(),
        }
    }
}

impl<T: Trait> Iflow<T> {
    fn get_pre_condition(&self, element_index: u128) -> u128 {
        if let Some(cond_table) = self.cond_table.get(&element_index) {
            cond_table[0]
        } else {
            0
        }
    }

    fn get_post_condition(&self, element_index: u128) -> u128 {
        if let Some(cond_table) = self.cond_table.get(&element_index) {
            cond_table[1]
        } else {
            0
        }
    }

    fn get_type_info(&self, element_index: u128) -> u128 {
        if let Some(cond_table) = self.cond_table.get(&element_index) {
            cond_table[2]
        } else {
            0
        }
    }

    fn get_element_info(&self, element_index: u128) -> ([u128; 3], &[u128]) {
        (
            self.cond_table[&element_index],
            &self.next_elem[&element_index],
        )
    }

    fn get_first_elem(&self) -> u128 {
        self.start_evt
    }

    fn get_ady_elements(&self, element_index: u128) -> &[u128] {
        &self.next_elem[&element_index]
    }

    fn get_attached_to(&self, element_index: u128) -> u128 {
        self.attached_to[&element_index]
    }

    fn get_sub_process_instance(&self, element_index: u128) -> T::InstanceId {
        self.parent_references[&element_index]
    }

    fn get_sub_process_list(&self) -> &[u128] {
        &self.subprocesses
    }

    fn get_event_code(&self, element_index: u128) -> [u8; 32] {
        self.event_code[&element_index]
    }

    fn get_event_list(&self) -> &[u128] {
        &self.events
    }

    fn get_instance_count(&self, element_index: u128) -> u128 {
        self.instance_count[&element_index]
    }

    fn get_factory_instance_mut(&mut self) -> &mut Ifactory<T> {
        &mut self.factory
    }

    fn get_factory_instance(&self) -> &Ifactory<T> {
        &self.factory
    }

    fn set_factory_instance(&mut self, factory: Ifactory<T>) {
        self.factory = factory;
    }

    fn set_element(
        &mut self,
        element_index: u128,
        pre_condition: u128,
        post_condition: u128,
        type_info: u128,
        event_code: [u8; 32],
        _next_elem: Vec<u128>,
    ) {
        if type_info & 4 == 4 {
            self.events.push(element_index);
            if type_info & 36 == 36 {
                self.start_evt = element_index;
            }
            self.event_code.insert(element_index, event_code);
        } else if type_info & 33 == 33 {
            self.subprocesses.push(element_index);
        }
        self.cond_table
            .insert(element_index, [pre_condition, post_condition, type_info]);
        self.next_elem.insert(element_index, _next_elem);
    }

    fn link_sub_process(
        &mut self,
        parent_index: u128,
        child_flow_inst: T::InstanceId,
        attached_events: Vec<u128>,
        count_instances: u128,
    ) {
        self.parent_references.insert(parent_index, child_flow_inst);
        for attached_event in attached_events.into_iter() {
            if self.get_type_info(parent_index) & 4 == 4 {
                self.attached_to.insert(attached_event, parent_index);
            }
        }
        self.instance_count.insert(parent_index, count_instances);
    }
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Clone, PartialEq)]
pub struct Idata<T: Trait> {
    tokens_on_edges: u128,
    started_activities: u128,
    idata_parent: Option<T::InstanceId>,
    iflow_node: T::InstanceId,
    index_in_parent: u128,
    children: BTreeMap<u128, Vec<T::InstanceId>>,
    instance_count: BTreeMap<u128, u128>,
}

impl<T: Trait> Default for Idata<T> {
    fn default() -> Self {
        Self {
            tokens_on_edges: 0,
            started_activities: 0,
            idata_parent: None,
            iflow_node: T::InstanceId::default(),
            index_in_parent: 0,
            children: BTreeMap::default(),
            instance_count: BTreeMap::default(),
        }
    }
}

impl<T: Trait> Idata<T> {
    fn set_marking(&mut self, n_marking: u128) {
        self.tokens_on_edges = n_marking
    }

    fn set_activity_marking(&mut self, n_marking: u128) {
        self.started_activities = n_marking
    }

    fn set_parent(
        &mut self,
        idata_parent: Option<T::InstanceId>,
        iflow_node: T::InstanceId,
        index_in_parent: u128,
    ) {
        self.index_in_parent = index_in_parent;
        self.idata_parent = idata_parent;
        self.iflow_node = iflow_node;
    }

    fn add_child(&mut self, element_index: u128, child_id: T::InstanceId) {
        if let Some(children) = self.children.get_mut(&element_index) {
            children.push(child_id);
        } else {
            self.children.insert(element_index, vec![child_id]);
        }
        self.increment_instance_count(element_index)
    }

    fn increment_instance_count(&mut self, element_index: u128) {
        if let Some(instance_count) = self.instance_count.get_mut(&element_index) {
            *instance_count += 1;
        } else {
            self.instance_count.insert(element_index, 1);
        }
    }

    fn decrement_instance_count(&mut self, element_index: u128) {
        if let Some(instance_count) = self.instance_count.get_mut(&element_index) {
            *instance_count -= 1;
        }
    }

    fn set_instance_count(&mut self, element_index: u128, new_instance_count: u128) {
        if let Some(instance_count) = self.instance_count.get_mut(&element_index) {
            *instance_count = new_instance_count;
        } else {
            self.instance_count
                .insert(element_index, new_instance_count);
        }
    }

    fn get_index_in_parent(&self) -> u128 {
        self.index_in_parent
    }

    fn get_child_process_instances(&self, element_index: u128) -> &[T::InstanceId] {
        &self.children[&element_index]
    }

    fn get_flow_node(&self) -> T::InstanceId {
        self.iflow_node
    }

    fn get_idata_parent(&self) -> Option<T::InstanceId> {
        self.idata_parent
    }

    fn get_started_activities(&self) -> u128 {
        self.started_activities
    }

    fn get_marking(&self) -> u128 {
        self.tokens_on_edges
    }

    fn get_instance_count(&self, element_index: u128) -> u128 {
        self.instance_count[&element_index]
    }

    fn continue_execution(&self, element_index: u128) -> Result<(), &'static str> {
        // Call bpmn interpreter execution on given index
        Module::<T>::execute_elements(self.get_flow_node(), element_index)
    }
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Clone, PartialEq)]
pub struct Ifactory<T: Trait> {
    /// Data & scripts hash
    data_hash: T::Hash,
    address: Option<T::AccountId>,
    instantiate_selector: Vec<u8>,
    execute_script_selector: Vec<u8>,
}

impl<T: Trait> Default for Ifactory<T> {
    fn default() -> Self {
        Self {
            data_hash: T::Hash::default(),
            address: None,
            instantiate_selector: vec![],
            execute_script_selector: vec![],
        }
    }
}

impl<T: Trait> Ifactory<T> {
    fn new(
        data_hash: T::Hash,
        instantiate_selector: Vec<u8>,
        execute_script_selector: Vec<u8>,
    ) -> Self {
        Self {
            data_hash,
            address: None,
            instantiate_selector,
            execute_script_selector,
        }
    }

    pub fn set_instantiate_selector(&mut self, instantiate_selector: Vec<u8>) {
        self.instantiate_selector = instantiate_selector
    }

    pub fn get_address(&self) -> &Option<T::AccountId> {
        &self.address
    }

    pub fn get_instantiate_selector(&self) -> &[u8] {
        &self.instantiate_selector
    }

    pub fn get_execute_script_selector(&self) -> &[u8] {
        &self.execute_script_selector
    }

    fn new_instance(&mut self, intance_id: T::InstanceId) -> Result<T::AccountId, &'static str> {
        
        // Initialize new instance of data & scripts contract
        if let Some(address) = &self.address {
            Ok(address.clone())
        } else {

            let encoded_instance_id = u128::encode(&intance_id.into());
            let input_data = [self.get_instantiate_selector(), &encoded_instance_id[..]].concat();

            let contract_address = T::ContractAddressFor::contract_address_for(
                &self.data_hash,
                &input_data,
                &T::AccountId::default(),
            );
            let origin = T::Origin::from(RawOrigin::Root);

            ensure!(
                <contracts::Module<T>>::instantiate(
                    origin,
                    ENDOWMENT.into(),
                    GAS.into(),
                    self.data_hash,
                    input_data
                )
                .is_ok(),
                INSTANTIATION_ERROR
            );

            self.address = Some(contract_address.clone());

            Ok(contract_address)
        }
    }
}

/// The pallet's configuration trait.
pub trait Trait: frame_system::Trait + contracts::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

    type ContractAddressFor: contracts::ContractAddressFor<CodeHash<Self>, Self::AccountId>;

    /// Type of identifier for instances.
    type InstanceId: Parameter
        + Member
        + CheckedAdd
        + Codec
        + Default
        + Copy
        + Into<u128>
        + MaybeSerialize
        + PartialEq;
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as BpmnInterpreter {
        pub IflowById get(fn iflow_by_id): map hasher(blake2_128_concat) T::InstanceId => Iflow<T>;

        pub IdataById get(fn idata_by_id): map hasher(blake2_128_concat) T::InstanceId => Idata<T>;

        InstanceIdCount get(fn instance_id_count): T::InstanceId;
    }
}

// The pallet's dispatchable functions.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        // Initializing events
        fn deposit_event() = default;

        #[weight = 10_000]
        pub fn set_element(
            origin,
            iflow_index: T::InstanceId,
            element_index: u128,
            pre_condition: u128,
            post_condition: u128,
            type_info: u128,
            event_code: [u8; 32],
            _next_elem: Vec<u128>
        ) -> DispatchResult {
            ensure_signed(origin)?;
            let iflow = Self::ensure_iflow_instance_exists(iflow_index)?;

            let _type_info = iflow.get_type_info(element_index);
            if  _type_info != 0 {
                ensure!(_type_info == type_info, "Should be equal");
            }

            //
            // == MUTATION SAFE ==
            //

            if <IflowById<T>>::contains_key(iflow_index) {
                <IflowById<T>>::mutate(iflow_index, |inner_iflow|
                    inner_iflow.set_element(
                        element_index,
                        pre_condition,
                        post_condition,
                        type_info,
                        event_code,
                        _next_elem)
                );
            } else {
                let mut iflow = Iflow::default();
                iflow.set_element(
                    element_index,
                    pre_condition,
                    post_condition,
                    type_info,
                    event_code,
                    _next_elem
                );
                <IflowById<T>>::insert(iflow_index, iflow);
            }
            Ok(())
        }

        #[weight = 10_000]
        pub fn link_sub_process(
            origin,
            iflow_index: T::InstanceId,
            parent_index: u128,
            child_flow_inst: T::InstanceId,
            attached_events: Vec<u128>,
            count_instances: u128,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            let iflow = Self::ensure_iflow_instance_exists(iflow_index)?;
            Self::ensure_subprocess_to_link_in_data_structure(&iflow, parent_index)?;

            //
            // == MUTATION SAFE ==
            //

            <IflowById<T>>::mutate(iflow_index, |inner_iflow|
            inner_iflow.link_sub_process(
                parent_index,
                child_flow_inst,
                attached_events,
                count_instances,
            ));
            Ok(())
        }

        #[weight = 10_000]
        pub fn set_factory_instance(
            origin,
            instance_id: T::InstanceId,
            data_hash: T::Hash,
            instantiate_selector: Vec<u8>,
            execute_script_selector: Vec<u8>
        ) -> DispatchResult {
            ensure_signed(origin)?;
            let factory = Ifactory::new(data_hash, instantiate_selector, execute_script_selector);

            //
            // == MUTATION SAFE ==
            //

            <IflowById<T>>::mutate(instance_id, |iflow| iflow.set_factory_instance(factory));
            Self::deposit_event(RawEvent::FactorySet(instance_id, data_hash));
            Ok(())
        }

        #[weight = 10_000]
        pub fn continue_execution(origin, instance_id: T::InstanceId, element_index: u128) -> DispatchResult {
            ensure_signed(origin)?;
            let idata = Self::ensure_idata_instance_exists(instance_id)?;

            //
            // == MUTATION SAFE ==
            //

            idata.continue_execution(element_index)?;
            Ok(())
        }

        /// Instantiation of Root-Process
        #[weight = 10_000]
        pub fn create_root_instance(origin, parent_case: T::InstanceId) -> DispatchResult {

            ensure_signed(origin)?;

            let mut iflow = Self::ensure_iflow_instance_exists(parent_case)?;

            let contract_id = iflow.get_factory_instance_mut().new_instance(parent_case)?;

            //
            // == MUTATION SAFE ==
            //

            <IflowById<T>>::insert(parent_case, iflow.clone());


            let mut idata = Idata::default();

            idata.set_parent(None, parent_case, 0);

            <IdataById<T>>::insert(parent_case, idata);

            Self::deposit_event(RawEvent::NewCaseCreated(contract_id));

            Self::execution_required(parent_case, &iflow)?;

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    /// BPMN Interpreter logic

    /// Instantiation of a sub-process by its parent
    pub fn create_instance(
        element_index: u128,
        parent_case: T::InstanceId,
    ) -> Result<(), &'static str> {
        ensure!(
            parent_case != T::InstanceId::default(),
            "Parent case should not be root"
        );

        let idata = Self::ensure_idata_instance_exists(parent_case)?;

        let parent_flow_id = idata.get_flow_node();
        let parent_flow = Self::ensure_iflow_instance_exists(parent_flow_id)?;

        let child_flow_id = parent_flow.get_sub_process_instance(element_index);
        let mut child_flow = Self::ensure_iflow_instance_exists(child_flow_id)?;

        let contract_id = child_flow
            .get_factory_instance_mut()
            .new_instance(child_flow_id)?;

        //
        // == MUTATION SAFE ==
        //

        <IflowById<T>>::insert(child_flow_id, child_flow.clone());

        <IdataById<T>>::mutate(parent_case, |inner_data| {
            inner_data.set_parent(Some(parent_case), child_flow_id, element_index);
            inner_data.add_child(element_index, child_flow_id)
        });

        Self::deposit_event(RawEvent::NewCaseCreated(contract_id));

        Self::execution_required(child_flow_id, &child_flow)?;

        Ok(())
    }

    fn execution_required(
        child_flow_id: T::InstanceId,
        child_flow: &Iflow<T>,
    ) -> Result<(), &'static str> {
        Self::ensure_idata_instance_exists(child_flow_id)?;

        let first_elem = child_flow.get_first_elem();
        <IdataById<T>>::mutate(child_flow_id, |idata| {
            let post_condition = child_flow.get_post_condition(first_elem);
            idata.set_marking(post_condition);
        });
        let next = child_flow.get_ady_elements(first_elem);
        if !next.is_empty() {
            Self::execute_elements(child_flow_id, next[0])?;
        }
        Ok(())
    }

    /// This function only receive THROW EVENTS (throw event verification made in function executeElement)
    fn throw_event(
        parent_case: T::InstanceId,
        idata: &Idata<T>,
        event_code: [u8; 32],
        event_info: u128,
    ) -> Result<(), &'static str> {
        let mut parent_state: [u128; 2] = [0; 2];
        parent_state[0] = idata.get_marking();
        parent_state[0] = idata.get_started_activities();
        match event_info {
            event_info if event_info & 4096 == 4096 => {
                // Message (BIT 15), to publish a Message in the Event Log
                Self::deposit_event(RawEvent::MessageSent(event_code.to_vec()));
            }
            event_info if event_info & 5632 == 5632 => {
                // 9- End, 10- Default, 12- Message
                // If there are not tokens to consume nor started activities in any subprocess
                if parent_state[0] | parent_state[1] == 0 {
                    // Sub-process ended, thus continue execution on parent
                    Self::try_catch_event(parent_case, idata, event_code, event_info, true)?;
                }
            }
            event_info => {
                if event_info & 2048 == 2048 {
                    // Terminate Event (BIT 11), only END EVENT from standard,
                    // Terminate the execution in the current Sub-process and each children
                    Self::kill_process(parent_case)?;
                }
                Self::try_catch_event(
                    parent_case,
                    idata,
                    event_code,
                    event_info,
                    parent_state[0] | parent_state[1] == 0,
                )?;
            }
        }
        Ok(())
    }

    fn try_catch_event(
        parent_case: T::InstanceId,
        idata: &Idata<T>,
        event_code: [u8; 32],
        event_info: u128,
        instance_completed: bool,
    ) -> Result<(), &'static str> {
        if let Some(catch_case) = idata.get_idata_parent() {
            let mut catch_case_data = Self::ensure_idata_instance_exists(catch_case)?;
            let child_flow = idata.get_flow_node();
            let child_flow_instance = Self::ensure_iflow_instance_exists(child_flow)?;
            let mut parent_state: [u128; 2] = [0; 2];
            parent_state[0] = catch_case_data.get_marking();
            parent_state[1] = catch_case_data.get_started_activities();
            let sub_process_index = idata.get_index_in_parent();
            let run_inst_count = if instance_completed {
                <IdataById<T>>::mutate(catch_case, |catch_case_data| {
                    catch_case_data.decrement_instance_count(sub_process_index)
                });
                catch_case_data.get_instance_count(sub_process_index) - 1
            } else {
                catch_case_data.get_instance_count(sub_process_index)
            };
            if run_inst_count == 0 {
                // Update the corresponding sub-process, call activity as completed
                <IdataById<T>>::mutate(catch_case, |catch_case| {
                    catch_case
                        .set_activity_marking(parent_state[1] & !(1 << 1 << sub_process_index))
                });
            }
            let sub_process_info = child_flow_instance.get_instance_count(sub_process_index);
            if event_info & 7168 != 0 {
                // If receiving 10- Default, 11- Terminate or 12- Message
                if run_inst_count == 0 && sub_process_info & 4096 != 4096 {
                    // No Instances of the sub-process propagating the event and The sub-process isn't an event-sub-process (BIT 12)
                    let post_condition = child_flow_instance.get_post_condition(sub_process_index);
                    <IdataById<T>>::mutate(catch_case, |catch_case| {
                        catch_case.set_marking(parent_state[0] & !post_condition)
                    });
                    let first_ady_element =
                        child_flow_instance.get_ady_elements(sub_process_info)[0];
                    Self::execute_elements(catch_case, first_ady_element)?;
                } else if sub_process_info & 128 == 128 {
                    // Multi-Instance Sequential (BIT 7), with pending instances to be started.
                    Self::create_instance(sub_process_index, parent_case)?;
                }
            } else {
                // Signal, Error or Escalation
                // Signals are only handled from the Root-Process by Broadcast, thus the propagation must reach the Root-Process.
                if event_info & 32768 == 32768 {
                    // Propagating the Signal to the Root-Process
                    while let Some(parent_case) = catch_case_data.get_idata_parent() {
                        catch_case_data = Self::ensure_idata_instance_exists(parent_case)?;
                    }
                    Self::broadcast_signal(parent_case)?;
                    return Ok(());
                }
                let events = child_flow_instance.get_event_list();

                // The event can be catched only once, unless it is a signal where a broadcast must happen.
                // Precondition: Event-subprocess must appear before boundary events on the event list.
                for event in events {
                    let ev_code = child_flow_instance.get_event_code(*event);
                    if event_code == ev_code {
                        // Verifiying there is a match with the throw-cath events.
                        let catch_event_info = child_flow_instance.get_type_info(*event);
                        let attached_to = child_flow_instance.get_attached_to(*event);

                        if catch_event_info & 6 == 6 {
                            // Start event-sub-process (BIT 6)
                            if catch_event_info & 16 == 16 {
                                // Interrupting (BIT 4 must be 1, 0 if non-interrupting)
                                // Before starting the event subprocess, the parent is killed
                                Self::kill_process(catch_case)?;
                            }

                            // Starting event sub-process
                            Self::create_instance(attached_to, parent_case)?;

                            // Marking the event-sub-process as started
                            <IdataById<T>>::mutate(catch_case, |catch_case| {
                                catch_case
                                    .set_activity_marking(parent_state[1] | (1 << attached_to))
                            });
                            return Ok(());
                        } else if catch_event_info & 256 == 256 && attached_to == sub_process_index
                        {
                            // Boundary (BIT 6) of the subproces propagating the event
                            if catch_event_info & 16 == 16 {
                                // Interrupting (BIT 4 must be 1, 0 if non-interrupting)
                                Self::kill_process(parent_case)?;
                            }

                            // The subprocess propagating the event must be interrupted
                            let post_condition = child_flow_instance.get_post_condition(*event);
                            let first_ady_element = child_flow_instance.get_ady_elements(*event)[0];

                            // Update the marking with the output of the boundary event
                            <IdataById<T>>::mutate(catch_case, |catch_case| {
                                catch_case.set_marking(parent_state[0] & !post_condition)
                            });
                            Self::execute_elements(catch_case, first_ady_element)?;
                            return Ok(());
                        }
                    }
                }
                // If the event was not caught the propagation continues to the parent unless it's the root process
                Self::throw_event(catch_case, &catch_case_data, event_code, event_info)?;
            }
        } else {
            // No Parent exist, root node
            if event_info & 8192 == 8192 {
                // Error event (BIT 13), only END EVENT from standard, in the root process.
                Self::kill_process(parent_case)?;
            }
        }
        Ok(())
    }

    fn kill_process(parent_case: T::InstanceId) -> Result<(), &'static str> {
        let catch_case_data = Self::ensure_idata_instance_exists(parent_case)?;
        let started_activities = catch_case_data.get_started_activities();
        let child_flow_index = catch_case_data.get_flow_node();
        let child_flow_instance = Self::ensure_iflow_instance_exists(child_flow_index)?;
        let children = child_flow_instance.get_sub_process_list();

        <IdataById<T>>::mutate(parent_case, |parent_case_data| {
            parent_case_data.set_marking(0);
            parent_case_data.set_activity_marking(0);
        });

        for child in children {
            if started_activities & (1 << child) != 0 {
                let child_proc_instance = catch_case_data.get_child_process_instances(*child);
                Self::kill_processes(child_proc_instance)?;
            }
        }
        Ok(())
    }

    fn kill_processes(parent_cases: &[T::InstanceId]) -> Result<(), &'static str> {
        for &parent_case in parent_cases {
            Self::kill_process(parent_case)?;
        }
        Ok(())
    }

    fn broadcast_signal(parent_case: T::InstanceId) -> Result<(), &'static str> {
        let parent_case_instance = Self::ensure_idata_instance_exists(parent_case)?;
        let child_flow_index = parent_case_instance.get_flow_node();
        let child_flow_instance = Self::ensure_iflow_instance_exists(child_flow_index)?;

        let events = child_flow_instance.get_event_list();
        for &event in events {
            let event_info = child_flow_instance.get_type_info(event);

            if event_info & 32780 == 32772 {
                // Event Catch Signal (BITs 2, 3 [0-catch, 1-throw], 15)
                let catch_event_info = child_flow_instance.get_type_info(event);
                let attached_to = child_flow_instance.get_attached_to(event);

                if catch_event_info & 6 == 6 {
                    // Start event-sub-process (BIT 6)
                    if catch_event_info & 16 == 16 {
                        // Interrupting (BIT 4 must be 1, 0 if non-interrupting)
                        // Before starting the event subprocess, the current process-instance is killed
                        Self::kill_process(parent_case)?;
                        Self::create_instance(attached_to, parent_case)?;
                        <IdataById<T>>::mutate(parent_case, |parent_case_instance| {
                            parent_case_instance.set_activity_marking(1 << attached_to);
                        });
                    } else if catch_event_info & 256 == 256 {
                        // Boundary (BIT 6) of the subproces propagating the event
                        if catch_event_info & 16 == 16 {
                            // Interrupting (BIT 4 must be 1, 0 if non-interrupting)
                            // The subprocess propagating the event must be interrupted
                            let child_process_instances =
                                parent_case_instance.get_child_process_instances(attached_to);
                            Self::kill_processes(child_process_instances)?;
                        }
                        let marking = parent_case_instance.get_marking();
                        let post_condition = child_flow_instance.get_post_condition(event);

                        // Update the marking with the output of the boundary event
                        <IdataById<T>>::mutate(parent_case, |parent_case_data| {
                            parent_case_data.set_marking(marking & !post_condition);
                        });
                        let first_ady_element = child_flow_instance.get_ady_elements(event)[0];

                        // Continue the execution of possible internal elements
                        Self::execute_elements(parent_case, first_ady_element)?;
                    } else if event_info & 160 == 160 {
                        // Start (not Event Subprocess) OR Intermediate Event
                        let marking = parent_case_instance.get_marking();
                        let post_condition = child_flow_instance.get_post_condition(event);
                        let pre_condition = child_flow_instance.get_pre_condition(event);
                        let first_ady_element = child_flow_instance.get_ady_elements(event)[0];

                        <IdataById<T>>::mutate(parent_case, |parent_case_data| {
                            parent_case_data.set_marking(marking & !pre_condition | post_condition);
                        });

                        // Continue the execution of possible internal elements
                        Self::execute_elements(parent_case, first_ady_element)?;
                    }
                }
            }
            let children = child_flow_instance.get_sub_process_list();
            let started_activities = parent_case_instance.get_started_activities();
            for &child in children {
                if started_activities & (1 << child) != 0 {
                    let child_proc_instances =
                        parent_case_instance.get_child_process_instances(child);
                    Self::broadcast_signals(child_proc_instances)?;
                }
            }
        }
        Ok(())
    }

    fn broadcast_signals(parent_cases: &[T::InstanceId]) -> Result<(), &'static str> {
        for &parent_case in parent_cases {
            Self::broadcast_signal(parent_case)?;
        }
        Ok(())
    }

    fn execute_elements(
        parent_case: T::InstanceId,
        mut element_index: u128,
    ) -> Result<(), &'static str> {
        let idata = Self::ensure_idata_instance_exists(parent_case)?;
        let child_flow_index = idata.get_flow_node();
        let child_flow = Self::ensure_iflow_instance_exists(child_flow_index)?;
        // 0- tokensOnEdges
        // 1- startedActivities
        let mut parent_state: [u128; 2] = [0; 2];
        parent_state[0] = idata.get_marking();
        parent_state[1] = idata.get_started_activities();

        // Execution queue and pointers to the first & last element (i.e. basic circular queue implementation)
        let mut queue: [u128; 100] = [0; 100];
        let mut i: usize = 0;
        let mut count: usize = 0;
        queue[count] = element_index;
        count += 1;
        while i < count {
            element_index = queue[i];
            i += 1;
            let ([pre_condition, post_condition, type_info], next) =
                child_flow.get_element_info(element_index);

            // Verifying Preconditions (i.e. Is the element enabled?)
            match type_info {
                type_info if type_info & 42 == 42 => {
                    // else if (AND Join)
                    if parent_state[0] & pre_condition != pre_condition {
                        continue;
                    }
                    parent_state[0] &= !pre_condition;
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
                    if parent_state[0] & pre_condition == 0 {
                        continue;
                    }
                    // Removing tokens from input arcs
                    parent_state[0] &= !pre_condition;
                }
                _ => {
                    continue;
                }
            }

            // Executing current element (If enabled)
            match type_info {
                type_info if type_info & 65 == 65 => {
                    // (0- Activity, 6- Parallel Multi-Instance)
                    let child_instances = child_flow.get_instance_count(element_index);
                    for _ in 0..child_instances {
                        Self::create_instance(element_index, parent_case)?;
                    }
                    parent_state[1] |= 1 << element_index;
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
                    Self::create_instance(element_index, parent_case)?;
                    <IdataById<T>>::mutate(parent_case, |idata| {
                        let instance_count = child_flow.get_instance_count(element_index);
                        idata.set_instance_count(element_index, instance_count);
                    });

                    parent_state[1] |= 1 << element_index;
                }
                type_info
                    if (type_info & 4105 == 4105
                        || (type_info & 10 == 2 && type_info & 80 != 0)) =>
                {
                    // (0- Activity, 3- Task, 12- Script) ||
                    // Exclusive(XOR) Split (1- Gateway, 3- Split(0), 4- Exclusive) ||
                    // Inclusive(OR) Split (1- Gateway, 3- Split(0), 6- Inclusive)
                    let factory = child_flow.get_factory_instance();

                    let origin = T::Origin::from(RawOrigin::Root);
                    let account_id = ensure_signed(origin)?;

                    if let Some(address) = factory.get_address() {
                        match <contracts::Module<T>>::bare_call(
                            account_id,
                            address.clone(),
                            ENDOWMENT.into(),
                            GAS.into(),
                            factory.get_execute_script_selector().to_vec(),
                        ) {
                            Ok(result) => {
                                parent_state[0] |= u128::decode(&mut &result.data[..])
                                    .map_err(|_| DECODING_ERROR)?;
                            }
                            Err(e) => return Err(e.reason.into()),
                        }
                    }
                }
                type_info
                    if ((type_info & 9 == 9 && type_info & 27657 != 0) || type_info & 2 == 2) =>
                {
                    // If (User(11), Service(13), Receive(14) or Default(10) Task || Gateways(1) not XOR/OR Split)
                    // The execution of User/Service/Receive is triggered off-chain,
                    // Thus the starting point would be the data contract which executes any script/data-update related to the task.
                    parent_state[0] |= post_condition;
                }
                type_info if type_info & 12 == 12 => {
                    // If (2- Event, 3- Throw(1))
                    <IdataById<T>>::mutate(parent_case, |idata| {
                        idata.set_marking(parent_state[0]);
                        idata.set_activity_marking(parent_state[1]);
                    });
                    let event_code = child_flow.get_event_code(element_index);
                    Self::throw_event(parent_case, &idata, event_code, type_info)?;
                    let marking = idata.get_marking();
                    let started_activities = idata.get_started_activities();
                    if marking | started_activities == 0 {
                        // By throwing the event, a kill was performed so the current instance was terminated
                        return Ok(());
                    }
                    parent_state[0] = marking;
                    parent_state[1] = started_activities;
                    if type_info & 128 == 128 {
                        // If Intermediate event (BIT 7)
                        parent_state[0] |= post_condition;
                    }
                }
                _ => (),
            }

            // Adding the possible candidates to be executed to the queue.
            // The enablement of the element is checked at the moment it gets out of the queue.
            for next_elem in next {
                queue[count] = *next_elem;
                count = (count + 1) % 100;
            }
        }

        // Updating the state (storage) after the execution of each internal element.
        <IdataById<T>>::mutate(parent_case, |idata| {
            idata.set_marking(parent_state[0]);
            idata.set_activity_marking(parent_state[1]);
        });
        Ok(())
    }

    fn ensure_iflow_instance_exists(instance_id: T::InstanceId) -> Result<Iflow<T>, &'static str> {
        if <IflowById<T>>::contains_key(instance_id) {
            Ok(Self::iflow_by_id(instance_id))
        } else {
            Err(INSTANCE_ID_NOT_FOUND)
        }
    }

    fn ensure_idata_instance_exists(instance_id: T::InstanceId) -> Result<Idata<T>, &'static str> {
        if <IdataById<T>>::contains_key(instance_id) {
            Ok(Self::idata_by_id(instance_id))
        } else {
            Err(INSTANCE_ID_NOT_FOUND)
        }
    }

    fn ensure_subprocess_to_link_in_data_structure(
        iflow: &Iflow<T>,
        parent_index: u128,
    ) -> DispatchResult {
        //BITs (0, 5) Veryfing the subprocess to link is already in the data structure
        ensure!(
            iflow.get_type_info(parent_index) & 33 != 33,
            SUBPROCESS_TO_LINK_NOT_FOUND
        );
        Ok(())
    }
}

decl_event!(
    pub enum Event<T>
    where
        InstanceId = <T as Trait>::InstanceId,
        Hash = <T as frame_system::Trait>::Hash,
        AccountId = <T as frame_system::Trait>::AccountId,
    {
        FactorySet(InstanceId, Hash),
        NewCaseCreated(AccountId),
        MessageSent(Vec<u8>),
    }
);
