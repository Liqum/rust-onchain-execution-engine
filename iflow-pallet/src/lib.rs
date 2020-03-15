#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage, Parameter, decl_event, dispatch::DispatchResult, StorageMap, StorageValue};
use system::{self, ensure_signed};
use codec::{Codec, Decode, Encode};
use sp_runtime::traits::{MaybeSerialize, Member, One, SimpleArithmetic};

/// The pallet's configuration trait.
pub trait Trait: system::Trait {
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
		StartEvent get(fn start_event): map T::InstanceId => u128;
		Factory get(fn factory): map T::InstanceId => T::InstanceId;
		Interpreter get(fn interpreter): map T::InstanceId => T::InstanceId;
		// elemIndex => [preC, postC, type]
		CondTable get(fn cond_table): map (T::InstanceId, u128) => [u128; 3];
		// Element Index => List of elements that can be enabled with the completion of the key element
		NextElem get(fn next_elem): map (T::InstanceId, u128) => Vec<u128>;
		// List of Indexes of the subprocesses
		SubProcesses get(fn subprocesses): map T::InstanceId => Vec<u128>;
		// List of Event Indexes defined in the current Subprocess
		Events get(fn events): map T::InstanceId => Vec<u128>;
		// Event Index => Index of the element where event is attachedTo
		AttachedTo get(fn attached_to): map (T::InstanceId, u128) => u128;
		// Event Index => String representing the code to identify the event (for catching)
		EventCode get(fn event_code): map (T::InstanceId, u128) => [u8; 32];
		// Subprocess Index => Child Subproces address
		ParentRefernces get (fn parent_references): map (T::InstanceId, u128) => T::InstanceId;
		// Subprocess Index => number of instances
		InstanceCount get(fn instance_count): map (T::InstanceId, u128) => u128;
		// InstanceIds counter
		InstanceIdCount get(fn instance_id_count): T::InstanceId;
	}
}

// The pallet's dispatchable functions.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your pallet
		fn deposit_event() = default;
	}
}

impl<T: Trait> Module<T> {
	
	fn get_pre_condition(iflow_id: T::InstanceId, element_index: u128) -> u128 {
		Self::cond_table((iflow_id, element_index))[0]
	}
	
	fn get_post_condition(iflow_id: T::InstanceId, element_index: u128) -> u128 {
		Self::cond_table((iflow_id, element_index))[1]
	}
	
	fn get_type_info(iflow_id: T::InstanceId, element_index: u128) -> u128 {
		Self::cond_table((iflow_id, element_index))[2]
	}
	
	fn set_factory_instance(iflow_id: T::InstanceId, factory_id: T::InstanceId) {
		<Factory<T>>::insert(iflow_id, factory_id);
		Self::deposit_event(RawEvent::FactorySet(iflow_id, factory_id))
	}

	fn set_element(iflow_index: T::InstanceId, element_index: u128, pre_condition: u128, post_condition: u128, type_info: u128, event_code: [u8; 32], _next_elem: Vec<u128>) {
		if !<StartEvent<T>>::exists(iflow_index) {
			<InstanceIdCount<T>>::mutate(|count| *count += T::InstanceId::one());
		}
		let _type_info = Self::get_type_info(iflow_index, element_index);
		match _type_info {
			0 => {
				if type_info & 4 == 4 {
					if <Events<T>>::exists(iflow_index) {
						<Events<T>>::mutate(iflow_index, |events| events.push(element_index));
					} else {
						<Events<T>>::insert(iflow_index, vec![element_index])
					}
					if type_info & 36 == 36 {
						<StartEvent<T>>::insert(iflow_index, element_index);
					}
					<EventCode<T>>::insert((iflow_index, element_index), event_code);
				} else if type_info & 33 == 33 {
					if <SubProcesses<T>>::exists(iflow_index) {
						<SubProcesses<T>>::mutate(iflow_index, |subprocesses| subprocesses.push(element_index))
					} else {
						<SubProcesses<T>>::insert(iflow_index, vec![element_index])
					}
				}
			}
			_ => {
				//"Should be equal!"
				if type_info != _type_info {
					return;
				}
			}
		}
		<CondTable<T>>::insert((iflow_index, element_index), [pre_condition, post_condition, type_info]);
		<NextElem<T>>::insert((iflow_index, element_index), _next_elem);
	}

	fn link_sub_process(
		iflow_index: T::InstanceId,
		parent_index: u128,
		child_flow_inst: T::InstanceId,
		attached_events: Vec<u128>,
		count_instances: u128,
	) {
		//BITs (0, 5) Veryfing the subprocess to link is already in the data structure
		if Self::get_type_info(iflow_index, parent_index) & 33 != 33 {
			return;
		}
		<ParentRefernces<T>>::insert((iflow_index, parent_index), child_flow_inst);
		for attached_event in attached_events.iter() {
			if Self::get_type_info(iflow_index, parent_index) & 4 == 4 {
				<AttachedTo<T>>::insert((iflow_index, *attached_event), parent_index);
			}
		}
		<InstanceCount<T>>::insert((iflow_index, parent_index), count_instances);
	}
}

decl_event!(
	pub enum Event<T>
	where
		InstanceId = <T as Trait>::InstanceId,
    {
        FactorySet(InstanceId, InstanceId),
    }
);