#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Codec, Decode, Encode};
use frame_support::{
    decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure, Parameter,
};
use sp_runtime::traits::{MaybeSerialize, Member, SimpleArithmetic};
use sp_std::collections::btree_map::BTreeMap;
use system::ensure_signed;

mod errors;
use errors::*;

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
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
    factory:  Ifactory<T>
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
	
	fn get_first_elem(&self) -> u128 {
		self.start_evt
	}

	fn get_ady_elements(&self, element_index: u128) -> &[u128] {
		&self.next_elem[&element_index]
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
        self.cond_table.insert(element_index,  [pre_condition, post_condition, type_info]);
        self.next_elem.insert(element_index,   _next_elem);
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
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Idata<T: Trait>  {
	tokens_on_edges: u128,
	started_activities: u128,
	idata_parent: Option<T::InstanceId>,
	iflow_node: T::InstanceId,
	index_in_parent: u128,
	children: BTreeMap<u128, Vec<T::InstanceId>>,
	instance_count: BTreeMap<u128, u128>
}

impl <T: Trait> Idata<T> {
	fn set_marking(&mut self, n_marking: u128) {
		self.tokens_on_edges = n_marking
	}

	fn set_activity_marking(&mut self, n_marking: u128) {
		self.started_activities = n_marking
	}

	fn set_parent(&mut self, idata_parent: Option<T::InstanceId>, iflow_node: T::InstanceId, index_in_parent: u128) {
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
			instance_count.checked_sub(1);
		}
	}

	fn set_instance_count(&mut self, element_index: u128, new_instance_count: u128) {
		if let Some(instance_count) = self.instance_count.get_mut(&element_index) {
			*instance_count = new_instance_count;
		} else {
			self.instance_count.insert(element_index, new_instance_count);
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

	fn continue_execution(&self, element_index: u128) {
		// Call bpmn interpreter execution on given index
	}
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Ifactory<T: Trait> {
    /// Data & scripts hash
    data_hash: T::Hash,
}

impl <T: Trait> Ifactory <T> {
	fn new(data_hash: T::Hash) -> Self {
		Self {
			data_hash
		}
    }
    
    fn new_instance(&self) -> T::AccountId {
        // Initialize new instance of data & scripts contract
        T::AccountId::default()
    }
}

/// The pallet's configuration trait.
pub trait Trait: system::Trait + Default {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    /// Type of identifier for instances.
    type InstanceId: Parameter
        + Member
        + SimpleArithmetic
        + Codec
        + Default
        + Copy
        + MaybeSerialize
        + PartialEq;
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as TemplateModule {
		IflowById get(fn iflow_by_id): map T::InstanceId => Iflow<T>;
		
		IdataById get(fn idata_by_id): map T::InstanceId => Idata<T>;

        InstanceIdCount get(fn instance_id_count): T::InstanceId;
    }
}

// The pallet's dispatchable functions.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        // this is needed only if you are using events in your pallet
        fn deposit_event() = default;

        fn set_element(
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

			<IflowById<T>>::mutate(iflow_index, |inner_iflow| 
			inner_iflow.set_element(
                element_index,
                pre_condition,
                post_condition,
                type_info,
                event_code,
				_next_elem)
			);
            Ok(())
        }

        fn link_sub_process(
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
		
		fn set_factory_instance(origin, instance_id: T::InstanceId, data_hash: T::Hash) -> DispatchResult {
			ensure_signed(origin)?;
			let factory = Ifactory::new(data_hash);
			<IflowById<T>>::mutate(instance_id, |iflow| iflow.set_factory_instance(factory));
			Self::deposit_event(RawEvent::FactorySet(instance_id, data_hash));
			Ok(())
    	}
    }
}

impl<T: Trait> Module<T> {
    /// BPMN Interpreter logic
    
    /// Instantiation of Root-Process
    pub fn create_root_instance(parent_case: T::InstanceId) -> DispatchResult {
        let iflow = Self::ensure_iflow_instance_exists(parent_case)?;
        
        let ifactory = iflow.get_factory_instance();

        ifactory.new_instance();

        let mut idata = Idata::default();

        idata.set_parent(None, parent_case, 0);

        <IdataById<T>>::insert(parent_case, idata);

        Self::deposit_event(RawEvent::NewCaseCreated(parent_case));

        Self::execution_required(parent_case);

        Ok(())
    }

    /// Instantiation of a sub-process by its parent
    pub fn create_instance(element_index: u128, parent_case: T::InstanceId) -> DispatchResult {

        ensure!(parent_case != T::InstanceId::default(), "Parent case should not be root");

        let idata = Self::ensure_idata_instance_exists(parent_case)?;

        let parent_flow_id = idata.get_flow_node();
        let parent_flow = Self::ensure_iflow_instance_exists(parent_flow_id)?;

        let child_flow_id = parent_flow.get_sub_process_instance(element_index);
        let child_flow = Self::ensure_iflow_instance_exists(child_flow_id)?;
        
        let ifactory = child_flow.get_factory_instance();

        ifactory.new_instance();

        <IdataById<T>>::mutate(child_flow_id, |inner_data| inner_data.set_parent(Some(parent_case), child_flow_id, element_index));
        <IdataById<T>>::mutate(parent_case, |inner_data| inner_data.add_child(element_index, child_flow_id));

        Self::execution_required(child_flow_id);

        Ok(())
    }

    fn execution_required(parent_case: T::InstanceId) -> DispatchResult {

        Ok(())
    }

    fn ensure_iflow_instance_exists(instance_id: T::InstanceId) -> Result<Iflow<T>, &'static str> {
        if <IflowById<T>>::exists(instance_id) {
            Ok(Self::iflow_by_id(instance_id))
        } else {
            Err(INSTANCE_ID_NOT_FOUND)
        }
    }

    fn ensure_idata_instance_exists(instance_id: T::InstanceId) -> Result<Idata<T>, &'static str> {
        if <IdataById<T>>::exists(instance_id) {
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
		Hash = <T as system::Trait>::Hash {
        FactorySet(InstanceId, Hash),
        NewCaseCreated(InstanceId),
    }
);
